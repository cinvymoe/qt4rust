// 起重机数据仓库

use crate::models::SensorData;
use crate::data_sources::SensorDataSource;
use std::sync::{Arc, Mutex};

/// 起重机数据仓库
pub struct CraneDataRepository {
    /// 传感器数据源
    sensor_source: Arc<Mutex<SensorDataSource>>,
    
    /// 数据缓存
    cache: Arc<Mutex<Option<SensorData>>>,
}

impl CraneDataRepository {
    /// 创建新的数据仓库
    pub fn new() -> Self {
        Self {
            sensor_source: Arc::new(Mutex::new(SensorDataSource::new())),
            cache: Arc::new(Mutex::new(None)),
        }
    }
    
    /// 获取最新传感器数据
    pub fn get_latest_sensor_data(&self) -> Result<SensorData, String> {
        let sensor_source = self.sensor_source.lock()
            .map_err(|e| format!("Failed to lock sensor source: {}", e))?;
        
        let data = sensor_source.read_data()?;
        
        // 更新缓存
        if let Ok(mut cache) = self.cache.lock() {
            *cache = Some(data.clone());
        }
        
        Ok(data)
    }
    
    /// 获取缓存的数据
    pub fn get_cached_data(&self) -> Option<SensorData> {
        self.cache.lock().ok()?.clone()
    }
    
    /// 克隆 Repository（用于跨线程共享）
    pub fn clone_arc(&self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            sensor_source: Arc::clone(&self.sensor_source),
            cache: Arc::clone(&self.cache),
        }))
    }
}

impl Default for CraneDataRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_latest_sensor_data() {
        let repo = CraneDataRepository::new();
        let result = repo.get_latest_sensor_data();
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_cache() {
        let repo = CraneDataRepository::new();
        
        // 初始缓存为空
        assert!(repo.get_cached_data().is_none());
        
        // 读取数据后缓存应该有值
        let _ = repo.get_latest_sensor_data();
        assert!(repo.get_cached_data().is_some());
    }
}
