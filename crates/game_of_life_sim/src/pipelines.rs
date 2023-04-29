use glass::{
    wgpu::{self, StorageTextureAccess},
    GlassContext,
};

use crate::{shaders::ShaderImportProcessor, GameOfLifePushConstants};

pub struct Pipelines {
    pub draw_pipeline: wgpu::ComputePipeline,
    pub init_pipeline: wgpu::ComputePipeline,
    pub game_of_life_pipeline: wgpu::ComputePipeline,
}

impl Pipelines {
    fn create_draw_pipeline(context: &mut GlassContext) -> wgpu::ComputePipeline {
        let dr_layout =
            context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            format: wgpu::TextureFormat::Rgba16Float,
                        },
                        count: None,
                    }],
                    label: Some("draw_bind_group_layout"),
                });

        let brush_shader = ShaderImportProcessor::default()
            .load_shader(context.device(), "draw.wgsl", Some("draw_shader"))
            .unwrap();

        let draw_layout =
            context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Draw Layout"),
                    bind_group_layouts: &[&dr_layout],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::COMPUTE,
                        range: 0..std::mem::size_of::<GameOfLifePushConstants>() as u32,
                    }],
                });

        context
            .device()
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Draw Pipeline"),
                layout: Some(&draw_layout),
                module: &brush_shader,
                entry_point: "main",
            })
    }

    fn create_init_pipeline(
        context: &mut GlassContext,
        bg_layout: &wgpu::BindGroupLayout,
        game_of_life_shader: &wgpu::ShaderModule,
    ) -> wgpu::ComputePipeline {
        let game_of_life_init_layout =
            context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    push_constant_ranges: &[],
                    label: Some("Game of Life Init Layout"),
                    bind_group_layouts: &[bg_layout],
                });

        context
            .device()
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Init Pipeline"),
                layout: Some(&game_of_life_init_layout),
                module: game_of_life_shader,
                entry_point: "init",
            })
    }

    fn create_compute_pipeline(
        context: &mut GlassContext,
        bg_layout: &wgpu::BindGroupLayout,
        game_of_life_shader: &wgpu::ShaderModule,
    ) -> wgpu::ComputePipeline {
        let game_of_life_layout =
            context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    push_constant_ranges: &[],
                    label: Some("Game of Life Layout"),
                    bind_group_layouts: &[bg_layout],
                });

        context
            .device()
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Update Pipeline"),
                layout: Some(&game_of_life_layout),
                module: game_of_life_shader,
                entry_point: "update",
            })
    }

    pub fn load(context: &mut GlassContext) -> Self {
        let bg_layout =
            context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            count: None,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                format: wgpu::TextureFormat::Rgba16Float,
                            },
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            count: None,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                format: wgpu::TextureFormat::Rgba16Float,
                            },
                        },
                    ],
                    label: Some("gol_bind_group_layout"),
                });

        let game_of_life_shader = ShaderImportProcessor::default()
            .load_shader_with_dsl(
                context.device(),
                "game_of_life.wgsl",
                &crate::dsl::rulesets::conways_game_of_life(),
                Some("game_of_life_shader"),
            )
            .unwrap();

        let draw_pipeline = Self::create_draw_pipeline(context);
        let init_pipeline = Self::create_init_pipeline(context, &bg_layout, &game_of_life_shader);
        let game_of_life_pipeline =
            Self::create_compute_pipeline(context, &bg_layout, &game_of_life_shader);

        Self {
            init_pipeline,
            draw_pipeline,
            game_of_life_pipeline,
        }
    }
}
