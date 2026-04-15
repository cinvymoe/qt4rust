use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::digital::{DigitalInputConfig, DigitalInputSourceFactory};
use crate::{DigitalInputSource, SensorError, SensorResult};

/// 数字输入源注册表（全局单例）
pub struct DigitalInputRegistry {
    factories: RwLock<HashMap<String, Arc<dyn DigitalInputSourceFactory>>>,
}

impl DigitalInputRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
        }
    }

    /// 获取全局实例
    pub fn global() -> &'static Self {
        static INSTANCE: std::sync::OnceLock<DigitalInputRegistry> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// 注册工厂
    pub fn register(&self, factory: Arc<dyn DigitalInputSourceFactory>) {
        let mut factories = self.factories.write().unwrap();
        let name = factory.name().to_string();
        factories.insert(name.clone(), factory);
        tracing::debug!("注册数字输入源工厂: {}", name);
    }

    /// 创建数字输入源
    pub fn create(&self, config: &DigitalInputConfig) -> SensorResult<Box<dyn DigitalInputSource>> {
        let factories = self.factories.read().unwrap();

        let factory = factories.get(&config.source_type).ok_or_else(|| {
            SensorError::ConfigError(format!("未知的数字输入源类型: {}", config.source_type))
        })?;

        factory.validate_config(config)?;
        factory.create(config)
    }

    /// 列出所有已注册的类型
    pub fn list_available(&self) -> Vec<String> {
        let factories = self.factories.read().unwrap();
        factories.keys().cloned().collect()
    }
}

impl Default for DigitalInputRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DigitalInputSource;

    struct MockFactory;

    impl DigitalInputSourceFactory for MockFactory {
        fn create(
            &self,
            _config: &DigitalInputConfig,
        ) -> SensorResult<Box<dyn DigitalInputSource>> {
            Ok(Box::new(MockDigitalInput))
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    struct MockDigitalInput;

    impl DigitalInputSource for MockDigitalInput {
        fn read(&self) -> SensorResult<(bool, bool)> {
            Ok((true, false))
        }

        fn source_name(&self) -> &str {
            "MockDigitalInput"
        }
    }

    #[test]
    fn test_register_and_create() {
        let registry = DigitalInputRegistry::new();
        registry.register(Arc::new(MockFactory));

        let config = DigitalInputConfig {
            source_type: "mock".to_string(),
            ..Default::default()
        };

        let source = registry.create(&config).unwrap();
        let (di0, di1) = source.read().unwrap();
        assert_eq!((di0, di1), (true, false));
    }

    #[test]
    fn test_unknown_source_type() {
        let registry = DigitalInputRegistry::new();

        let config = DigitalInputConfig {
            source_type: "unknown".to_string(),
            ..Default::default()
        };

        let result = registry.create(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_available() {
        let registry = DigitalInputRegistry::new();
        registry.register(Arc::new(MockFactory));

        let available = registry.list_available();
        assert!(available.contains(&"mock".to_string()));
    }
}
