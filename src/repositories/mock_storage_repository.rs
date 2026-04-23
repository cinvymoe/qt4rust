// Mock 存储仓库（用于测试）

use crate::models::{AlarmRecord, AlarmStatistics, ProcessedData};
use crate::repositories::sensor_data_repository::SensorDataRepository;
use crate::repositories::storage_repository::StorageRepository;
use async_trait::async_trait;
use sensor_core::SensorData;
use std::sync::Mutex;

/// Mock 存储仓库
///
/// 用于单元测试，不依赖真实数据库
pub struct MockStorageRepository {
    runtime_data: Mutex<Vec<ProcessedData>>,
    sensor_data: Mutex<Vec<SensorData>>,
    alarm_records: Mutex<Vec<AlarmRecord>>,
    should_fail: Mutex<bool>,
}

impl MockStorageRepository {
    /// 创建新的 Mock 存储仓库
    pub fn new() -> Self {
        Self {
            runtime_data: Mutex::new(Vec::new()),
            sensor_data: Mutex::new(Vec::new()),
            alarm_records: Mutex::new(Vec::new()),
            should_fail: Mutex::new(false),
        }
    }

    /// 设置是否模拟失败
    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    /// 获取存储的运行数据数量
    pub fn get_runtime_data_count(&self) -> usize {
        self.runtime_data.lock().unwrap().len()
    }

    /// 获取存储的报警数量
    pub fn get_alarm_count(&self) -> usize {
        self.alarm_records.lock().unwrap().len()
    }

    /// 清空所有数据
    pub fn clear(&self) {
        self.runtime_data.lock().unwrap().clear();
        self.alarm_records.lock().unwrap().clear();
    }
}

impl Default for MockStorageRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageRepository for MockStorageRepository {
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String> {
        if *self.should_fail.lock().unwrap() {
            return Err("Mock failure".to_string());
        }

        let mut storage = self.runtime_data.lock().unwrap();
        storage.extend_from_slice(data);
        Ok(data.len())
    }

    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String> {
        if *self.should_fail.lock().unwrap() {
            return Err("Mock failure".to_string());
        }

        let mut alarms = self.alarm_records.lock().unwrap();
        let alarm = AlarmRecord::from_processed_data(data);
        alarms.push(alarm);
        Ok(alarms.len() as i64)
    }

    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String> {
        let storage = self.runtime_data.lock().unwrap();
        Ok(storage.iter().rev().take(limit).cloned().collect())
    }

    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String> {
        let alarms = self.alarm_records.lock().unwrap();
        Ok(alarms.iter().filter(|a| !a.acknowledged).cloned().collect())
    }

    async fn acknowledge_alarm(&self, alarm_id: i64) -> Result<(), String> {
        let mut alarms = self.alarm_records.lock().unwrap();
        if let Some(alarm) = alarms.get_mut((alarm_id - 1) as usize) {
            alarm.acknowledged = true;
            alarm.acknowledged_at = Some(std::time::SystemTime::now());
            Ok(())
        } else {
            Err("Alarm not found".to_string())
        }
    }

    async fn get_last_stored_sequence(&self) -> Result<u64, String> {
        let storage = self.runtime_data.lock().unwrap();
        Ok(storage.last().map(|d| d.sequence_number).unwrap_or(0))
    }

    async fn health_check(&self) -> Result<(), String> {
        if *self.should_fail.lock().unwrap() {
            Err("Mock health check failed".to_string())
        } else {
            Ok(())
        }
    }

    async fn purge_old_records(
        &self,
        max_records: usize,
        _purge_threshold: usize,
    ) -> Result<usize, String> {
        if max_records == 0 {
            return Ok(0);
        }

        let mut storage = self.runtime_data.lock().unwrap();
        if storage.len() <= max_records {
            return Ok(0);
        }

        let removed = storage.len() - max_records;
        storage.drain(0..removed);
        Ok(removed)
    }

    async fn purge_old_alarms(
        &self,
        alarm_max_records: usize,
        _alarm_purge_threshold: usize,
    ) -> Result<usize, String> {
        if alarm_max_records == 0 {
            return Ok(0);
        }

        let mut alarms = self.alarm_records.lock().unwrap();
        if alarms.len() <= alarm_max_records {
            return Ok(0);
        }

        let removed = alarms.len() - alarm_max_records;
        alarms.drain(0..removed);
        Ok(removed)
    }

    async fn get_runtime_data_count(&self) -> Result<i64, String> {
        let storage = self.runtime_data.lock().unwrap();
        Ok(storage.len() as i64)
    }

    async fn get_runtime_data_range(
        &self,
        offset: i64,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String> {
        let storage = self.runtime_data.lock().unwrap();
        let start = offset as usize;
        let end = (start + limit).min(storage.len());

        if start >= storage.len() {
            return Ok(vec![]);
        }

        Ok(storage[start..end].to_vec())
    }

    async fn query_runtime_data_by_time_range(
        &self,
        start_time: std::time::SystemTime,
        end_time: std::time::SystemTime,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String> {
        let storage = self.runtime_data.lock().unwrap();
        let result: Vec<ProcessedData> = storage
            .iter()
            .filter(|d| d.timestamp >= start_time && d.timestamp <= end_time)
            .rev()
            .take(limit)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn query_alarm_records_by_time_range(
        &self,
        start_time: std::time::SystemTime,
        end_time: std::time::SystemTime,
    ) -> Result<Vec<AlarmRecord>, String> {
        let alarms = self.alarm_records.lock().unwrap();
        let result: Vec<AlarmRecord> = alarms
            .iter()
            .filter(|a| a.timestamp >= start_time && a.timestamp <= end_time)
            .rev()
            .cloned()
            .collect();
        Ok(result)
    }

    async fn query_alarm_records_by_filter(
        &self,
        _filter: &str,
    ) -> Result<Vec<AlarmRecord>, String> {
        let alarms = self.alarm_records.lock().unwrap();
        Ok(alarms.iter().rev().cloned().collect())
    }

    async fn query_runtime_data_by_filter(
        &self,
        _filter: &str,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String> {
        let storage = self.runtime_data.lock().unwrap();
        Ok(storage.iter().rev().take(limit).cloned().collect())
    }

    async fn get_alarm_statistics(&self) -> Result<AlarmStatistics, String> {
        let alarms = self.alarm_records.lock().unwrap();
        let total_count = alarms.len() as i32;
        let warning_count = alarms
            .iter()
            .filter(|a| a.alarm_type == crate::models::AlarmType::Warning)
            .count() as i32;
        let danger_count = alarms
            .iter()
            .filter(|a| a.alarm_type == crate::models::AlarmType::Danger)
            .count() as i32;
        Ok(AlarmStatistics::new(total_count, warning_count, danger_count))
    }

    async fn get_alarm_statistics_by_time_range(
        &self,
        start_time: std::time::SystemTime,
        end_time: std::time::SystemTime,
    ) -> Result<AlarmStatistics, String> {
        let alarms = self.alarm_records.lock().unwrap();
        let filtered: Vec<_> = alarms
            .iter()
            .filter(|a| a.timestamp >= start_time && a.timestamp <= end_time)
            .collect();
        let total_count = filtered.len() as i32;
        let warning_count = filtered
            .iter()
            .filter(|a| a.alarm_type == crate::models::AlarmType::Warning)
            .count() as i32;
        let danger_count = filtered
            .iter()
            .filter(|a| a.alarm_type == crate::models::AlarmType::Danger)
            .count() as i32;
        Ok(AlarmStatistics::new(total_count, warning_count, danger_count))
    }
}

#[async_trait]
impl SensorDataRepository for MockStorageRepository {
    async fn save_sensor_data_batch(&self, data: &[SensorData]) -> Result<usize, String> {
        if *self.should_fail.lock().unwrap() {
            return Err("Mock failure".to_string());
        }

        let mut storage = self.sensor_data.lock().unwrap();
        storage.extend_from_slice(data);
        Ok(data.len())
    }

    async fn query_recent_sensor_data(&self, limit: usize) -> Result<Vec<SensorData>, String> {
        let storage = self.sensor_data.lock().unwrap();
        Ok(storage.iter().rev().take(limit).cloned().collect())
    }

    async fn get_latest_sensor_data(&self) -> Result<Option<SensorData>, String> {
        let storage = self.sensor_data.lock().unwrap();
        Ok(storage.last().cloned())
    }

    async fn get_sensor_data_count(&self) -> Result<i64, String> {
        let storage = self.sensor_data.lock().unwrap();
        Ok(storage.len() as i64)
    }

    async fn purge_old_sensor_data(&self, max_records: usize) -> Result<usize, String> {
        if max_records == 0 {
            return Ok(0);
        }

        let mut storage = self.sensor_data.lock().unwrap();
        if storage.len() <= max_records {
            return Ok(0);
        }

        let removed = storage.len() - max_records;
        storage.drain(0..removed);
        Ok(removed)
    }

    async fn health_check(&self) -> Result<(), String> {
        if *self.should_fail.lock().unwrap() {
            Err("Mock health check failed".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensor_core::SensorData;

    #[tokio::test]
    async fn test_new() {
        let repo = MockStorageRepository::new();
        assert_eq!(repo.get_runtime_data_count(), 0);
        assert_eq!(repo.get_alarm_count(), 0);
    }

    #[tokio::test]
    async fn test_save_runtime_data() {
        let repo = MockStorageRepository::new();
        let sensor_data = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);

        let result = repo.save_runtime_data_batch(&[processed]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(repo.get_runtime_data_count(), 1);
    }

    #[tokio::test]
    async fn test_save_alarm_record() {
        let repo = MockStorageRepository::new();
        let sensor_data = SensorData::from_tuple(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);

        let result = repo.save_alarm_record(&processed).await;
        assert!(result.is_ok());
        assert_eq!(repo.get_alarm_count(), 1);
    }

    #[tokio::test]
    async fn test_should_fail() {
        let repo = MockStorageRepository::new();
        repo.set_should_fail(true);

        let sensor_data = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);

        let result = repo.save_runtime_data_batch(&[processed]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_query_recent_runtime_data() {
        let repo = MockStorageRepository::new();

        // 添加 5 条数据
        for i in 1..=5 {
            let sensor_data = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
            let processed = ProcessedData::from_sensor_data(sensor_data, i);
            repo.save_runtime_data_batch(&[processed]).await.unwrap();
        }

        // 查询最近 3 条
        let data = repo.query_recent_runtime_data(3).await.unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0].sequence_number, 5); // 倒序
        assert_eq!(data[1].sequence_number, 4);
        assert_eq!(data[2].sequence_number, 3);
    }

    #[tokio::test]
    async fn test_acknowledge_alarm() {
        let repo = MockStorageRepository::new();

        // 添加报警
        let sensor_data = SensorData::from_tuple(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        let alarm_id = repo.save_alarm_record(&processed).await.unwrap();

        // 确认报警
        assert!(repo.acknowledge_alarm(alarm_id).await.is_ok());

        // 查询未确认报警
        let alarms = repo.query_unacknowledged_alarms().await.unwrap();
        assert_eq!(alarms.len(), 0);
    }

    #[tokio::test]
    async fn test_get_last_stored_sequence() {
        let repo = MockStorageRepository::new();

        // 初始为 0
        let seq = repo.get_last_stored_sequence().await.unwrap();
        assert_eq!(seq, 0);

        // 添加数据
        let sensor_data = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 5);
        repo.save_runtime_data_batch(&[processed]).await.unwrap();

        // 应该返回 5
        let seq = repo.get_last_stored_sequence().await.unwrap();
        assert_eq!(seq, 5);
    }

    #[tokio::test]
    async fn test_clear() {
        let repo = MockStorageRepository::new();

        // 添加数据
        let sensor_data = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        repo.save_runtime_data_batch(&[processed.clone()])
            .await
            .unwrap();
        repo.save_alarm_record(&processed).await.unwrap();

        assert_eq!(repo.get_runtime_data_count(), 1);
        assert_eq!(repo.get_alarm_count(), 1);

        // 清空
        repo.clear();

        assert_eq!(repo.get_runtime_data_count(), 0);
        assert_eq!(repo.get_alarm_count(), 0);
    }
}
