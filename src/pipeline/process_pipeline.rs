// 计算管道 - 多速率数据流架构
// 从滤波层获取数据 -> 计算处理 -> 发送给显示/存储层

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tokio::task::JoinHandle;
use crate::models::ProcessedData;
use crate::models::crane_config::CraneConfig;
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
            storage_event_sender: Some(storage_event_sender),
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
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
                    fb.get_filtered().clone()
                };

                if let Some(raw_data) = sensor_data {
                    let seq = sequence_number.fetch_add(1, Ordering::Relaxed);
                    let processed = ProcessedData::from_sensor_data_with_config(
                        raw_data,
                        &crane_config,
                        seq,
                    );

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
