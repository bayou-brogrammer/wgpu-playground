use winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct DiamondContext {
    window: Window,
    queue: wgpu::Queue,
    device: wgpu::Device,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

impl DiamondContext {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

pub async fn run<A: DiamondApp + 'static>(mut app: A) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let body = doc.body().unwrap();
                let canvas = web_sys::Element::from(window.canvas());
                body.append_child(&canvas).unwrap();
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut ctx = DiamondContext::new(window).await;

    app.start(&event_loop, &mut ctx);

    let mut request_window_close = false;
    event_loop.run(move |event, event_loop, control_flow| {
        control_flow.set_poll();

        // Run input fn
        app.input(&mut ctx, event_loop, &event);

        match event {
            Event::WindowEvent { ref event, .. } => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        ctx.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so w have to dereference it twice
                        ctx.resize(**new_inner_size);
                    }
                    WindowEvent::KeyboardInput {
                        input,
                        is_synthetic,
                        ..
                    } => {
                        if let Some(key) = input.virtual_keycode {
                            if !is_synthetic
                                && key == VirtualKeyCode::Escape
                                && input.state == ElementState::Pressed
                            {
                                request_window_close = true;
                            }
                        }
                    }
                    WindowEvent::CloseRequested => {
                        request_window_close = true;
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                // Close window(s)
                if request_window_close {
                    control_flow.set_exit();

                    // Run end
                    app.end(&mut ctx);
                }
            }
            Event::RedrawRequested(window_id) if window_id == ctx.window().id() => {
                app.update(&mut ctx);

                match ctx.surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            ctx.device
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("Render Commands"),
                                });

                        // Run render & post processing functions
                        app.render(
                            &ctx,
                            RenderData {
                                frame: &frame,
                                encoder: &mut encoder,
                            },
                        );

                        app.post_processing(
                            &ctx,
                            RenderData {
                                frame: &frame,
                                encoder: &mut encoder,
                            },
                        );

                        ctx.queue.submit(Some(encoder.finish()));

                        frame.present();

                        app.after_render(&ctx);
                    }
                    Err(error) => {
                        if error == wgpu::SurfaceError::OutOfMemory {
                            panic!("Swapchain error: {error}. Rendering cannot continue.")
                        }
                    }
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                ctx.window().request_redraw();
            }
            _ => {}
        }
    });
}

/// A trait to define all stages of your Glass app. Each function here is run at a specific stage
/// within winit event loop. When you impl this for your app, think of this as the
/// table of contents of your app flow.
pub trait DiamondApp {
    /// Run at start
    fn start(&mut self, _event_loop: &EventLoop<()>, _context: &mut DiamondContext) {}

    /// Run on each event received from winit
    fn input(
        &mut self,
        _context: &mut DiamondContext,
        _event_loop: &EventLoopWindowTarget<()>,
        _event: &Event<()>,
    ) {
    }

    /// Run each frame
    fn update(&mut self, _context: &mut DiamondContext) {}

    /// Run each frame for each window after update
    fn render(&mut self, _context: &DiamondContext, _render_data: RenderData) {
        let RenderData { encoder, frame, .. } = _render_data;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let _r = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }
    }
    /// Run each frame for each window after rendering per window
    fn post_processing(&mut self, _context: &DiamondContext, _render_data: RenderData) {}

    /// Run each frame for each window after post processing
    fn after_render(&mut self, _context: &DiamondContext) {}

    /// Run each frame last
    fn end_of_frame(&mut self, _context: &mut DiamondContext) {}

    /// Run at exit
    fn end(&mut self, _context: &mut DiamondContext) {}
}

/// All necessary data required to render with wgpu. This data only lives for the duration of
/// rendering.
/// The command queue will be submitted each frame.
pub struct RenderData<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub frame: &'a wgpu::SurfaceTexture,
}

#[derive(Debug)]
pub enum DiamondError {
    AdapterError,
    ImageError(image::ImageError),
    WindowError(winit::error::OsError),
    DeviceError(wgpu::RequestDeviceError),
    SurfaceError(wgpu::CreateSurfaceError),
}

impl std::fmt::Display for DiamondError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DiamondError::WindowError(e) => format!("WindowError: {}", e),
            DiamondError::SurfaceError(e) => format!("SurfaceError: {}", e),
            DiamondError::AdapterError => "AdapterError".to_owned(),
            DiamondError::DeviceError(e) => format!("DeviceError: {}", e),
            DiamondError::ImageError(e) => format!("ImageError: {}", e),
        };
        write!(f, "{}", s)
    }
}
