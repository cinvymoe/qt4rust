// 存储队列（线程安全）

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::models::ProcessedData;

/// 存储队列
/// 
/// 管理待存储的数据，避免重复存储
pub struct StorageQueue {
    /// 数据队列
    queue: Arc<Mutex<VecDeque<ProcessedData>>>,
    
    /// 最大容量
    max_size: usize,
    
    /// 最后存储的序列号
    last_stored_sequence: Arc<Mutex<u64>>,
}

impl StorageQueue {
    /// 创建新的存储队列
    /// 
    /// # 参数
    /// - `max_size`: 队列最大容量
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
            last_stored_sequence: Arc::new(Mutex::new(0)),
        }
    }
    
    /// 添加数据到队列
    /// 
    /// 自动过滤已存储的数据（sequence_number <= last_stored_sequence）
    /// 
    /// # 参数
    /// - `data`: 要添加的数据
    /// 
    /// # 返回
    /// - `Ok(())`: 添加成功或已存储（跳过）
    /// - `Err(String)`: 错误信息
    pub fn push(&self, data: ProcessedData) -> Result<(), String> {
        // 检查是否已存储
        if let Ok(last_seq) = self.last_stored_sequence.lock() {
            if data.sequence_number <= *last_seq {
                // 已存储，跳过
                return Ok(());
            }
        }
        
        // 添加到队列
        let mut queue = self.queue.lock()
            .map_err(|e| format!("Failed to lock queue: {}", e))?;
        
        // 检查队列容量
        if queue.len() >= self.max_size {
            eprintln!("[WARN] Storage queue full ({}), dropping oldest data", self.max_size);
            queue.pop_front();
        }
        
        queue.push_back(data);
        Ok(())
    }
    
    /// 批量取出数据（不删除）
    /// 
    /// # 参数
    /// - `count`: 要取出的数量
    /// 
    /// # 返回
    /// 数据列表（最多 count 条）
    pub fn peek_batch(&self, count: usize) -> Vec<ProcessedData> {
        if let Ok(queue) = self.queue.lock() {
            queue.iter().take(count).cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// 删除已存储的数据
    /// 
    /// # 参数
    /// - `count`: 要删除的数量
    /// - `max_sequence`: 已存储的最大序列号
    /// 
    /// # 返回
    /// - `Ok(())`: 删除成功
    /// - `Err(String)`: 错误信息
    pub fn remove_stored(&self, count: usize, max_sequence: u64) -> Result<(), String> {
        let mut queue = self.queue.lock()
            .map_err(|e| format!("Failed to lock queue: {}", e))?;
        
        // 删除前 count 条数据
        for _ in 0..count.min(queue.len()) {
            queue.pop_front();
        }
        
        // 更新最后存储的序列号
        if let Ok(mut last_seq) = self.last_stored_sequence.lock() {
            *last_seq = max_sequence;
        }
        
        Ok(())
    }
    
    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.queue.lock().map(|q| q.len()).unwrap_or(0)
    }
    
    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// 获取最后存储的序列号
    pub fn last_stored_sequence(&self) -> u64 {
        self.last_stored_sequence.lock()
            .map(|seq| *seq)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sensor_data::SensorData;
    
    #[test]
    fn test_new() {
        let queue = StorageQueue::new(100);
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        assert_eq!(queue.last_stored_sequence(), 0);
    }
    
    #[test]
    fn test_push() {
        let queue = StorageQueue::new(10);
        let sensor_data = SensorData::new(20.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        
        assert!(queue.push(processed).is_ok());
        assert_eq!(queue.len(), 1);
    }
    
    #[test]
    fn test_push_duplicate() {
        let queue = StorageQueue::new(10);
        let sensor_data = SensorData::new(20.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        
        // 第一次添加
        assert!(queue.push(processed.clone()).is_ok());
        assert_eq!(queue.len(), 1);
        
        // 标记为已存储
        assert!(queue.remove_stored(1, 1).is_ok());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.last_stored_sequence(), 1);
        
        // 再次添加相同序列号（应该被跳过）
        assert!(queue.push(processed).is_ok());
        assert_eq!(queue.len(), 0);  // 未添加
    }
    
    #[test]
    fn test_peek_batch() {
        let queue = StorageQueue::new(10);
        
        // 添加 5 条数据
        for i in 1..=5 {
            let sensor_data = SensorData::new(20.0, 10.0, 60.0);
            let processed = ProcessedData::from_sensor_data(sensor_data, i);
            queue.push(processed).unwrap();
        }
        
        // 取出 3 条
        let batch = queue.peek_batch(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].sequence_number, 1);
        assert_eq!(batch[1].sequence_number, 2);
        assert_eq!(batch[2].sequence_number, 3);
        
        // 队列长度不变
        assert_eq!(queue.len(), 5);
    }
    
    #[test]
    fn test_remove_stored() {
        let queue = StorageQueue::new(10);
        
        // 添加 5 条数据
        for i in 1..=5 {
            let sensor_data = SensorData::new(20.0, 10.0, 60.0);
            let processed = ProcessedData::from_sensor_data(sensor_data, i);
            queue.push(processed).unwrap();
        }
        
        // 删除前 3 条
        assert!(queue.remove_stored(3, 3).is_ok());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.last_stored_sequence(), 3);
        
        // 剩余数据
        let batch = queue.peek_batch(10);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].sequence_number, 4);
        assert_eq!(batch[1].sequence_number, 5);
    }
    
    #[test]
    fn test_queue_full() {
        let queue = StorageQueue::new(3);
        
        // 添加 5 条数据（超过容量）
        for i in 1..=5 {
            let sensor_data = SensorData::new(20.0, 10.0, 60.0);
            let processed = ProcessedData::from_sensor_data(sensor_data, i);
            queue.push(processed).unwrap();
        }
        
        // 队列长度应该是 3（最大容量）
        assert_eq!(queue.len(), 3);
        
        // 应该保留最新的 3 条（3, 4, 5）
        let batch = queue.peek_batch(10);
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].sequence_number, 3);
        assert_eq!(batch[1].sequence_number, 4);
        assert_eq!(batch[2].sequence_number, 5);
    }
}
