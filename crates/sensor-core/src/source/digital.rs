use serde::Deserialize;

use crate::{DigitalInputSource, SensorResult};

/// 数字输入源工厂 trait
/// 每种实现提供自己的工厂
pub trait DigitalInputSourceFactory: Send + Sync {
    /// 从配置创建数字输入源
    fn create(&self, config: &DigitalInputConfig) -> SensorResult<Box<dyn DigitalInputSource>>;

    /// 工厂名称
    fn name(&self) -> &str;

    /// 配置验证
    fn validate_config(&self, _config: &DigitalInputConfig) -> SensorResult<()> {
        Ok(())
    }
}

/// 数字输入配置
#[derive(Debug, Clone, Deserialize)]
pub struct DigitalInputConfig {
    /// 源类型: "simulator" | "gpio" | "spi" | "modbus"
    #[serde(default = "default_source_type")]
    pub source_type: String,

    /// 切换间隔（秒），模拟器使用
    #[serde(default = "default_toggle_interval")]
    pub toggle_interval_secs: u64,

    /// GPIO 配置
    #[serde(default)]
    pub gpio: Option<GpioConfig>,

    /// SPI 配置
    #[serde(default)]
    pub spi: Option<SpiConfig>,

    /// Modbus 配置
    #[serde(default)]
    pub modbus: Option<ModbusDigitalConfig>,
}

fn default_source_type() -> String {
    "simulator".to_string()
}
fn default_toggle_interval() -> u64 {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub struct GpioConfig {
    pub pin_0: u8,
    pub pin_1: u8,
    #[serde(default)]
    pub active_low: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpiConfig {
    pub device: String,
    pub chip_select: u8,
    pub speed_hz: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModbusDigitalConfig {
    pub slave_id: u8,
    pub register_address: u16,
    pub bit_0: u8,
    pub bit_1: u8,
}

impl Default for DigitalInputConfig {
    fn default() -> Self {
        Self {
            source_type: default_source_type(),
            toggle_interval_secs: default_toggle_interval(),
            gpio: None,
            spi: None,
            modbus: None,
        }
    }
}
