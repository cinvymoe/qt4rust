// SensorData 存储仓库 trait

use crate::models::SensorData;
use async_trait::async_trait;

/// SensorData 存储仓库 trait
///
/// 定义原始传感器数据的持久化抽象接口
#[async_trait]
pub trait SensorDataRepository: Send + Sync {
    /// 批量存储 SensorData
    ///
    /// # 参数
    /// - `data`: 要存储的 SensorData 切片
    ///
    /// # 返回
    /// - `Ok(usize)`: 成功存储的记录数
    /// - `Err(String)`: 错误信息
    async fn save_sensor_data_batch(&self, data: &[SensorData]) -> Result<usize, String>;

    /// 查询最近的 SensorData
    ///
    /// # 参数
    /// - `limit`: 查询数量限制
    ///
    /// # 返回
    /// - `Ok(Vec<SensorData>)`: SensorData 列表（按时间倒序）
    /// - `Err(String)`: 错误信息
    async fn query_recent_sensor_data(&self, limit: usize) -> Result<Vec<SensorData>, String>;

    /// 获取最新的一条 SensorData
    ///
    /// # 返回
    /// - `Ok(Option<SensorData>)`: 最新的 SensorData，如果没有记录则返回 None
    /// - `Err(String)`: 错误信息
    async fn get_latest_sensor_data(&self) -> Result<Option<SensorData>, String>;

    /// 获取 SensorData 总记录数
    ///
    /// # 返回
    /// - `Ok(i64)`: 记录总数
    /// - `Err(String)`: 错误信息
    async fn get_sensor_data_count(&self) -> Result<i64, String>;

    /// 清理旧数据（LRU）
    ///
    /// 当记录数超过 max_records 时，删除最早的记录直到记录数降到 max_records
    ///
    /// # 参数
    /// - `max_records`: 最大记录条数
    ///
    /// # 返回
    /// - `Ok(usize)`: 删除的记录数
    /// - `Err(String)`: 错误信息
    async fn purge_old_sensor_data(&self, max_records: usize) -> Result<usize, String>;

    /// 健康检查
    ///
    /// 检查仓库连接是否正常
    ///
    /// # 返回
    /// - `Ok(())`: 健康
    /// - `Err(String)`: 错误信息
    async fn health_check(&self) -> Result<(), String>;
}
