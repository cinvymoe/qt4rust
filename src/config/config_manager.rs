// src/config/config_manager.rs

use std::sync::{Arc, Mutex};
use crane_data_layer::error::{DataError, DataResult};
use crate::models::crane_config::CraneConfig;
use crate::data_sources::config_data_source::ConfigDataSource;

/// 配置变更观察者 trait
/// 
/// 实现此 trait 以接收配置变更通知
pub trait ConfigObserver: Send + Sync {
    /// 配置变更回调
    /// 
    /// # 参数
    /// - `new_config`: 新的配置对象
    fn on_config_changed(&self, new_config: &CraneConfig);
}

/// 配置管理器
/// 
/// 负责配置的加载、重载和变更通知。
/// 使用观察者模式通知订阅者配置变更。
/// 
/// # 示例
/// ```
/// use crate::config::config_manager::ConfigManager;
/// 
/// let manager = ConfigManager::new()?;
/// let config = manager.get_config()?;
/// ```
pub struct ConfigManager {
    /// 配置数据源
    config_source: Arc<ConfigDataSource>,
    /// 观察者列表
    observers: Arc<Mutex<Vec<Box<dyn ConfigObserver>>>>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    /// 
    /// 初始化配置数据源并加载配置。
    /// 
    /// # 返回
    /// - `Ok(ConfigManager)`: 初始化成功
    /// - `Err(DataError)`: 初始化失败（配置加载失败）
    /// 
    /// # 示例
    /// ```
    /// let manager = ConfigManager::new()?;
    /// ```
    pub fn new() -> DataResult<Self> {
        tracing::info!("初始化配置管理器...");
        
        let config_source = Arc::new(ConfigDataSource::new());
        
        // 启动时加载配置
        config_source.load()?;
        
        tracing::info!("配置管理器初始化成功");
        
        Ok(Self {
            config_source,
            observers: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// 获取当前配置
    /// 
    /// 从缓存中读取配置，不会触发文件 I/O。
    /// 
    /// # 返回
    /// - `Ok(CraneConfig)`: 返回当前配置
    /// - `Err(DataError::NotFound)`: 配置未加载
    /// 
    /// # 示例
    /// ```
    /// let config = manager.get_config()?;
    /// let weight = config.sensor_calibration.convert_weight_ad_to_value(2048.0);
    /// ```
    pub fn get_config(&self) -> DataResult<CraneConfig> {
        self.config_source.get_cached_config()
    }
    
    /// 获取配置版本号
    /// 
    /// 版本号在每次配置重载时递增，可用于检测配置是否已更新。
    /// 
    /// # 返回
    /// - `Ok(u64)`: 当前配置版本号
    /// - `Err(DataError::NotFound)`: 配置未加载
    /// 
    /// # 示例
    /// ```
    /// let version = manager.get_config_version()?;
    /// println!("当前配置版本: {}", version);
    /// ```
    pub fn get_config_version(&self) -> DataResult<u64> {
        self.config_source.get_config_version()
    }
    
    /// 重新加载配置
    /// 
    /// 从文件重新读取配置，验证后更新缓存，并通知所有观察者。
    /// 如果加载失败，会自动回滚到旧配置。
    /// 
    /// # 返回
    /// - `Ok(CraneConfig)`: 重新加载成功，返回新配置
    /// - `Err(DataError)`: 重新加载失败，已回滚到旧配置
    /// 
    /// # 示例
    /// ```
    /// match manager.reload_config() {
    ///     Ok(config) => println!("配置重载成功，版本: {}", manager.get_config_version()?),
    ///     Err(e) => eprintln!("配置重载失败: {:?}", e),
    /// }
    /// ```
    pub fn reload_config(&self) -> DataResult<CraneConfig> {
        tracing::info!("用户触发配置重载");
        
        // 重新加载配置
        let new_config = self.config_source.reload()?;
        
        // 通知所有观察者（在单独的作用域中持有锁，避免死锁）
        {
            let observers = self.observers.lock()
                .map_err(|e| {
                    let err_msg = format!("无法锁定观察者列表: {}", e);
                    tracing::error!("{}", err_msg);
                    DataError::Unknown(err_msg)
                })?;
            
            tracing::info!("通知 {} 个观察者配置已更新", observers.len());
            
            // 不在持有锁的情况下调用观察者，避免死锁
            // 创建观察者指针列表
            let observers_clone: Vec<_> = observers.iter()
                .map(|o| o.as_ref() as *const dyn ConfigObserver)
                .collect();
            
            drop(observers); // 释放锁
            
            // 安全地通知观察者
            for observer_ptr in observers_clone {
                unsafe {
                    (*observer_ptr).on_config_changed(&new_config);
                }
            }
        }
        
        tracing::info!("配置重载完成，已通知所有观察者");
        Ok(new_config)
    }
    
    /// 订阅配置变更
    /// 
    /// 添加观察者到订阅列表，当配置重载时会收到通知。
    /// 
    /// # 参数
    /// - `observer`: 实现了 ConfigObserver trait 的观察者
    /// 
    /// # 示例
    /// ```
    /// struct MyObserver;
    /// 
    /// impl ConfigObserver for MyObserver {
    ///     fn on_config_changed(&self, new_config: &CraneConfig) {
    ///         println!("配置已更新！");
    ///     }
    /// }
    /// 
    /// manager.subscribe(Box::new(MyObserver));
    /// ```
    pub fn subscribe(&self, observer: Box<dyn ConfigObserver>) {
        if let Ok(mut observers) = self.observers.lock() {
            observers.push(observer);
            tracing::info!("新增配置观察者，当前观察者数量: {}", observers.len());
        } else {
            tracing::error!("无法锁定观察者列表，订阅失败");
        }
    }
    
    /// 获取配置源的引用（供 Repository 使用）
    /// 
    /// # 返回
    /// 配置数据源的 Arc 引用
    /// 
    /// # 示例
    /// ```
    /// let config_source = manager.get_config_source();
    /// ```
    pub fn get_config_source(&self) -> Arc<ConfigDataSource> {
        Arc::clone(&self.config_source)
    }
}

impl Default for ConfigManager {
    /// 提供默认实现
    /// 
    /// 如果初始化失败会 panic，建议使用 `new()` 方法以获得更好的错误处理。
    fn default() -> Self {
        Self::new().expect("配置管理器初始化失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    /// 测试用观察者
    struct TestObserver {
        call_count: Arc<AtomicUsize>,
    }
    
    impl ConfigObserver for TestObserver {
        fn on_config_changed(&self, _new_config: &CraneConfig) {
            self.call_count.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    #[test]
    fn test_config_manager_init() {
        let manager = ConfigManager::new();
        assert!(manager.is_ok(), "配置管理器初始化应该成功");
    }
    
    #[test]
    fn test_get_config() {
        let manager = ConfigManager::new().unwrap();
        let config = manager.get_config();
        assert!(config.is_ok(), "应该能获取配置");
        
        let config = config.unwrap();
        assert!(config.sensor_calibration.weight_scale_ad > 0.0);
        assert!(config.rated_load_table.entries.len() > 0);
    }
    
    #[test]
    fn test_get_config_version() {
        let manager = ConfigManager::new().unwrap();
        let version = manager.get_config_version();
        assert!(version.is_ok(), "应该能获取配置版本");
        assert_eq!(version.unwrap(), 1, "初始版本应为 1");
    }
    
    #[test]
    fn test_observer_notification() {
        let manager = ConfigManager::new().unwrap();
        let call_count = Arc::new(AtomicUsize::new(0));
        
        let observer = TestObserver {
            call_count: Arc::clone(&call_count),
        };
        
        manager.subscribe(Box::new(observer));
        
        // 重新加载配置应该触发观察者
        let result = manager.reload_config();
        assert!(result.is_ok(), "配置重载应该成功");
        
        assert_eq!(call_count.load(Ordering::SeqCst), 1, "观察者应该被调用一次");
    }
    
    #[test]
    fn test_multiple_observers() {
        let manager = ConfigManager::new().unwrap();
        let call_count1 = Arc::new(AtomicUsize::new(0));
        let call_count2 = Arc::new(AtomicUsize::new(0));
        
        let observer1 = TestObserver {
            call_count: Arc::clone(&call_count1),
        };
        let observer2 = TestObserver {
            call_count: Arc::clone(&call_count2),
        };
        
        manager.subscribe(Box::new(observer1));
        manager.subscribe(Box::new(observer2));
        
        // 重新加载配置应该触发所有观察者
        let _ = manager.reload_config();
        
        assert_eq!(call_count1.load(Ordering::SeqCst), 1, "观察者1应该被调用");
        assert_eq!(call_count2.load(Ordering::SeqCst), 1, "观察者2应该被调用");
    }
    
    #[test]
    fn test_get_config_source() {
        let manager = ConfigManager::new().unwrap();
        let config_source = manager.get_config_source();
        
        // 验证可以通过配置源获取配置
        let config = config_source.get_cached_config();
        assert!(config.is_ok(), "应该能通过配置源获取配置");
    }
    
    #[test]
    fn test_reload_increments_version() {
        let manager = ConfigManager::new().unwrap();
        let version1 = manager.get_config_version().unwrap();
        
        // 重新加载配置
        let _ = manager.reload_config();
        
        let version2 = manager.get_config_version().unwrap();
        assert_eq!(version2, version1 + 1, "重载后版本号应递增");
    }
    
    #[test]
    fn test_default_impl() {
        let manager = ConfigManager::default();
        let config = manager.get_config();
        assert!(config.is_ok(), "默认实现应该能获取配置");
    }
}
