// 报警防抖器 - 从 StoragePipeline 提取的报警状态机
use crate::models::ProcessedData;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(test)]
use crate::alarm::alarm_type::AlarmSource;

/// 报警动作 - 防抖后的决策结果
#[derive(Debug, Clone)]
pub enum AlarmAction {
    /// 触发新报警（从安全 → 危险转换）
    TriggerAlarm(ProcessedData),
    /// 解除报警（从危险 → 安全转换）
    ClearAlarm,
    /// 无变化（继续危险或继续安全）
    None,
}

impl PartialEq for AlarmAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AlarmAction::ClearAlarm, AlarmAction::ClearAlarm) => true,
            (AlarmAction::None, AlarmAction::None) => true,
            (AlarmAction::TriggerAlarm(a), AlarmAction::TriggerAlarm(b)) => {
                a.sequence_number == b.sequence_number
                    && a.is_danger == b.is_danger
                    && a.alarm_sources == b.alarm_sources
            }
            _ => false,
        }
    }
}

/// 报警防抖配置
#[derive(Debug, Clone)]
pub struct AlarmDebounceConfig {
    /// 连续多少次危险才触发报警（0 = 禁用防抖，立即触发）
    pub alarm_debounce_count: u32,
    /// 连续多少次安全才解除报警（0 = 禁用防抖，立即解除）
    pub alarm_clear_debounce_count: u32,
}

impl Default for AlarmDebounceConfig {
    fn default() -> Self {
        Self {
            alarm_debounce_count: 5,
            alarm_clear_debounce_count: 10,
        }
    }
}

/// 报警防抖器 - 有状态的状态机
pub struct AlarmDebouncer {
    danger_count: AtomicU32,
    safe_count: AtomicU32,
    last_was_danger: AtomicBool,
    config: AlarmDebounceConfig,
}

impl AlarmDebouncer {
    pub fn new(config: AlarmDebounceConfig) -> Self {
        Self {
            danger_count: AtomicU32::new(0),
            safe_count: AtomicU32::new(0),
            last_was_danger: AtomicBool::new(false),
            config,
        }
    }

    pub fn process(&self, data: &ProcessedData) -> AlarmAction {
        let has_any_alarm = data.is_danger || !data.alarm_sources.is_empty();

        if has_any_alarm {
            let current_danger_count = self.danger_count.fetch_add(1, Ordering::Relaxed) + 1;
            self.safe_count.store(0, Ordering::Relaxed);

            if self.config.alarm_debounce_count == 0
                || current_danger_count >= self.config.alarm_debounce_count
            {
                let expected = false;
                if self
                    .last_was_danger
                    .compare_exchange(expected, true, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    for source in &data.alarm_sources {
                        tracing::warn!("Alarm triggered: {:?}", source);
                    }
                    tracing::warn!(
                        "⚠️  NEW ALARM triggered at sequence {} (danger_count: {}, threshold: {}, is_danger: {}, alarm_sources: {:?})",
                        data.sequence_number,
                        current_danger_count,
                        self.config.alarm_debounce_count,
                        data.is_danger,
                        data.alarm_sources
                    );
                    return AlarmAction::TriggerAlarm(data.clone());
                }
            }
            AlarmAction::None
        } else {
            let current_safe_count = self.safe_count.fetch_add(1, Ordering::Relaxed) + 1;
            self.danger_count.store(0, Ordering::Relaxed);

            if self.config.alarm_clear_debounce_count == 0
                || current_safe_count >= self.config.alarm_clear_debounce_count
            {
                let expected = true;
                if self
                    .last_was_danger
                    .compare_exchange(expected, false, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    tracing::info!(
                        "✅ Alarm CLEARED at sequence {} (safe_count: {}, threshold: {})",
                        data.sequence_number,
                        current_safe_count,
                        self.config.alarm_clear_debounce_count
                    );
                    return AlarmAction::ClearAlarm;
                }
            }
            AlarmAction::None
        }
    }

    pub fn reset(&self) {
        self.danger_count.store(0, Ordering::Relaxed);
        self.safe_count.store(0, Ordering::Relaxed);
        self.last_was_danger.store(false, Ordering::Relaxed);
    }

    pub fn notify_danger_cleared(&self) {
        self.last_was_danger.store(false, Ordering::Relaxed);
    }

    pub fn clone_state(&self) -> AlarmDebouncerState {
        AlarmDebouncerState {
            danger_count: self.danger_count.load(Ordering::Relaxed),
            safe_count: self.safe_count.load(Ordering::Relaxed),
            last_was_danger: self.last_was_danger.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlarmDebouncerState {
    pub danger_count: u32,
    pub safe_count: u32,
    pub last_was_danger: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn make_data(sequence: u64, is_danger: bool, alarm_sources: Vec<AlarmSource>) -> ProcessedData {
        ProcessedData {
            current_load: 10.0,
            rated_load: 25.0,
            working_radius: 5.0,
            boom_angle: 45.0,
            boom_length: 10.0,
            moment_percentage: 50.0,
            is_warning: is_danger,
            is_danger,
            validation_error: None,
            timestamp: SystemTime::now(),
            sequence_number: sequence,
            alarm_sources,
            alarm_messages: Vec::new(),
        }
    }

    #[test]
    fn test_debouncer_triggers_alarm_after_threshold() {
        let config = AlarmDebounceConfig {
            alarm_debounce_count: 3,
            alarm_clear_debounce_count: 5,
        };
        let debouncer = AlarmDebouncer::new(config);

        assert_eq!(
            debouncer.process(&make_data(1, true, vec![AlarmSource::Moment])),
            AlarmAction::None
        );
        assert_eq!(
            debouncer.process(&make_data(2, true, vec![AlarmSource::Moment])),
            AlarmAction::None
        );

        let action = debouncer.process(&make_data(3, true, vec![AlarmSource::Moment]));
        assert!(matches!(action, AlarmAction::TriggerAlarm(_)));

        assert_eq!(
            debouncer.process(&make_data(4, true, vec![AlarmSource::Moment])),
            AlarmAction::None
        );
    }

    #[test]
    fn test_debouncer_clears_alarm_after_threshold() {
        let config = AlarmDebounceConfig {
            alarm_debounce_count: 1,
            alarm_clear_debounce_count: 2,
        };
        let debouncer = AlarmDebouncer::new(config);

        debouncer.process(&make_data(1, true, vec![AlarmSource::Moment]));
        assert_eq!(
            debouncer.process(&make_data(2, false, vec![])),
            AlarmAction::None
        );

        let action = debouncer.process(&make_data(3, false, vec![]));
        assert_eq!(action, AlarmAction::ClearAlarm);
    }

    #[test]
    fn test_debouncer_zero_threshold_immediate() {
        let config = AlarmDebounceConfig {
            alarm_debounce_count: 0,
            alarm_clear_debounce_count: 0,
        };
        let debouncer = AlarmDebouncer::new(config);

        let action = debouncer.process(&make_data(1, true, vec![AlarmSource::Moment]));
        assert!(matches!(action, AlarmAction::TriggerAlarm(_)));

        let action = debouncer.process(&make_data(2, false, vec![]));
        assert_eq!(action, AlarmAction::ClearAlarm);
    }

    #[test]
    fn test_debouncer_reset() {
        let config = AlarmDebounceConfig::default();
        let debouncer = AlarmDebouncer::new(config);

        debouncer.process(&make_data(1, true, vec![AlarmSource::Moment]));
        debouncer.reset();

        let state = debouncer.clone_state();
        assert_eq!(state.danger_count, 0);
        assert_eq!(state.safe_count, 0);
        assert!(!state.last_was_danger);
    }

    #[test]
    fn test_notify_danger_cleared_compatible() {
        let config = AlarmDebounceConfig {
            alarm_debounce_count: 1,
            alarm_clear_debounce_count: 1,
        };
        let debouncer = AlarmDebouncer::new(config);

        debouncer.process(&make_data(1, true, vec![AlarmSource::Moment]));
        debouncer.notify_danger_cleared();

        let action = debouncer.process(&make_data(2, true, vec![AlarmSource::Moment]));
        assert!(matches!(action, AlarmAction::TriggerAlarm(_)));
    }
}
