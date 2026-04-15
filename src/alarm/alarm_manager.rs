// 报警管理器（责任链模式 + 防抖机制）

use super::alarm_checker::{
    AlarmCheckResult, AlarmChecker, AngleAlarmChecker, LoadOverloadChecker, MomentAlarmChecker,
};
use super::alarm_config::AlarmConfig;
use super::alarm_type::{AlarmSource, AlarmType};
use crate::models::ProcessedData;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 报警状态
#[derive(Debug, Clone)]
struct AlarmState {
    /// 当前是否处于报警状态
    active: bool,
    /// 连续触发计数
    trigger_count: u32,
    /// 连续清除计数
    clear_count: u32,
    /// 最后一次报警信息
    last_message: String,
}

impl Default for AlarmState {
    fn default() -> Self {
        Self {
            active: false,
            trigger_count: 0,
            clear_count: 0,
            last_message: String::new(),
        }
    }
}

/// 报警管理器
pub struct AlarmManager {
    /// 报警检查器列表
    checkers: Vec<Box<dyn AlarmChecker>>,

    /// 报警配置
    config: Arc<RwLock<AlarmConfig>>,

    /// 报警状态（按报警来源分类）
    alarm_states: Arc<RwLock<HashMap<AlarmSource, AlarmState>>>,
}

impl AlarmManager {
    /// 创建报警管理器
    pub fn new(config: AlarmConfig) -> Self {
        let mut checkers: Vec<Box<dyn AlarmChecker>> = Vec::new();

        // 添加力矩报警检查器
        if config.is_alarm_enabled(AlarmSource::Moment) {
            checkers.push(Box::new(MomentAlarmChecker::new(
                config.moment.warning_threshold,
                config.moment.danger_threshold,
            )));
        }

        // 添加角度报警检查器
        if config.is_alarm_enabled(AlarmSource::Angle) {
            checkers.push(Box::new(AngleAlarmChecker::new(
                config.angle.min_angle,
                config.angle.max_angle,
            )));
        }

        // 添加载荷超限报警检查器
        if config.is_alarm_enabled(AlarmSource::LoadOverload) {
            checkers.push(Box::new(LoadOverloadChecker::new(
                config.load_overload.max_load,
            )));
        }

        Self {
            checkers,
            config: Arc::new(RwLock::new(config)),
            alarm_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检查所有报警
    pub fn check_alarms(&self, data: &ProcessedData) -> Vec<AlarmCheckResult> {
        let mut results = Vec::new();

        for checker in &self.checkers {
            if !checker.is_enabled() {
                continue;
            }

            let result = checker.check(data);

            // 应用防抖机制
            if let Some(debounced_result) = self.apply_debounce(checker.source(), result) {
                results.push(debounced_result);
            }
        }

        results
    }

    /// 应用防抖机制
    fn apply_debounce(
        &self,
        source: AlarmSource,
        result: AlarmCheckResult,
    ) -> Option<AlarmCheckResult> {
        let config = self.config.read().unwrap();

        // 如果防抖未启用，直接返回结果
        if !config.debounce.enabled {
            return Some(result);
        }

        let mut states = self.alarm_states.write().unwrap();
        let state = states.entry(source).or_insert_with(AlarmState::default);

        if result.triggered {
            // 报警触发
            state.clear_count = 0;
            state.trigger_count += 1;

            // 检查是否达到触发阈值
            if state.trigger_count >= config.debounce.trigger_count {
                // 检查是否是新报警（状态转换）
                if !state.active {
                    state.active = true;
                    state.last_message = result.message.clone();
                    tracing::warn!(
                        "⚠️  新报警触发: {} (连续 {} 次)",
                        result.message,
                        state.trigger_count
                    );
                    return Some(result);
                } else {
                    // 持续报警，跳过
                    tracing::debug!("持续报警: {} (已触发)", source.as_str());
                    return None;
                }
            } else {
                // 未达到触发阈值，跳过
                tracing::debug!(
                    "报警防抖中: {} ({}/{})",
                    source.as_str(),
                    state.trigger_count,
                    config.debounce.trigger_count
                );
                return None;
            }
        } else {
            // 报警解除
            state.trigger_count = 0;
            state.clear_count += 1;

            // 检查是否达到解除阈值
            if state.clear_count >= config.debounce.clear_count {
                if state.active {
                    state.active = false;
                    tracing::info!(
                        "✅ 报警解除: {} (连续 {} 次安全)",
                        source.as_str(),
                        state.clear_count
                    );
                }
                state.clear_count = 0;
            }

            return None;
        }
    }

    /// 重置报警状态
    pub fn reset_alarm(&self, source: AlarmSource) {
        let mut states = self.alarm_states.write().unwrap();
        if let Some(state) = states.get_mut(&source) {
            state.active = false;
            state.trigger_count = 0;
            state.clear_count = 0;
            tracing::info!("报警状态已重置: {}", source.as_str());
        }
    }

    /// 重置所有报警状态
    pub fn reset_all_alarms(&self) {
        let mut states = self.alarm_states.write().unwrap();
        states.clear();
        tracing::info!("所有报警状态已重置");
    }

    /// 获取当前活跃的报警
    pub fn get_active_alarms(&self) -> Vec<AlarmSource> {
        let states = self.alarm_states.read().unwrap();
        states
            .iter()
            .filter(|(_, state)| state.active)
            .map(|(source, _)| *source)
            .collect()
    }

    /// 更新配置
    pub fn update_config(&mut self, new_config: AlarmConfig) {
        *self.config.write().unwrap() = new_config.clone();

        // 重新创建检查器
        self.checkers.clear();

        if new_config.is_alarm_enabled(AlarmSource::Moment) {
            self.checkers.push(Box::new(MomentAlarmChecker::new(
                new_config.moment.warning_threshold,
                new_config.moment.danger_threshold,
            )));
        }

        if new_config.is_alarm_enabled(AlarmSource::Angle) {
            self.checkers.push(Box::new(AngleAlarmChecker::new(
                new_config.angle.min_angle,
                new_config.angle.max_angle,
            )));
        }

        if new_config.is_alarm_enabled(AlarmSource::LoadOverload) {
            self.checkers.push(Box::new(LoadOverloadChecker::new(
                new_config.load_overload.max_load,
            )));
        }

        tracing::info!("报警配置已更新");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensor_core::SensorData;

    #[test]
    fn test_alarm_manager_check() {
        // 禁用防抖以便测试单次检查
        let mut config = AlarmConfig::default();
        config.debounce.enabled = false;
        let manager = AlarmManager::new(config);

        let sensor_data = SensorData::new(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);

        let results = manager.check_alarms(&processed);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_alarm_manager_debounce() {
        let mut config = AlarmConfig::default();
        config.debounce.trigger_count = 3;
        let manager = AlarmManager::new(config);

        let sensor_data = SensorData::new(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);

        // 第一次检查，未达到阈值
        let results = manager.check_alarms(&processed);
        assert!(results.is_empty());

        // 第二次检查，未达到阈值
        let results = manager.check_alarms(&processed);
        assert!(results.is_empty());

        // 第三次检查，达到阈值，触发报警
        let results = manager.check_alarms(&processed);
        assert!(!results.is_empty());
    }
}
