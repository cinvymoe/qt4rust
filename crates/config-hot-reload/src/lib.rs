//! 配置热加载 crate
//!
//! 提供运行时配置文件监控、解析、验证和热加载功能。
//!
//! # 功能特性
//!
//! - 自动监控配置文件变化
//! - 支持 TOML 和 CSV 格式
//! - 配置有效性验证
//! - 原子性配置更新
//! - 观察者模式通知机制
//! - 异步非阻塞设计
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use config_hot_reload::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), HotReloadError> {
//!     // 创建配置管理器
//!     // let mut manager = HotReloadConfigManager::new("config".into())?;
//!     
//!     // 订阅配置变更
//!     // manager.subscribe(Box::new(MySubscriber));
//!     
//!     // 启动热加载服务
//!     // manager.start().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod manager;
pub mod parser;
pub mod subscriber;
pub mod subscribers;
pub mod types;
pub mod validator;
pub mod watcher;

// 重新导出常用类型
pub use error::{HotReloadError, ValidationError};
pub use manager::{ConfigCache, HotReloadConfigManager};
pub use parser::ConfigParser;
pub use subscriber::ConfigSubscriber;
pub use subscribers::{
    register_all_subscribers, AlarmDetectionSubscriber, DataProcessingSubscriber,
    LoggingConfigSubscriber, PipelineConfigSubscriber, SensorDataSourceSubscriber,
    SharedConfigRefs,
};
pub use types::{ConfigChange, ConfigFileEvent, ConfigFileType, ConfigSnapshot};
pub use validator::ConfigValidator;
pub use watcher::FileWatcher;

/// Prelude 模块，包含最常用的导入
pub mod prelude {
    pub use crate::error::{HotReloadError, ValidationError};
    pub use crate::manager::{ConfigCache, HotReloadConfigManager};
    pub use crate::parser::ConfigParser;
    pub use crate::subscriber::ConfigSubscriber;
    pub use crate::types::{ConfigChange, ConfigFileType, ConfigSnapshot};
    pub use crate::validator::ConfigValidator;
    pub use crate::watcher::FileWatcher;
}
