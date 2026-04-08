//! 文件监控模块
//!
//! 提供配置文件变更监控功能，自动检测文件修改并发送通知。

pub mod file_watcher;

pub use file_watcher::FileWatcher;
