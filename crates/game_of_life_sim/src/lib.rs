mod camera;
mod canvas_data;
mod dsl;
mod pipelines;
mod shaders;

use camera::{CameraProjection, OrthographicCamera, CAMERA_MOVE_SPEED};
use instant::Instant;

use bytemuck::{Pod, Zeroable};
use canvas_data::CanvasData;
use glam::{Mat4, Quat, Vec2, Vec3};
use glass::{
    pipelines::QuadPipeline,
    wgpu,
    window::GlassWindow,
    winit::{self, event::VirtualKeyCode},
    GlassApp, GlassContext, RenderData,
};
use pipelines::Pipelines;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub const SIM_SIZE: u32 = 1024;
pub const WORK_GROUP_SIZE: u32 = 32;
pub const FPS_60: f32 = 16.0 / 1000.0;

#[rustfmt::skip]
const OPENGL_TO_WGPU: glam::Mat4 = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

pub struct GameOfLifeApp {
    dt_sum: f32,
    num_dts: f32,
    count: usize,
    time: Instant,
    updated_time: Instant,

    drawing: bool,
    cursor_pos: Vec2,
    prev_cursor_pos: Option<Vec2>,

    camera: camera::OrthographicCamera,

    data: Option<CanvasData>,
    quad_pipeline: Option<QuadPipeline>,
    init_pipeline: Option<wgpu::ComputePipeline>,
    draw_pipeline: Option<wgpu::ComputePipeline>,
    game_of_life_pipeline: Option<wgpu::ComputePipeline>,
}

impl Default for GameOfLifeApp {
    fn default() -> Self {
        Self {
            count: 0,
            dt_sum: 0.0,
            num_dts: 0.0,
            time: Instant::now(),
            updated_time: Instant::now(),

            drawing: false,
            prev_cursor_pos: None,
            cursor_pos: Default::default(),

            camera: camera::OrthographicCamera::default(),

            data: None,
            quad_pipeline: None,
            init_pipeline: None,
            draw_pipeline: None,
            game_of_life_pipeline: None,
        }
    }
}

impl GameOfLifeApp {
    fn cursor_to_canvas(&self, width: f32, height: f32) -> (Vec2, Vec2) {
        let half_screen = Vec2::new(width, height) / 2.0;
        let current_canvas_pos = self.cursor_pos - half_screen + SIM_SIZE as f32 / 2.0;
        let prev_canvas_pos = self.prev_cursor_pos.unwrap_or(current_canvas_pos) - half_screen
            + SIM_SIZE as f32 / 2.0;
        (current_canvas_pos, prev_canvas_pos)
    }
}

// Think of this like reading a "table of contents".
// - Start is run before event loop
// - Input is run on winit input
// - Update is run every frame
// - Render is run for each window after update every frame
impl GlassApp for GameOfLifeApp {
    fn start(
        &mut self,
        _event_loop: &winit::event_loop::EventLoop<()>,
        context: &mut GlassContext,
    ) {
        // Create pipelines
        let Pipelines {
            init_pipeline,
            game_of_life_pipeline,
            draw_pipeline,
        } = Pipelines::load(context);

        let quad_pipeline = QuadPipeline::new(context.device(), GlassWindow::surface_format());
        self.data = Some(CanvasData::create(
            context,
            &quad_pipeline,
            &init_pipeline,
            &draw_pipeline,
        ));

        self.quad_pipeline = Some(quad_pipeline);
        self.init_pipeline = Some(init_pipeline);
        self.draw_pipeline = Some(draw_pipeline);
        self.game_of_life_pipeline = Some(game_of_life_pipeline);

        init_game_of_life(self, context);
    }

    fn input(
        &mut self,
        _context: &mut GlassContext,
        _event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        event: &winit::event::Event<()>,
    ) {
        handle_inputs(self, event);
    }

    fn update(&mut self, context: &mut GlassContext) {
        run_update(self, context);
    }

    fn render(&mut self, _context: &GlassContext, render_data: RenderData) {
        render(self, render_data);
    }
}

fn run_update(app: &mut GameOfLifeApp, context: &mut GlassContext) {
    let now = Instant::now();
    app.dt_sum += (now - app.time).as_secs_f32();
    app.num_dts += 1.0;
    if app.num_dts == 100.0 {
        // Set fps
        context.primary_render_window().window().set_title(&format!(
            "Game Of Life: {:.2}",
            1.0 / (app.dt_sum / app.num_dts)
        ));
        app.num_dts = 0.0;
        app.dt_sum = 0.0;
    }
    app.time = Instant::now();

    // Use only single command queue
    let mut encoder = context
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Computes"),
        });

    let (width, height) = {
        let size = context.primary_render_window().window().inner_size();
        (size.width as f32, size.height as f32)
    };
    app.camera.update(width, height);

    // Update 60fps
    if (app.time - app.updated_time).as_secs_f32() > FPS_60 {
        update_game_of_life(app, context, &mut encoder);
        app.updated_time = app.time;
    }

    if app.drawing {
        draw_game_of_life(app, context, &mut encoder);
    }

    // Update prev cursor pos
    app.prev_cursor_pos = Some(app.cursor_pos);

    // Submit
    context.queue().submit(Some(encoder.finish()));
}

fn render(app: &mut GameOfLifeApp, render_data: RenderData) {
    let GameOfLifeApp {
        data,
        quad_pipeline,
        ..
    } = app;

    let canvas_data = data.as_ref().unwrap();
    let quad_pipeline = quad_pipeline.as_ref().unwrap();
    let RenderData { encoder, frame, .. } = render_data;

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            depth_stencil_attachment: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
            })],
        });

        quad_pipeline.draw(
            &mut rpass,
            &canvas_data.canvas_bind_group,
            [0.0; 4],
            camera_projection(&app.camera).to_cols_array_2d(),
            canvas_data.canvas.size,
        );
    }
}

fn handle_inputs(app: &mut GameOfLifeApp, event: &winit::event::Event<()>) {
    if let winit::event::Event::WindowEvent { event, .. } = event {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                app.cursor_pos = Vec2::new(position.x as f32, position.y as f32);
            }
            winit::event::WindowEvent::MouseInput {
                button: winit::event::MouseButton::Left,
                state,
                ..
            } => {
                app.drawing = state == &winit::event::ElementState::Pressed;
            }
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                println!("{:?}", app.dt_sum);
                if let Some(key) = input.virtual_keycode {
                    // Move camera with arrows & WASD
                    let up = key == VirtualKeyCode::W || key == VirtualKeyCode::Up;
                    let down = key == VirtualKeyCode::S || key == VirtualKeyCode::Down;
                    let left = key == VirtualKeyCode::A || key == VirtualKeyCode::Left;
                    let right = key == VirtualKeyCode::D || key == VirtualKeyCode::Right;

                    let x_axis = -(right as i8) + left as i8;
                    let y_axis = -(up as i8) + down as i8;
                    let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
                    if move_delta != Vec2::ZERO {
                        move_delta /= move_delta.length();
                        app.camera.translate(move_delta * 0.01 * CAMERA_MOVE_SPEED);
                    }
                }
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                // I just took this from three-rs, no idea why this magic number was chosen ¯\_(ツ)_/¯
                const PIXELS_PER_LINE: f64 = 38.0;

                let mut x_scroll_diff = 0.0;
                let mut y_scroll_diff = 0.0;

                match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        x_scroll_diff += x;
                        y_scroll_diff += y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {
                        y_scroll_diff += (delta.y / PIXELS_PER_LINE) as f32;
                        x_scroll_diff += (delta.x / PIXELS_PER_LINE) as f32;
                    }
                }

                if x_scroll_diff != 0.0 || y_scroll_diff != 0.0 {
                    if y_scroll_diff < 0.0 {
                        app.camera.zoom(1.05)
                    } else {
                        app.camera.zoom(1.0 / 1.05);
                    }
                }
            }
            _ => (),
        }
    }
}

fn draw_game_of_life(
    app: &mut GameOfLifeApp,
    context: &mut GlassContext,
    encoder: &mut wgpu::CommandEncoder,
) {
    let (width, height) = {
        let size = context.primary_render_window().window().inner_size();
        (size.width as f32, size.height as f32)
    };

    let (end, start) = app.cursor_to_canvas(width, height);
    let GameOfLifeApp {
        data,
        draw_pipeline,
        ..
    } = app;

    let data = data.as_ref().unwrap();
    let draw_pipeline = draw_pipeline.as_ref().unwrap();
    let pc = GameOfLifePushConstants::new(start, end, 10.0);

    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("draw_game_of_life"),
    });
    cpass.set_pipeline(draw_pipeline);
    cpass.set_bind_group(0, &data.draw_bind_group, &[]);
    cpass.set_push_constants(0, bytemuck::cast_slice(&[pc]));
    cpass.dispatch_workgroups(SIM_SIZE / WORK_GROUP_SIZE, SIM_SIZE / WORK_GROUP_SIZE, 1);
}

fn update_game_of_life(
    app: &mut GameOfLifeApp,
    context: &GlassContext,
    encoder: &mut wgpu::CommandEncoder,
) {
    let GameOfLifeApp {
        data,
        game_of_life_pipeline,
        ..
    } = app;

    let data = data.as_ref().unwrap();
    let game_of_life_pipeline = game_of_life_pipeline.as_ref().unwrap();

    let (canvas, data_in) = if app.count % 2 == 0 {
        (&data.canvas.views[0], &data.data_in.views[0])
    } else {
        (&data.data_in.views[0], &data.canvas.views[0])
    };

    let update_bind_group = context
        .device()
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Update Bind Group"),
            layout: &game_of_life_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(canvas),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(data_in),
                },
            ],
        });

    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Update"),
    });
    cpass.set_pipeline(game_of_life_pipeline);
    cpass.set_bind_group(0, &update_bind_group, &[]);
    cpass.dispatch_workgroups(SIM_SIZE / WORK_GROUP_SIZE, SIM_SIZE / WORK_GROUP_SIZE, 1);

    app.count += 1;
}

fn init_game_of_life(app: &mut GameOfLifeApp, context: &mut GlassContext) {
    let GameOfLifeApp {
        data,
        init_pipeline,
        ..
    } = app;

    let data = data.as_ref().unwrap();
    let init_pipeline = init_pipeline.as_ref().unwrap();

    let mut encoder = context
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Init"),
        });
        cpass.set_pipeline(init_pipeline);
        cpass.set_bind_group(0, &data.init_bind_group, &[]);
        cpass.dispatch_workgroups(SIM_SIZE / WORK_GROUP_SIZE, SIM_SIZE / WORK_GROUP_SIZE, 1);
    }
    context.queue().submit(Some(encoder.finish()));
}

// =============================== CAMERA =============================== //

fn camera_projection(camera: &OrthographicCamera) -> glam::Mat4 {
    OPENGL_TO_WGPU
        * camera.ortho.get_projection_matrix()
        * Mat4::from_scale_rotation_translation(Vec3::ONE, Quat::IDENTITY, camera.pos.extend(-10.0))
}

// =============================== MISC =============================== //

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GameOfLifePushConstants {
    draw_start: [f32; 2],
    draw_end: [f32; 2],
    draw_radius: f32,
}

impl GameOfLifePushConstants {
    pub fn new(draw_start: Vec2, draw_end: Vec2, draw_radius: f32) -> Self {
        Self {
            draw_radius,
            draw_end: draw_end.to_array(),
            draw_start: draw_start.to_array(),
        }
    }
}
