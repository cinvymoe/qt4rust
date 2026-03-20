// 存储仓库 trait（抽象接口）

use async_trait::async_trait;
use crate::models::{ProcessedData, AlarmRecord};

/// 存储仓库 trait
/// 
/// 定义数据持久化的抽象接口，支持多种数据库实现
#[async_trait]
pub trait StorageRepository: Send + Sync {
    /// 批量存储运行数据
    /// 
    /// # 参数
    /// - `data`: 要存储的数据切片
    /// 
    /// # 返回
    /// - `Ok(usize)`: 成功存储的记录数
    /// - `Err(String)`: 错误信息
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String>;
    
    /// 存储单条报警记录
    /// 
    /// # 参数
    /// - `data`: 处理后的数据（包含报警信息）
    /// 
    /// # 返回
    /// - `Ok(i64)`: 报警记录的 ID
    /// - `Err(String)`: 错误信息
    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String>;
    
    /// 查询最近的运行数据
    /// 
    /// # 参数
    /// - `limit`: 查询数量限制
    /// 
    /// # 返回
    /// - `Ok(Vec<ProcessedData>)`: 运行数据列表（按时间倒序）
    /// - `Err(String)`: 错误信息
    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String>;
    
    /// 查询未确认的报警
    /// 
    /// # 返回
    /// - `Ok(Vec<AlarmRecord>)`: 未确认的报警列表
    /// - `Err(String)`: 错误信息
    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String>;
    
    /// 确认报警
    /// 
    /// # 参数
    /// - `alarm_id`: 报警记录 ID
    /// 
    /// # 返回
    /// - `Ok(())`: 确认成功
    /// - `Err(String)`: 错误信息
    async fn acknowledge_alarm(&self, alarm_id: i64) -> Result<(), String>;
    
    /// 获取最后存储的序列号
    /// 
    /// 用于避免重复存储
    /// 
    /// # 返回
    /// - `Ok(u64)`: 最后存储的序列号
    /// - `Err(String)`: 错误信息
    async fn get_last_stored_sequence(&self) -> Result<u64, String>;
    
    /// 健康检查
    /// 
    /// 检查数据库连接是否正常
    /// 
    /// # 返回
    /// - `Ok(())`: 健康
    /// - `Err(String)`: 错误信息
    async fn health_check(&self) -> Result<(), String>;
}
