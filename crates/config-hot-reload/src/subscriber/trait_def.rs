//! 配置订阅者 trait 定义
//!
//! 定义配置变更通知的订阅者接口。

use crate::{ConfigChange, ConfigSnapshot};
use async_trait::async_trait;

/// 配置订阅者 trait
///
/// 实现此 trait 的组件可以订阅配置变更通知，
/// 在配置更新后收到通知并应用新配置。
///
/// # 示例
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use config_hot_reload::{ConfigSubscriber, ConfigChange, ConfigSnapshot};
///
/// struct MyComponent;
///
/// #[async_trait]
/// impl ConfigSubscriber for MyComponent {
///     async fn on_config_changed(&self, change: ConfigChange, snapshot: ConfigSnapshot) {
///         println!("配置已更新: {:?}", change.file_type);
///         // 应用新配置...
///     }
///
///     fn name(&self) -> &str {
///         "MyComponent"
///     }
/// }
/// ```
#[async_trait]
pub trait ConfigSubscriber: Send + Sync {
    /// 配置变更通知回调
    ///
    /// 当配置更新成功后，配置管理器会调用此方法通知订阅者。
    ///
    /// # 参数
    ///
    /// * `change` - 配置变更信息，包含变更的配置类型、版本号和时间戳
    /// * `snapshot` - 更新后的完整配置快照
    ///
    /// # 注意
    ///
    /// - 此方法应该快速返回，避免阻塞配置管理器
    /// - 如果需要执行耗时操作，应该在独立的任务中执行
    /// - 此方法中的错误会被捕获并记录，不会影响其他订阅者
    async fn on_config_changed(&self, change: ConfigChange, snapshot: ConfigSnapshot);

    /// 获取订阅者名称
    ///
    /// 用于日志记录和调试，应该返回一个唯一标识此订阅者的名称。
    ///
    /// # 返回值
    ///
    /// 订阅者的名称字符串
    fn name(&self) -> &str;
}
