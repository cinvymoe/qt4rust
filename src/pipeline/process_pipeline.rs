// 计算管道 - 多速率数据流架构
// 从滤波层获取数据 -> 计算处理 -> 发送给显示/存储层

use crate::alarm::alarm_type::AlarmSource;
use crate::alarm::{AlarmConfig, AlarmManager};
use crate::models::crane_config::CraneConfig;
use crate::models::rated_load_table::RatedLoadTable;
use crate::models::ProcessedData;
use crate::pipeline::event_channel::StorageEventSender;
use crate::pipeline::filter_buffer::FilterBuffer;
use crate::pipeline::shared_buffer::SharedBuffer;
use sensor_core::{AlarmThresholds, SensorCalibration};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use std::time::Duration;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct ProcessPipelineConfig {
    pub interval: Duration,
}

impl Default for ProcessPipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),
        }
    }
}

pub struct ProcessPipeline {
    config: ProcessPipelineConfig,
    filter_buffer: Arc<Mutex<FilterBuffer>>,
    display_buffer: SharedBuffer,
    crane_config: Arc<CraneConfig>,
    // 热重载配置引用（优先使用）
    sensor_calibration: Option<Arc<RwLock<SensorCalibration>>>,
    rated_load_table: Option<Arc<RwLock<RatedLoadTable>>>,
    alarm_thresholds: Option<Arc<RwLock<AlarmThresholds>>>,
    storage_event_sender: Option<StorageEventSender>,
    running: Arc<AtomicBool>,
    sequence_number: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
    alarm_manager: Option<Arc<RwLock<AlarmManager>>>,
}

impl ProcessPipeline {
    pub fn new(
        config: ProcessPipelineConfig,
        filter_buffer: Arc<Mutex<FilterBuffer>>,
        display_buffer: SharedBuffer,
        crane_config: Arc<CraneConfig>,
    ) -> Self {
        Self {
            config,
            filter_buffer,
            display_buffer,
            crane_config,
            sensor_calibration: None,
            rated_load_table: None,
            alarm_thresholds: None,
            storage_event_sender: None,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_manager: None,
        }
    }

    pub fn with_event_sender(
        config: ProcessPipelineConfig,
        filter_buffer: Arc<Mutex<FilterBuffer>>,
        display_buffer: SharedBuffer,
        crane_config: Arc<CraneConfig>,
        storage_event_sender: StorageEventSender,
    ) -> Self {
        Self {
            config,
            filter_buffer,
            display_buffer,
            crane_config,
            sensor_calibration: None,
            rated_load_table: None,
            alarm_thresholds: None,
            storage_event_sender: Some(storage_event_sender),
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_manager: None,
        }
    }

    /// 设置热重载配置引用（用于配置热重载）
    pub fn set_hot_reload_config(
        &mut self,
        sensor_calibration: Arc<RwLock<SensorCalibration>>,
        rated_load_table: Arc<RwLock<RatedLoadTable>>,
        alarm_thresholds: Arc<RwLock<AlarmThresholds>>,
    ) {
        self.sensor_calibration = Some(sensor_calibration.clone());
        self.rated_load_table = Some(rated_load_table.clone());
        self.alarm_thresholds = Some(alarm_thresholds.clone());

        // 打印当前配置值
        if let Ok(cal) = sensor_calibration.read() {
            tracing::info!("🔧 [ProcessPipeline] 热重载配置已设置");
            tracing::info!("📋 [初始标定参数] weight: zero_ad={:.2}, zero_value={:.2}, scale_ad={:.2}, scale_value={:.2}, multiplier={:.2}",
                cal.weight.zero_ad,
                cal.weight.zero_value,
                cal.weight.scale_ad,
                cal.weight.scale_value,
                cal.weight.multiplier);
        }

        if let Ok(thresholds) = alarm_thresholds.read() {
            tracing::info!(
                "⚠️  [ProcessPipeline] 预警阈值已设置: warning={}%, alarm={}%",
                thresholds.moment.warning_percentage,
                thresholds.moment.alarm_percentage
            );

            // 从 AlarmThresholds 创建 AlarmConfig
            let alarm_config = AlarmConfig {
                moment: crate::alarm::alarm_config::MomentAlarmConfig {
                    warning_threshold: thresholds.moment.warning_percentage,
                    danger_threshold: thresholds.moment.alarm_percentage,
                },
                angle: crate::alarm::alarm_config::AngleAlarmConfig {
                    min_angle: thresholds.angle.min_angle,
                    max_angle: thresholds.angle.max_angle,
                },
                load_overload: Default::default(),
                debounce: Default::default(),
                enabled_alarms: {
                    let mut map = HashMap::new();
                    map.insert("moment".to_string(), true);
                    map.insert("angle".to_string(), true);
                    map
                },
            };

            let alarm_manager = AlarmManager::new(alarm_config);
            self.alarm_manager = Some(Arc::new(RwLock::new(alarm_manager)));
            tracing::info!("🔔 [ProcessPipeline] AlarmManager 已初始化，角度报警已启用");
        }
    }

    pub fn set_initial_sequence(&mut self, sequence: u64) {
        self.sequence_number.store(sequence, Ordering::Relaxed);
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            return;
        }
        self.running.store(true, Ordering::Relaxed);

        let filter_buffer = Arc::clone(&self.filter_buffer);
        let display_buffer = Arc::clone(&self.display_buffer);
        let crane_config = Arc::clone(&self.crane_config);
        let sensor_calibration = self.sensor_calibration.clone();
        let rated_load_table = self.rated_load_table.clone();
        let alarm_thresholds = self.alarm_thresholds.clone();
        let alarm_manager = self.alarm_manager.clone();
        let storage_event_sender = self.storage_event_sender.clone();
        let sequence_number = Arc::clone(&self.sequence_number);
        let running = Arc::clone(&self.running);
        let interval = self.config.interval;

        self.handle = Some(qt_threading_utils::runtime::global_runtime().spawn(async move {
            let mut tick_interval = tokio::time::interval(interval);
            tracing::info!("🚀 [ProcessPipeline] 管道循环已启动，间隔: {:?}", interval);

            loop {
                tick_interval.tick().await;
                if !running.load(Ordering::Relaxed) {
                    break;
                }

                let sensor_data = {
                    let fb = filter_buffer.lock().unwrap();
                    let filtered = fb.get_filtered().clone();
                    if let Some(ref data) = filtered {
                        tracing::debug!("[ProcessPipeline] 从FilterBuffer读取: ad1={:.2}, ad2={:.2}, ad3={:.2}",
                            data.ad1_load, data.ad2_radius, data.ad3_angle);
                    }
                    filtered
                };

                if let Some(raw_data) = sensor_data {
                    let seq = sequence_number.fetch_add(1, Ordering::Relaxed);

                    // 如果有热重载配置，使用热重载配置；否则使用静态配置
                    let processed = if let (Some(cal), Some(table), Some(thresholds)) = (&sensor_calibration, &rated_load_table, &alarm_thresholds) {
                        // 使用热重载配置
                        let cal_guard = cal.read().unwrap();
                        let table_guard = table.read().unwrap();
                        let thresholds_guard = thresholds.read().unwrap();

                        tracing::info!("🔥 [ProcessPipeline] 使用热重载配置进行AD转换");
                        tracing::info!("📊 [标定参数] weight.zero_ad={:.2}, zero_value={:.2}, scale_ad={:.2}, scale_value={:.2}, multiplier={:.2}",
                            cal_guard.weight.zero_ad,
                            cal_guard.weight.zero_value,
                            cal_guard.weight.scale_ad,
                            cal_guard.weight.scale_value,
                            cal_guard.weight.multiplier);
                        tracing::info!("📊 [标定参数] angle.zero_ad={:.2}, zero_value={:.2}, scale_ad={:.2}, scale_value={:.2}",
                            cal_guard.angle.zero_ad,
                            cal_guard.angle.zero_value,
                            cal_guard.angle.scale_ad,
                            cal_guard.angle.scale_value);
                        tracing::info!("📊 [标定参数] radius.zero_ad={:.2}, zero_value={:.2}, scale_ad={:.2}, scale_value={:.2}",
                            cal_guard.radius.zero_ad,
                            cal_guard.radius.zero_value,
                            cal_guard.radius.scale_ad,
                            cal_guard.radius.scale_value);
                        tracing::info!("⚠️  [预警阈值] warning={}%, alarm={}%",
                            thresholds_guard.moment.warning_percentage,
                            thresholds_guard.moment.alarm_percentage);

                        // 创建临时CraneConfig
                        let hot_config = CraneConfig {
                            sensor_calibration: cal_guard.clone(),
                            rated_load_table: table_guard.clone(),
                            alarm_thresholds: thresholds_guard.clone(),
                        };

                        ProcessedData::from_sensor_data_with_config(
                            raw_data.clone(),
                            &hot_config,
                            seq,
                        )
                    } else {
                        // 使用静态配置（向后兼容）
                        tracing::warn!("⚠️  [ProcessPipeline] 热重载配置未设置，使用静态配置");
                        tracing::info!("📊 [静态配置] weight.scale_value={:.2}",
                            crane_config.sensor_calibration.weight.scale_value);

                        ProcessedData::from_sensor_data_with_config(
                            raw_data.clone(),
                            &crane_config,
                            seq,
                        )
                    };

                    tracing::info!("✅ [ProcessPipeline] AD转换完成: ad1={:.2} -> load={:.2}吨, ad2={:.2} -> radius={:.2}米, ad3={:.2} -> angle={:.2}度",
                        raw_data.ad1_load, processed.current_load,
                        raw_data.ad2_radius, processed.working_radius,
                        raw_data.ad3_angle, processed.boom_angle);

                    // 检查报警
                    let mut processed = processed;
                    if let Some(ref am) = alarm_manager {
                        if let Ok(manager) = am.read() {
                            let alarm_results: Vec<_> = manager.check_alarms(&processed);

                            for result in alarm_results {
                                if result.triggered {
                                    if let Some(ref alarm_type) = result.alarm_type {
                                        processed.alarm_sources.push(alarm_type.source.clone());
                                        processed.alarm_messages.push(result.message.clone());

                                        // 如果是角度报警，设置危险状态
                                        if alarm_type.source == AlarmSource::Angle {
                                            processed.is_danger = true;
                                            tracing::warn!("🚨 [ProcessPipeline] 角度报警触发: {}", result.message);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(ref sender) = storage_event_sender {
                        let _ = sender.try_send_data(vec![processed.clone()]);
                    }

                    if let Ok(mut buf) = display_buffer.write() {
                        buf.push(processed);
                    }
                }
            }
        }));
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for ProcessPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> ProcessPipelineConfig {
        ProcessPipelineConfig {
            interval: Duration::from_millis(100),
        }
    }

    #[test]
    fn test_default_config() {
        let config = ProcessPipelineConfig::default();
        assert_eq!(config.interval, Duration::from_millis(100));
    }

    #[test]
    fn test_process_pipeline_creation() {
        let filter_buffer = Arc::new(Mutex::new(FilterBuffer::default()));
        let display_buffer = Arc::new(std::sync::RwLock::new(
            crate::pipeline::ProcessedDataBuffer::new(100),
        ));
        let crane_config = Arc::new(CraneConfig::default());

        let pipeline =
            ProcessPipeline::new(make_config(), filter_buffer, display_buffer, crane_config);

        assert!(!pipeline.is_running());
    }
}
