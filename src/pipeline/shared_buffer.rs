// 共享数据缓冲区

use crate::models::ProcessedData;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// 最大内存限制（10MB）
const MAX_MEMORY: usize = 10 * 1024 * 1024;

/// 缓冲区统计信息
#[derive(Debug, Default, Clone)]
pub struct BufferStats {
    /// 总采集次数
    pub total_collections: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub error_count: u64,
    /// 最后更新时间
    pub last_update_time: Option<SystemTime>,
}

/// 共享数据缓冲区
#[derive(Debug)]
pub struct ProcessedDataBuffer {
    /// 最新数据
    latest: Option<ProcessedData>,

    /// 历史数据队列
    history: VecDeque<ProcessedData>,

    /// 最大历史容量
    max_history_size: usize,

    /// 统计信息
    stats: BufferStats,
}

impl ProcessedDataBuffer {
    /// 创建新的缓冲区
    pub fn new(max_history_size: usize) -> Self {
        Self {
            latest: None,
            history: VecDeque::with_capacity(max_history_size),
            max_history_size,
            stats: BufferStats::default(),
        }
    }

    /// 写入新数据
    pub fn push(&mut self, data: ProcessedData) {
        self.latest = Some(data.clone());

        // 限制历史大小（FIFO）
        if self.history.len() >= self.max_history_size {
            self.history.pop_front();
        }
        self.history.push_back(data);

        // 检查内存使用，如果超过限制则移除最旧数据
        while self.estimated_memory_usage() > MAX_MEMORY && !self.history.is_empty() {
            self.history.pop_front();
            tracing::warn!(
                " Memory pressure: removed oldest data (current usage: {} bytes)",
                self.estimated_memory_usage()
            );
        }

        // 更新统计
        self.stats.total_collections += 1;
        self.stats.success_count += 1;
        self.stats.last_update_time = Some(SystemTime::now());
    }

    /// 读取最新数据
    pub fn get_latest(&self) -> Option<ProcessedData> {
        self.latest.clone()
    }

    /// 读取历史数据（按时间倒序）
    pub fn get_history(&self, count: usize) -> Vec<ProcessedData> {
        self.history.iter().rev().take(count).cloned().collect()
    }

    /// 记录错误
    pub fn record_error(&mut self) {
        self.stats.error_count += 1;
        self.stats.total_collections += 1;
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &BufferStats {
        &self.stats
    }

    /// 估算内存使用（字节）
    pub fn estimated_memory_usage(&self) -> usize {
        self.history.len() * std::mem::size_of::<ProcessedData>()
    }
}

/// 线程安全的共享缓冲区类型
pub type SharedBuffer = Arc<RwLock<ProcessedDataBuffer>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SensorData;

    fn create_test_data(seq: u64) -> ProcessedData {
        let sensor_data = SensorData::new(10.0 + seq as f64, 8.0, 60.0);
        ProcessedData::from_sensor_data(sensor_data, seq)
    }

    #[test]
    fn test_push_and_get_latest() {
        let mut buffer = ProcessedDataBuffer::new(10);
        let data = create_test_data(1);

        buffer.push(data.clone());

        let latest = buffer.get_latest();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().sequence_number, 1);
    }

    #[test]
    fn test_capacity_limit() {
        let mut buffer = ProcessedDataBuffer::new(5);

        // 添加 10 条数据
        for i in 0..10 {
            buffer.push(create_test_data(i));
        }

        // 应该只保留最后 5 条
        let history = buffer.get_history(10);
        assert_eq!(history.len(), 5);
        assert_eq!(history[0].sequence_number, 9); // 最新的
        assert_eq!(history[4].sequence_number, 5); // 最旧的
    }

    #[test]
    fn test_get_history() {
        let mut buffer = ProcessedDataBuffer::new(10);

        for i in 0..5 {
            buffer.push(create_test_data(i));
        }

        let history = buffer.get_history(3);
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].sequence_number, 4); // 最新的
        assert_eq!(history[2].sequence_number, 2);
    }

    #[test]
    fn test_stats() {
        let mut buffer = ProcessedDataBuffer::new(10);

        buffer.push(create_test_data(1));
        buffer.push(create_test_data(2));
        buffer.record_error();

        let stats = buffer.get_stats();
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.total_collections, 3);
        assert!(stats.last_update_time.is_some());
    }

    #[test]
    fn test_memory_usage() {
        let mut buffer = ProcessedDataBuffer::new(100);

        for i in 0..10 {
            buffer.push(create_test_data(i));
        }

        let usage = buffer.estimated_memory_usage();
        let expected = 10 * std::mem::size_of::<ProcessedData>();

        // Should be exactly the expected size
        assert_eq!(usage, expected);
    }
}
