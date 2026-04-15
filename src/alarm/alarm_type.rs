// 报警类型定义

use serde::{Deserialize, Serialize};

/// 报警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlarmLevel {
    /// 预警（90-100%）
    Warning,
    /// 危险（>100%）
    Danger,
    /// 严重（系统级错误）
    Critical,
}

impl AlarmLevel {
    pub fn as_str(&self) -> &str {
        match self {
            AlarmLevel::Warning => "warning",
            AlarmLevel::Danger => "danger",
            AlarmLevel::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "warning" => Some(AlarmLevel::Warning),
            "danger" => Some(AlarmLevel::Danger),
            "critical" => Some(AlarmLevel::Critical),
            _ => None,
        }
    }

    /// 获取优先级（数值越大优先级越高）
    pub fn priority(&self) -> u8 {
        match self {
            AlarmLevel::Warning => 1,
            AlarmLevel::Danger => 2,
            AlarmLevel::Critical => 3,
        }
    }
}

/// 报警来源（报警类型）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlarmSource {
    /// 力矩报警
    Moment,
    /// 角度报警
    Angle,
    /// 主钩勾头开关报警
    MainHookSwitch,
    /// 副钩勾头开关报警
    AuxHookSwitch,
    /// 载荷超限报警
    LoadOverload,
    /// 传感器故障报警
    SensorFault,
    /// 系统错误报警
    SystemError,
}

impl AlarmSource {
    pub fn as_str(&self) -> &str {
        match self {
            AlarmSource::Moment => "moment",
            AlarmSource::Angle => "angle",
            AlarmSource::MainHookSwitch => "main_hook_switch",
            AlarmSource::AuxHookSwitch => "aux_hook_switch",
            AlarmSource::LoadOverload => "load_overload",
            AlarmSource::SensorFault => "sensor_fault",
            AlarmSource::SystemError => "system_error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "moment" => Some(AlarmSource::Moment),
            "angle" => Some(AlarmSource::Angle),
            "main_hook_switch" => Some(AlarmSource::MainHookSwitch),
            "aux_hook_switch" => Some(AlarmSource::AuxHookSwitch),
            "load_overload" => Some(AlarmSource::LoadOverload),
            "sensor_fault" => Some(AlarmSource::SensorFault),
            "system_error" => Some(AlarmSource::SystemError),
            _ => None,
        }
    }

    /// 获取报警描述
    pub fn description(&self) -> &str {
        match self {
            AlarmSource::Moment => "力矩报警",
            AlarmSource::Angle => "角度报警",
            AlarmSource::MainHookSwitch => "主钩勾头开关报警",
            AlarmSource::AuxHookSwitch => "副钩勾头开关报警",
            AlarmSource::LoadOverload => "载荷超限报警",
            AlarmSource::SensorFault => "传感器故障报警",
            AlarmSource::SystemError => "系统错误报警",
        }
    }
}

/// 报警类型（组合报警来源和级别）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlarmType {
    /// 报警来源
    pub source: AlarmSource,
    /// 报警级别
    pub level: AlarmLevel,
}

impl AlarmType {
    pub fn new(source: AlarmSource, level: AlarmLevel) -> Self {
        Self { source, level }
    }

    /// 获取报警唯一标识
    pub fn id(&self) -> String {
        format!("{}_{}", self.source.as_str(), self.level.as_str())
    }

    /// 获取报警描述
    pub fn description(&self) -> String {
        format!("{} - {}", self.source.description(), self.level.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alarm_level_priority() {
        assert!(AlarmLevel::Critical.priority() > AlarmLevel::Danger.priority());
        assert!(AlarmLevel::Danger.priority() > AlarmLevel::Warning.priority());
    }

    #[test]
    fn test_alarm_type_id() {
        let alarm = AlarmType::new(AlarmSource::Moment, AlarmLevel::Danger);
        assert_eq!(alarm.id(), "moment_danger");
    }
}
