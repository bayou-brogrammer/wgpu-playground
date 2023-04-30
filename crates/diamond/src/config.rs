use wgpu::PowerPreference;

use crate::{device_context::DeviceConfig, window::WindowConfig};

/// Configuration of your windows and devices.
#[derive(Debug, Clone)]
pub struct DiamondConfig {
    pub device_config: DeviceConfig,
    pub window_config: WindowConfig,
}

impl DiamondConfig {
    pub fn performance(width: u32, height: u32) -> Self {
        Self {
            device_config: DeviceConfig {
                power_preference: PowerPreference::HighPerformance,
                ..Default::default()
            },
            window_config: WindowConfig {
                width,
                height,
                exit_on_esc: false,
                ..WindowConfig::default()
            },
        }
    }
}

impl Default for DiamondConfig {
    fn default() -> Self {
        Self {
            device_config: DeviceConfig::default(),
            window_config: WindowConfig::default(),
        }
    }
}
