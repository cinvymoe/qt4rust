// 计算管道 - 多速率数据流架构
// 从滤波层获取数据 -> 计算处理 -> 发送给显示/存储层

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};
use std::time::Duration;
use tokio::task::JoinHandle;
use crate::models::ProcessedData;
use crate::models::crane_config::CraneConfig;
use crate::models::sensor_calibration::SensorCalibration;
use crate::models::rated_load_table::RatedLoadTable;
use crate::pipeline::filter_buffer::FilterBuffer;
use crate::pipeline::shared_buffer::SharedBuffer;
use crate::pipeline::event_channel::StorageEventSender;

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
    storage_event_sender: Option<StorageEventSender>,
    running: Arc<AtomicBool>,
    sequence_number: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
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
            storage_event_sender: None,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
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
            storage_event_sender: Some(storage_event_sender),
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
        }
    }
    
    /// 设置热重载配置引用（用于配置热重载）
    pub fn set_hot_reload_config(
        &mut self,
        sensor_calibration: Arc<RwLock<SensorCalibration>>,
        rated_load_table: Arc<RwLock<RatedLoadTable>>,
    ) {
        self.sensor_calibration = Some(sensor_calibration.clone());
        self.rated_load_table = Some(rated_load_table.clone());
        
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
        let storage_event_sender = self.storage_event_sender.clone();
        let sequence_number = Arc::clone(&self.sequence_number);
        let running = Arc::clone(&self.running);
        let interval = self.config.interval;

        self.handle = Some(qt_threading_utils::runtime::global_runtime().spawn(async move {
            let mut tick_interval = tokio::time::interval(interval);

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
                    let processed = if let (Some(cal), Some(table)) = (&sensor_calibration, &rated_load_table) {
                        // 使用热重载配置
                        let cal_guard = cal.read().unwrap();
                        let table_guard = table.read().unwrap();
                        
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
                        
                        // 创建临时CraneConfig
                        let hot_config = CraneConfig {
                            sensor_calibration: cal_guard.clone(),
                            rated_load_table: table_guard.clone(),
                            alarm_thresholds: crane_config.alarm_thresholds.clone(),
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
            crate::pipeline::ProcessedDataBuffer::new(100)
        ));
        let crane_config = Arc::new(CraneConfig::default());

        let pipeline = ProcessPipeline::new(
            make_config(),
            filter_buffer,
            display_buffer,
            crane_config,
        );

        assert!(!pipeline.is_running());
    }
}
