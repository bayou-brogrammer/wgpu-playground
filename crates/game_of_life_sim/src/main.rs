use game_of_life_sim::SIM_SIZE;
use glass::{device_context::DeviceConfig, wgpu, window::WindowConfig, Glass, GlassConfig};

fn config() -> GlassConfig {
    GlassConfig {
        device_config: DeviceConfig {
            power_preference: wgpu::PowerPreference::HighPerformance,
            features: wgpu::Features::PUSH_CONSTANTS
                | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits {
                    max_compute_invocations_per_workgroup: 1024,
                    ..wgpu::Limits::default()
                }
            },
            backends: wgpu::Backends::all(),
        },
        window_configs: vec![WindowConfig {
            width: SIM_SIZE,
            height: SIM_SIZE,
            exit_on_esc: true,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            ..WindowConfig::default()
        }],
    }
}

fn main() -> std::result::Result<(), glass::GlassError> {
    Glass::new(game_of_life_sim::GameOfLifeApp::default(), config()).run()
}
