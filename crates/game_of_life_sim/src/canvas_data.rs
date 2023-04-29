use glass::{
    pipelines::QuadPipeline,
    texture::Texture,
    wgpu::{self, BindGroupDescriptor, ComputePipeline},
    GlassContext,
};

use crate::SIM_SIZE;

pub struct CanvasData {
    pub canvas: Texture,
    pub data_in: Texture,

    pub draw_bind_group: wgpu::BindGroup,
    pub init_bind_group: wgpu::BindGroup,
    pub canvas_bind_group: wgpu::BindGroup,
}

impl CanvasData {
    fn create_texture(context: &GlassContext, width: u32, height: u32, label: &str) -> Texture {
        Texture::empty(
            context.device(),
            label,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            1,
            wgpu::TextureFormat::Rgba16Float,
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        )
    }

    pub fn create(
        context: &GlassContext,
        quad_pipeline: &QuadPipeline,
        init_pipeline: &ComputePipeline,
        draw_pipeline: &ComputePipeline,
    ) -> Self {
        let device = context.device();
        let canvas = Self::create_texture(context, SIM_SIZE, SIM_SIZE, "Canvas");
        let data_in = Self::create_texture(context, SIM_SIZE, SIM_SIZE, "Data In");

        // Create bind groups to match pipeline layouts (except update, create that dynamically each frame)
        let canvas_bind_group =
            quad_pipeline.create_bind_group(device, &canvas.views[0], &canvas.sampler);

        // These must match the bind group layout of our pipeline
        let init_bind_group_layout = init_pipeline.get_bind_group_layout(0);
        let init_bind_group = context.device().create_bind_group(&BindGroupDescriptor {
            label: Some("Init Bind Group"),
            layout: &init_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&canvas.views[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&data_in.views[0]),
                },
            ],
        });

        let draw_bing_group_layout = draw_pipeline.get_bind_group_layout(0);
        let draw_bind_group = context.device().create_bind_group(&BindGroupDescriptor {
            label: Some("Draw Bind Group"),
            layout: &draw_bing_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&data_in.views[0]),
            }],
        });

        CanvasData {
            canvas,
            data_in,

            init_bind_group,
            draw_bind_group,
            canvas_bind_group,
        }
    }
}
