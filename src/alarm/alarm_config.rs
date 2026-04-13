// 报警配置

use super::alarm_type::AlarmSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 报警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmConfig {
    /// 力矩报警配置
    pub moment: MomentAlarmConfig,

    /// 角度报警配置
    #[serde(default)]
    pub angle: AngleAlarmConfig,

    /// 载荷超限报警配置
    #[serde(default)]
    pub load_overload: LoadOverloadConfig,

    /// 防抖配置
    #[serde(default)]
    pub debounce: DebounceConfig,

    /// 报警启用状态
    #[serde(default)]
    pub enabled_alarms: HashMap<String, bool>,
}

impl Default for AlarmConfig {
    fn default() -> Self {
        let mut enabled_alarms = HashMap::new();
        enabled_alarms.insert(AlarmSource::Moment.as_str().to_string(), true);
        enabled_alarms.insert(AlarmSource::Angle.as_str().to_string(), true);
        enabled_alarms.insert(AlarmSource::LoadOverload.as_str().to_string(), false);

        Self {
            moment: MomentAlarmConfig::default(),
            angle: AngleAlarmConfig::default(),
            load_overload: LoadOverloadConfig::default(),
            debounce: DebounceConfig::default(),
            enabled_alarms,
        }
    }
}

impl AlarmConfig {
    /// 检查报警是否启用
    pub fn is_alarm_enabled(&self, source: AlarmSource) -> bool {
        self.enabled_alarms
            .get(source.as_str())
            .copied()
            .unwrap_or(false)
    }

    /// 设置报警启用状态
    pub fn set_alarm_enabled(&mut self, source: AlarmSource, enabled: bool) {
        self.enabled_alarms
            .insert(source.as_str().to_string(), enabled);
    }
}

/// 力矩报警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentAlarmConfig {
    /// 预警阈值（%）
    pub warning_threshold: f64,
    /// 危险阈值（%）
    pub danger_threshold: f64,
}

impl Default for MomentAlarmConfig {
    fn default() -> Self {
        Self {
            warning_threshold: 90.0,
            danger_threshold: 100.0,
        }
    }
}

/// 角度报警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngleAlarmConfig {
    /// 最小角度（度）
    pub min_angle: f64,
    /// 最大角度（度）
    pub max_angle: f64,
}

impl Default for AngleAlarmConfig {
    fn default() -> Self {
        Self {
            min_angle: 0.0,
            max_angle: 85.0,
        }
    }
}

/// 载荷超限报警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadOverloadConfig {
    /// 最大载荷（吨）
    pub max_load: f64,
}

impl Default for LoadOverloadConfig {
    fn default() -> Self {
        Self { max_load: 50.0 }
    }
}

/// 防抖配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebounceConfig {
    /// 报警触发防抖次数（连续多少次才触发）
    pub trigger_count: u32,
    /// 报警解除防抖次数（连续多少次才解除）
    pub clear_count: u32,
    /// 是否启用防抖
    pub enabled: bool,
}

impl Default for DebounceConfig {
    fn default() -> Self {
        Self {
            trigger_count: 5, // 连续 5 次（500ms）
            clear_count: 10,  // 连续 10 次（1s）
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alarm_config_default() {
        let config = AlarmConfig::default();
        assert!(config.is_alarm_enabled(AlarmSource::Moment));
        assert!(config.is_alarm_enabled(AlarmSource::Angle));
    }

    #[test]
    fn test_set_alarm_enabled() {
        let mut config = AlarmConfig::default();
        config.set_alarm_enabled(AlarmSource::Angle, true);
        assert!(config.is_alarm_enabled(AlarmSource::Angle));
    }
}
