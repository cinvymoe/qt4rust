// 存储服务 - 从 StoragePipeline 提取的业务逻辑层
//
// 职责: 处理报警防抖、批量写入、数据清理等存储相关业务逻辑。
// StoragePipeline 只负责数据流（接收、缓冲、定时刷盘），
// 具体的存储决策由 StorageService 处理。

use super::alarm_debouncer::{AlarmAction, AlarmDebounceConfig, AlarmDebouncer};
use crate::models::ProcessedData;
use crate::pipeline::retry_policy::{with_retry, RetryConfig};
use crate::pipeline::StorageError;
use crate::repositories::storage_repository::StorageRepository;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 存储服务配置 — 只包含存储业务相关的配置
#[derive(Debug, Clone)]
pub struct StorageServiceConfig {
    pub max_records: usize,
    pub purge_threshold: usize,
    pub alarm_max_records: usize,
    pub alarm_purge_threshold: usize,
    pub max_retries: u32,
    pub retry_delay: std::time::Duration,
    pub alarm_debounce: AlarmDebounceConfig,
}

impl Default for StorageServiceConfig {
    fn default() -> Self {
        Self {
            max_records: 0,
            purge_threshold: 0,
            alarm_max_records: 0,
            alarm_purge_threshold: 0,
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(100),
            alarm_debounce: AlarmDebounceConfig::default(),
        }
    }
}

/// 存储服务 — 处理报警防抖、批量写入和数据清理
pub struct StorageService {
    repository: Arc<dyn StorageRepository>,
    config: StorageServiceConfig,
    alarm_debouncer: AlarmDebouncer,
    last_stored_sequence: Arc<AtomicU64>,
}

impl StorageService {
    pub fn new(repository: Arc<dyn StorageRepository>, config: StorageServiceConfig) -> Self {
        let alarm_debouncer = AlarmDebouncer::new(config.alarm_debounce.clone());
        Self {
            repository,
            config,
            alarm_debouncer,
            last_stored_sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn initialize_sequence(&self) -> Result<(), String> {
        if let Ok(last_seq) = self.repository.get_last_stored_sequence().await {
            self.last_stored_sequence.store(last_seq, Ordering::Relaxed);
            tracing::info!("StorageService last_seq initialized to {}", last_seq);
        }
        Ok(())
    }

    pub fn process_alarm(&self, data: &ProcessedData) -> AlarmAction {
        self.alarm_debouncer.process(data)
    }

    pub async fn save_alarm(&self, data: ProcessedData) {
        let repo = Arc::clone(&self.repository);
        let alarm_max = self.config.alarm_max_records;
        let alarm_purge = self.config.alarm_purge_threshold;
        let seq = data.sequence_number;

        tokio::spawn(async move {
            match repo.save_alarm_record(&data).await {
                Ok(alarm_id) => {
                    tracing::info!("Alarm saved with id: {} (sequence: {})", alarm_id, seq);
                    if alarm_max > 0 {
                        if let Err(e) = repo.purge_old_alarms(alarm_max, alarm_purge).await {
                            tracing::error!("Alarm purge failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to save alarm: {}", e);
                }
            }
        });
    }

    pub async fn save_batch(&self, data: &[ProcessedData]) -> Result<usize, StorageError> {
        let max_seq = data.iter().map(|d| d.sequence_number).max().unwrap_or(0);

        let result = with_retry(
            &RetryConfig {
                max_retries: self.config.max_retries,
                base_delay: self.config.retry_delay,
                ..Default::default()
            },
            || {
                let data_clone = data.to_vec();
                let repo = Arc::clone(&self.repository);
                async move {
                    repo.save_runtime_data_batch(&data_clone)
                        .await
                        .map_err(|e| StorageError::Database(e.to_string()))
                }
            },
        )
        .await;

        match result {
            Ok(saved) => {
                tracing::info!("Saved {} records (seq <= {})", saved, max_seq);
                self.last_stored_sequence.store(max_seq, Ordering::Release);
                Ok(saved)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn purge_if_needed(&self) {
        let max_records = self.config.max_records;
        let purge_threshold = self.config.purge_threshold;

        if max_records > 0 {
            let repo = Arc::clone(&self.repository);
            tokio::spawn(async move {
                if let Err(e) = repo.purge_old_records(max_records, purge_threshold).await {
                    tracing::error!("Purge failed: {}", e);
                }
            });
        }
    }

    pub fn last_stored_sequence(&self) -> u64 {
        self.last_stored_sequence.load(Ordering::Relaxed)
    }

    pub fn set_initial_sequence(&self, seq: u64) {
        self.last_stored_sequence.store(seq, Ordering::Relaxed);
    }

    pub fn notify_danger_cleared(&self) {
        self.alarm_debouncer.notify_danger_cleared();
    }

    pub fn max_records(&self) -> usize {
        self.config.max_records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alarm::alarm_type::AlarmSource;
    use crate::repositories::mock_storage_repository::MockStorageRepository;
    use std::time::SystemTime;

    fn make_processed_data(seq: u64, is_danger: bool) -> ProcessedData {
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
            sequence_number: seq,
            alarm_sources: if is_danger {
                vec![AlarmSource::Moment]
            } else {
                vec![]
            },
            alarm_messages: Vec::new(),
        }
    }

    fn create_test_service() -> StorageService {
        let repo: Arc<dyn StorageRepository> = Arc::new(MockStorageRepository::new());
        let config = StorageServiceConfig::default();
        StorageService::new(repo, config)
    }

    #[tokio::test]
    async fn test_save_batch() {
        let service = create_test_service();
        let data = vec![make_processed_data(1, false)];

        let result = service.save_batch(&data).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(service.last_stored_sequence(), 1);
    }

    #[tokio::test]
    async fn test_process_alarm_trigger() {
        let config = StorageServiceConfig {
            alarm_debounce: AlarmDebounceConfig {
                alarm_debounce_count: 1,
                alarm_clear_debounce_count: 1,
            },
            ..Default::default()
        };
        let repo: Arc<dyn StorageRepository> = Arc::new(MockStorageRepository::new());
        let service = StorageService::new(repo, config);

        let action = service.process_alarm(&make_processed_data(1, true));
        assert!(matches!(action, AlarmAction::TriggerAlarm(_)));
    }

    #[tokio::test]
    async fn test_process_alarm_clear() {
        let config = StorageServiceConfig {
            alarm_debounce: AlarmDebounceConfig {
                alarm_debounce_count: 1,
                alarm_clear_debounce_count: 1,
            },
            ..Default::default()
        };
        let repo: Arc<dyn StorageRepository> = Arc::new(MockStorageRepository::new());
        let service = StorageService::new(repo, config);

        service.process_alarm(&make_processed_data(1, true));
        let action = service.process_alarm(&make_processed_data(2, false));
        assert_eq!(action, AlarmAction::ClearAlarm);
    }

    #[tokio::test]
    async fn test_initialize_sequence() {
        let service = create_test_service();
        service.set_initial_sequence(42);
        assert_eq!(service.last_stored_sequence(), 42);
    }

    #[tokio::test]
    async fn test_notify_danger_cleared() {
        let service = create_test_service();
        service.notify_danger_cleared();
    }
}
