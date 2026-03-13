# crane-data-layer 库设计文档

## 1. 概述

`crane-data-layer` 是一个通用的数据层抽象库，专为工业监控和 IoT 数据采集项目设计。它提供了 Repository 模式实现、数据源抽象、缓存管理和数据验证等核心功能。

## 2. 设计目标

### 2.1 核心目标
- **数据源抽象**: 统一的数据源接口，支持多种数据来源
- **Repository 模式**: 封装数据访问逻辑，提供清晰的 API
- **缓存管理**: 智能缓存策略，减少数据源访问
- **数据验证**: 内置数据验证框架
- **错误处理**: 统一的错误类型和处理机制
- **异步支持**: 支持同步和异步数据访问
- **可测试性**: 易于 mock 和单元测试

### 2.2 适用场景
- 工业设备监控系统
- IoT 数据采集平台
- 传感器数据管理
- 设备状态监控
- 历史数据查询

## 3. 架构设计

### 3.1 层次结构

```
┌─────────────────────────────────────────────────────────┐
│                   Application Layer                     │
│              (ViewModel, Business Logic)                │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   Repository Layer                      │
│         (DataRepository, CacheRepository)               │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   Data Source Layer                     │
│    (SensorSource, DatabaseSource, FileSource, etc.)     │
└─────────────────────────────────────────────────────────┘
```

### 3.2 核心组件

1. **Repository**: 数据仓库，统一数据访问接口
2. **DataSource**: 数据源抽象，定义数据获取方式
3. **Cache**: 缓存管理，提供多种缓存策略
4. **Validator**: 数据验证器，确保数据质量
5. **Error**: 错误处理，统一错误类型

## 4. 目录结构

```
crates/crane-data-layer/
├── Cargo.toml
├── README.md
├── LICENSE
├── CHANGELOG.md
```
├── examples/
│   ├── basic_repository.rs          # 基础 Repository 使用
│   ├── cached_repository.rs         # 带缓存的 Repository
│   ├── multi_source.rs              # 多数据源示例
│   └── custom_validator.rs          # 自定义验证器
├── src/
│   ├── lib.rs                       # 库入口
│   │
│   ├── repository/                  # Repository 模块
│   │   ├── mod.rs
│   │   ├── traits.rs                # Repository trait 定义
│   │   ├── base_repository.rs       # 基础 Repository 实现
│   │   ├── cached_repository.rs     # 带缓存的 Repository
│   │   └── composite_repository.rs  # 组合多个 Repository
│   │
│   ├── data_source/                 # Data Source 模块
│   │   ├── mod.rs
│   │   ├── traits.rs                # DataSource trait 定义
│   │   ├── memory_source.rs         # 内存数据源
│   │   ├── file_source.rs           # 文件数据源
│   │   └── mock_source.rs           # Mock 数据源（测试用）
│   │
│   ├── cache/                       # Cache 模块
│   │   ├── mod.rs
│   │   ├── traits.rs                # Cache trait 定义
│   │   ├── lru_cache.rs             # LRU 缓存策略
│   │   ├── ttl_cache.rs             # TTL 缓存策略
│   │   ├── memory_cache.rs          # 内存缓存
│   │   └── cache_policy.rs          # 缓存策略配置
│   │
│   ├── validator/                   # Validator 模块
│   │   ├── mod.rs
│   │   ├── traits.rs                # Validator trait 定义
│   │   ├── range_validator.rs       # 范围验证器
│   │   ├── composite_validator.rs   # 组合验证器
│   │   └── custom_validator.rs      # 自定义验证器基类
│   │
│   ├── error/                       # Error 模块
│   │   ├── mod.rs
│   │   ├── data_error.rs            # 数据层错误类型
│   │   └── result.rs                # Result 类型别名
│   │
│   └── utils/                       # 工具模块
│       ├── mod.rs
│       ├── retry.rs                 # 重试机制
│       └── metrics.rs               # 性能指标
│
└── tests/
    ├── repository_test.rs
    ├── cache_test.rs
    ├── validator_test.rs
    └── integration_test.rs
```

## 5. 核心 API 设计

### 5.1 Repository Trait

```rust
// src/repository/traits.rs

use std::future::Future;
use crate::error::DataResult;

/// Repository trait - 数据仓库抽象
pub trait Repository<T, K = String> {
    /// 根据 ID 获取单个数据
    fn get(&self, id: &K) -> DataResult<Option<T>>;
    
    /// 获取所有数据
    fn get_all(&self) -> DataResult<Vec<T>>;
    
    /// 保存数据
    fn save(&mut self, data: &T) -> DataResult<()>;
    
    /// 更新数据
    fn update(&mut self, id: &K, data: &T) -> DataResult<()>;
    
    /// 删除数据
    fn delete(&mut self, id: &K) -> DataResult<()>;
    
    /// 批量保存
    fn save_batch(&mut self, data: &[T]) -> DataResult<()> {
        for item in data {
            self.save(item)?;
        }
        Ok(())
    }
}

/// AsyncRepository trait - 异步数据仓库
pub trait AsyncRepository<T, K = String>: Send + Sync {
    /// 异步获取数据
    fn get_async(&self, id: &K) -> impl Future<Output = DataResult<Option<T>>> + Send;
    
    /// 异步保存数据
    fn save_async(&mut self, data: &T) -> impl Future<Output = DataResult<()>> + Send;
}
```

### 5.2 DataSource Trait

```rust
// src/data_source/traits.rs

use crate::error::DataResult;

/// DataSource trait - 数据源抽象
pub trait DataSource<T> {
    /// 读取数据
    fn read(&self) -> DataResult<T>;
    
    /// 写入数据
    fn write(&mut self, data: &T) -> DataResult<()>;
    
    /// 检查数据源是否可用
    fn is_available(&self) -> bool;
    
    /// 获取数据源名称
    fn name(&self) -> &str;
}

/// ReadOnlyDataSource - 只读数据源
pub trait ReadOnlyDataSource<T> {
    fn read(&self) -> DataResult<T>;
    fn is_available(&self) -> bool;
}

/// WriteOnlyDataSource - 只写数据源
pub trait WriteOnlyDataSource<T> {
    fn write(&mut self, data: &T) -> DataResult<()>;
    fn is_available(&self) -> bool;
}

/// StreamDataSource - 流式数据源（用于传感器等持续数据）
pub trait StreamDataSource<T> {
    /// 订阅数据流
    fn subscribe<F>(&mut self, callback: F) -> DataResult<()>
    where
        F: Fn(T) + Send + 'static;
    
    /// 取消订阅
    fn unsubscribe(&mut self) -> DataResult<()>;
}
```

### 5.3 Cache Trait

```rust
// src/cache/traits.rs

use std::time::Duration;
use crate::error::DataResult;

/// Cache trait - 缓存抽象
pub trait Cache<K, V> {
    /// 获取缓存数据
    fn get(&self, key: &K) -> Option<&V>;
    
    /// 设置缓存数据
    fn set(&mut self, key: K, value: V);
    
    /// 删除缓存数据
    fn remove(&mut self, key: &K) -> Option<V>;
    
    /// 清空缓存
    fn clear(&mut self);
    
    /// 获取缓存大小
    fn len(&self) -> usize;
    
    /// 检查缓存是否为空
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// TTLCache trait - 带过期时间的缓存
pub trait TTLCache<K, V>: Cache<K, V> {
    /// 设置带过期时间的缓存
    fn set_with_ttl(&mut self, key: K, value: V, ttl: Duration);
    
    /// 检查缓存是否过期
    fn is_expired(&self, key: &K) -> bool;
}

/// CachePolicy - 缓存策略
#[derive(Debug, Clone)]
pub struct CachePolicy {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 默认 TTL
    pub default_ttl: Option<Duration>,
    /// 是否启用缓存
    pub enabled: bool,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Some(Duration::from_secs(300)), // 5 分钟
            enabled: true,
        }
    }
}
```

### 5.4 Validator Trait

```rust
// src/validator/traits.rs

use crate::error::DataResult;

/// Validator trait - 数据验证器
pub trait Validator<T> {
    /// 验证数据
    fn validate(&self, data: &T) -> DataResult<()>;
    
    /// 验证并返回详细错误信息
    fn validate_detailed(&self, data: &T) -> ValidationResult {
        match self.validate(data) {
            Ok(_) => ValidationResult::valid(),
            Err(e) => ValidationResult::invalid(vec![e.to_string()]),
        }
    }
}

/// ValidationResult - 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: vec![],
        }
    }
    
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
    
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }
}
```

## 6. 实现示例

### 6.1 基础 Repository 实现

```rust
// src/repository/base_repository.rs

use std::collections::HashMap;
use crate::repository::traits::Repository;
use crate::data_source::traits::DataSource;
use crate::validator::traits::Validator;
use crate::error::{DataResult, DataError};

/// BaseRepository - 基础 Repository 实现
pub struct BaseRepository<T, K, S>
where
    K: Eq + std::hash::Hash + Clone,
    S: DataSource<Vec<T>>,
{
    data_source: S,
    validator: Option<Box<dyn Validator<T>>>,
    cache: HashMap<K, T>,
}

impl<T, K, S> BaseRepository<T, K, S>
where
    K: Eq + std::hash::Hash + Clone,
    S: DataSource<Vec<T>>,
    T: Clone,
{
    pub fn new(data_source: S) -> Self {
        Self {
            data_source,
            validator: None,
            cache: HashMap::new(),
        }
    }
    
    pub fn with_validator(mut self, validator: Box<dyn Validator<T>>) -> Self {
        self.validator = Some(validator);
        self
    }
    
    fn validate(&self, data: &T) -> DataResult<()> {
        if let Some(validator) = &self.validator {
            validator.validate(data)?;
        }
        Ok(())
    }
}

impl<T, K, S> Repository<T, K> for BaseRepository<T, K, S>
where
    K: Eq + std::hash::Hash + Clone,
    S: DataSource<Vec<T>>,
    T: Clone + HasId<K>,
{
    fn get(&self, id: &K) -> DataResult<Option<T>> {
        // 先查缓存
        if let Some(data) = self.cache.get(id) {
            return Ok(Some(data.clone()));
        }
        
        // 从数据源读取
        let all_data = self.data_source.read()?;
        Ok(all_data.into_iter().find(|item| item.id() == id))
    }
    
    fn get_all(&self) -> DataResult<Vec<T>> {
        self.data_source.read()
    }
    
    fn save(&mut self, data: &T) -> DataResult<()> {
        // 验证数据
        self.validate(data)?;
        
        // 保存到数据源
        let mut all_data = self.data_source.read().unwrap_or_default();
        all_data.push(data.clone());
        self.data_source.write(&all_data)?;
        
        // 更新缓存
        self.cache.insert(data.id().clone(), data.clone());
        
        Ok(())
    }
    
    fn update(&mut self, id: &K, data: &T) -> DataResult<()> {
        self.validate(data)?;
        
        let mut all_data = self.data_source.read()?;
        if let Some(item) = all_data.iter_mut().find(|item| item.id() == id) {
            *item = data.clone();
            self.data_source.write(&all_data)?;
            self.cache.insert(id.clone(), data.clone());
            Ok(())
        } else {
            Err(DataError::NotFound(format!("Data with id {:?} not found", id)))
        }
    }
    
    fn delete(&mut self, id: &K) -> DataResult<()> {
        let mut all_data = self.data_source.read()?;
        all_data.retain(|item| item.id() != id);
        self.data_source.write(&all_data)?;
        self.cache.remove(id);
        Ok(())
    }
}

/// HasId trait - 用于获取数据的 ID
pub trait HasId<K> {
    fn id(&self) -> &K;
}
```

### 6.2 带缓存的 Repository

```rust
// src/repository/cached_repository.rs

use std::sync::{Arc, Mutex};
use crate::repository::traits::Repository;
use crate::cache::traits::Cache;
use crate::data_source::traits::DataSource;
use crate::error::DataResult;

/// CachedRepository - 带缓存的 Repository
pub struct CachedRepository<T, K, S, C>
where
    K: Eq + std::hash::Hash + Clone,
    S: DataSource<Vec<T>>,
    C: Cache<K, T>,
{
    data_source: S,
    cache: Arc<Mutex<C>>,
    _phantom: std::marker::PhantomData<(T, K)>,
}

impl<T, K, S, C> CachedRepository<T, K, S, C>
where
    K: Eq + std::hash::Hash + Clone,
    S: DataSource<Vec<T>>,
    C: Cache<K, T>,
    T: Clone + HasId<K>,
{
    pub fn new(data_source: S, cache: C) -> Self {
        Self {
            data_source,
            cache: Arc::new(Mutex::new(cache)),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, K, S, C> Repository<T, K> for CachedRepository<T, K, S, C>
where
    K: Eq + std::hash::Hash + Clone,
    S: DataSource<Vec<T>>,
    C: Cache<K, T>,
    T: Clone + HasId<K>,
{
    fn get(&self, id: &K) -> DataResult<Option<T>> {
        // 先查缓存
        if let Ok(cache) = self.cache.lock() {
            if let Some(data) = cache.get(id) {
                return Ok(Some(data.clone()));
            }
        }
        
        // 缓存未命中，从数据源读取
        let all_data = self.data_source.read()?;
        let result = all_data.into_iter().find(|item| item.id() == id);
        
        // 更新缓存
        if let (Some(ref data), Ok(mut cache)) = (&result, self.cache.lock()) {
            cache.set(id.clone(), data.clone());
        }
        
        Ok(result)
    }
    
    fn get_all(&self) -> DataResult<Vec<T>> {
        self.data_source.read()
    }
    
    fn save(&mut self, data: &T) -> DataResult<()> {
        let mut all_data = self.data_source.read().unwrap_or_default();
        all_data.push(data.clone());
        self.data_source.write(&all_data)?;
        
        // 更新缓存
        if let Ok(mut cache) = self.cache.lock() {
            cache.set(data.id().clone(), data.clone());
        }
        
        Ok(())
    }
    
    fn update(&mut self, id: &K, data: &T) -> DataResult<()> {
        let mut all_data = self.data_source.read()?;
        if let Some(item) = all_data.iter_mut().find(|item| item.id() == id) {
            *item = data.clone();
            self.data_source.write(&all_data)?;
            
            // 更新缓存
            if let Ok(mut cache) = self.cache.lock() {
                cache.set(id.clone(), data.clone());
            }
            
            Ok(())
        } else {
            Err(DataError::NotFound(format!("Data not found")))
        }
    }
    
    fn delete(&mut self, id: &K) -> DataResult<()> {
        let mut all_data = self.data_source.read()?;
        all_data.retain(|item| item.id() != id);
        self.data_source.write(&all_data)?;
        
        // 清除缓存
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(id);
        }
        
        Ok(())
    }
}
```

### 6.3 LRU 缓存实现

```rust
// src/cache/lru_cache.rs

use std::coll// src/cache/lru_cache.rs

use std::collections::{HashMap, VecDeque};
use crate::cache::traits::Cache;

/// LRUCache - LRU 缓存实现
pub struct LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    capacity: usize,
    cache: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            order: VecDeque::new(),
        }
    }
}

impl<K, V> Cache<K, V> for LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }
    
    fn set(&mut self, key: K, value: V) {
        // 如果 key 已存在，先移除旧的顺序记录
        if self.cache.contains_key(&key) {
            self.order.retain(|k| k != &key);
        }
        
        // 如果缓存已满，移除最久未使用的项
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            if let Some(oldest_key) = self.order.pop_front() {
                self.cache.remove(&oldest_key);
            }
        }
        
        // 插入新数据
        self.cache.insert(key.clone(), value);
        self.order.push_back(key);
    }
    
    fn remove(&mut self, key: &K) -> Option<V> {
        self.order.retain(|k| k != key);
        self.cache.remove(key)
    }
    
    fn clear(&mut self) {
        self.cache.clear();
        self.order.clear();
    }
    
    fn len(&self) -> usize {
        self.cache.len()
    }
}
```

### 6.4 范围验证器

```rust
// src/validator/range_validator.rs

use crate::validator::traits::Validator;
use crate::error::{DataResult, DataError};

/// RangeValidator - 范围验证器
pub struct RangeValidator<T>
where
    T: PartialOrd,
{
    min: Option<T>,
    max: Option<T>,
    field_name: String,
}

impl<T> RangeValidator<T>
where
    T: PartialOrd,
{
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            min: None,
            max: None,
            field_name: field_name.into(),
        }
    }
    
    pub fn with_min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }
    
    pub fn with_max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }
    
    pub fn validate_value(&self, value: &T) -> DataResult<()> {
        if let Some(ref min) = self.min {
            if value < min {
                return Err(DataError::ValidationError(
                    format!("{} is below minimum value", self.field_name)
                ));
            }
        }
        
        if let Some(ref max) = self.max {
            if value > max {
                return Err(DataError::ValidationError(
                    format!("{} exceeds maximum value", self.field_name)
                ));
            }
        }
        
        Ok(())
    }
}

// 为具有可提取字段的类型实现 Validator
impl<T, V> Validator<T> for RangeValidator<V>
where
    V: PartialOrd,
    T: HasField<V>,
{
    fn validate(&self, data: &T) -> DataResult<()> {
        self.validate_value(data.get_field())
    }
}

/// HasField trait - 用于提取字段值
pub trait HasField<V> {
    fn get_field(&self) -> &V;
}
```

### 6.5 错误类型定义

```rust
// src/error/data_error.rs

use std::fmt;

/// DataError - 数据层错误类型
#[derive(Debug, Clone)]
pub enum DataError {
    /// 数据源不可用
    SourceUnavailable(String),
    
    /// 数据未找到
    NotFound(String),
    
    /// 数据验证失败
    ValidationError(String),
    
    /// 缓存错误
    CacheError(String),
    
    /// 序列化/反序列化错误
    SerializationError(String),
    
    /// I/O 错误
    IoError(String),
    
    /// 超时错误
    Timeout(String),
    
    /// 权限错误
    PermissionDenied(String),
    
    /// 未知错误
    Unknown(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SourceUnavailable(msg) => write!(f, "数据源不可用: {}", msg),
            Self::NotFound(msg) => write!(f, "数据未找到: {}", msg),
            Self::ValidationError(msg) => write!(f, "数据验证失败: {}", msg),
            Self::CacheError(msg) => write!(f, "缓存错误: {}", msg),
            Self::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
            Self::IoError(msg) => write!(f, "I/O 错误: {}", msg),
            Self::Timeout(msg) => write!(f, "超时: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "权限不足: {}", msg),
            Self::Unknown(msg) => write!(f, "未知错误: {}", msg),
        }
    }
}

impl std::error::Error for DataError {}

// src/error/result.rs

/// DataResult - 数据层 Result 类型别名
pub type DataResult<T> = Result<T, DataError>;
```

## 7. 使用示例

### 7.1 基础使用

```rust
// examples/basic_repository.rs

use crane_data_layer::prelude::*;

#[derive(Debug, Clone)]
struct SensorData {
    id: String,
    temperature: f64,
    humidity: f64,
}

impl HasId<String> for SensorData {
    fn id(&self) -> &String {
        &self.id
    }
}

fn main() -> DataResult<()> {
    // 创建内存数据源
    let data_source = MemoryDataSource::new();
    
    // 创建 Repository
    let mut repo = BaseRepository::new(data_source);
    
    // 保存数据
    let sensor = SensorData {
        id: "sensor_001".to_string(),
        temperature: 25.5,
        humidity: 60.0,
    };
    repo.save(&sensor)?;
    
    // 获取数据
    if let Some(data) = repo.get(&"sensor_001".to_string())? {
        println!("Temperature: {}", data.temperature);
    }
    
    // 获取所有数据
    let all_data = repo.get_all()?;
    println!("Total sensors: {}", all_data.len());
    
    Ok(())
}
```

### 7.2 带缓存和验证器

```rust
// examples/cached_repository.rs

use crane_data_layer::prelude::*;

#[derive(Debug, Clone)]
struct SensorReading {
    id: String,
    value: f64,
}

impl HasId<String> for SensorReading {
    fn id(&self) -> &String {
        &self.id
    }
}

impl HasField<f64> for SensorReading {
    fn get_field(&self) -> &f64 {
        &self.value
    }
}

fn main() -> DataResult<()> {
    // 创建数据源
    let data_source = MemoryDataSource::new();
    
    // 创建 LRU 缓存（容量 100）
    let cache = LRUCache::new(100);
    
    // 创建带缓存的 Repository
    let mut repo = CachedRepository::new(data_source, cache);
    
    // 创建范围验证器（值必须在 0-100 之间）
    let validator = RangeValidator::new("value")
        .with_min(0.0)
        .with_max(100.0);
    
    // 保存数据（会自动验证）
    let reading = SensorReading {
        id: "reading_001".to_string(),
        value: 75.5,
    };
    
    repo.save(&reading)?;
    
    // 第一次获取：从数据源读取
    let data1 = repo.get(&"reading_001".to_string())?;
    
    // 第二次获取：从缓存读取（更快）
    let data2 = repo.get(&"reading_001".to_string())?;
    
    Ok(())
}
```

### 7.3 多数据源组合

```rust
// examples/multi_source.rs

use crane_data_layer::prelude::*;

/// CompositeDataSource - 组合多个数据源
pub struct CompositeDataSource<T> {
    primary: Box<dyn DataSource<T>>,
    fallback: Box<dyn DataSource<T>>,
}

impl<T> CompositeDataSource<T> {
    pub fn new(
        primary: Box<dyn DataSource<T>>,
        fallback: Box<dyn DataSource<T>>,
    ) -> Self {
        Self { primary, fallback }
    }
}

impl<T> DataSource<T> for CompositeDataSource<T> {
    fn read(&self) -> DataResult<T> {
        // 先尝试主数据源
        match self.primary.read() {
            Ok(data) => Ok(data),
            Err(_) => {
                // 主数据源失败，使用备用数据源
                eprintln!("[WARN] Primary source failed, using fallback");
                self.fallback.read()
            }
        }
    }
    
    fn write(&mut self, data: &T) -> DataResult<()> {
        // 同时写入两个数据源
        self.primary.write(data)?;
        self.fallback.write(data)?;
        Ok(())
    }
    
    fn is_available(&self) -> bool {
        self.primary.is_available() || self.fallback.is_available()
    }
    
    fn name(&self) -> &str {
        "CompositeDataSource"
    }
}
```

## 8. 在 Crane 项目中的应用

### 8.1 传感器数据 Repository

```rust
// src/repositories/crane_data_repository.rs (主应用)

use crane_data_layer::prelude::*;
use crate::models::sensor_data::SensorData;

pub struct CraneDataRepository {
    sensor_repo: Box<dyn Repository<SensorData, String>>,
}

impl CraneDataRepository {
    pub fn new() -> Self {
        // 创建传感器数据源（使用 sensor-simulator）
        let sensor_source = SensorDataSourceAdapter::new();
        
        // 创建 LRU 缓存
        let cache = LRUCache::new(100);
        
        // 创建带缓存的 Repository
        let sensor_repo = CachedRepository::new(sensor_source, cache);
        
        Self {
            sensor_repo: Box::new(sensor_repo),
        }
    }
    
    pub fn get_latest_sensor_data(&self) -> Result<SensorData, String> {
        self.sensor_repo
            .get(&"latest".to_string())
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No sensor data available".to_string())
    }
    
    pub fn save_sensor_data(&mut self, data: &SensorData) -> Result<(), String> {
        self.sensor_repo
            .save(data)
            .map_err(|e| e.to_string())
    }
}

/// SensorDataSourceAdapter - 适配器，将 sensor-simulator 适配为 DataSource
struct SensorDataSourceAdapter {
    simulator: sensor_simulator::SineSimulator,
}

impl SensorDataSourceAdapter {
    fn new() -> Self {
        let config = sensor_simulator::SimulatorConfig::default();
        Self {
            simulator: sensor_simulator::SineSimulator::new(config),
        }
    }
}

impl DataSource<Vec<SensorData>> for SensorDataSourceAdapter {
    fn read(&self) -> DataResult<Vec<SensorData>> {
        // 从模拟器读取数据
        let simulated_value = self.simulator.generate();
        
        let sensor_data = SensorData {
            load: simulated_value,
            rated_load: 25.0,
            radius: 10.0,
            angle: 60.0,
            boom_length: 22.6,
        };
        
        Ok(vec![sensor_data])
    }
    
    fn write(&mut self, _data: &Vec<SensorData>) -> DataResult<()> {
        // 传感器数据通常是只读的
        Err(DataError::PermissionDenied(
            "Sensor data is read-only".to_string()
        ))
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn name(&self) -> &str {
        "SensorDataSource"
    }
}
```

### 8.2 历史数据 Repository

```rust
// src/repositories/history_repository.rs (主应用)

use crane_data_layer::prelude::*;
use crate::models::alarm_record::AlarmRecord;

pub struct HistoryRepository {
    alarm_repo: Box<dyn Repository<AlarmRecord, i64>>,
}

impl HistoryRepository {
    pub fn new() -> Self {
        // 创建 SQLite 数据源（未来实现）
        let db_source = SqliteDataSource::new("crane_history.db");
        
        // 创建 Repository
        let alarm_repo = BaseRepository::new(db_source);
        
        Self {
            alarm_repo: Box::new(alarm_repo),
        }
    }
    
    pub fn save_alarm(&mut self, alarm: &AlarmRecord) -> Result<(), String> {
        self.alarm_repo
            .save(alarm)
            .map_err(|e| e.to_string())
    }
    
    pub fn get_recent_alarms(&self, limit: usize) -> Result<Vec<AlarmRecord>, String> {
        let all_alarms = self.alarm_repo
            .get_all()
            .map_err(|e| e.to_string())?;
        
        Ok(all_alarms.into_iter().take(limit).collect())
    }
}
```

## 9. 高级特性

### 9.1 重试机制

```rust
// src/utils/retry.rs

use std::time::Duration;
use std::thread;
use crate::error::DataResult;

/// RetryPolicy - 重试策略
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub delay: Duration,
    pub exponential_backoff: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: Duration::from_millis(100),
            exponential_backoff: true,
        }
    }
}

/// 带重试的执行函数
pub fn retry_with_policy<F, T>(
    policy: &RetryPolicy,
    mut operation: F,
) -> DataResult<T>
where
    F: FnMut() -> DataResult<T>,
{
    let mut attempts = 0;
    let mut delay = policy.delay;
    
    loop {
        attempts += 1;
        
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempts >= policy.max_attempts {
                    return Err(e);
                }
                
                eprintln!(
                    "[RETRY] Attempt {}/{} failed: {}",
                    attempts, policy.max_attempts, e
                );
                
                thread::sleep(delay);
                
                if policy.exponential_backoff {
                    delay *= 2;
                }
            }
        }
    }
}

/// RetryableDataSource - 带重试的数据源包装器
pub struct RetryableDataSource<T, S>
where
    S: DataSource<T>,
{
    inner: S,
    policy: RetryPolicy,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, S> RetryableDataSource<T, S>
where
    S: DataSource<T>,
{
    pub fn new(inner: S, policy: RetryPolicy) -> Self {
        Self {
            inner,
            policy,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, S> DataSource<T> for RetryableDataSource<T, S>
where
    S: DataSource<T>,
{
    fn read(&self) -> DataResult<T> {
        retry_with_policy(&self.policy, || self.inner.read())
    }
    
    fn write(&mut self, data: &T) -> DataResult<()> {
        retry_with_policy(&self.policy, || self.inner.write(data))
    }
    
    fn is_available(&self) -> bool {
        self.inner.is_available()
    }
    
    fn name(&self) -> &str {
        self.inner.name()
    }
}
```

### 9.2 性能指标

```rust
// src/utils/metrics.rs

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/// Metrics - 性能指标收集
#[derive(Debug, Clone)]
pub struct Metrics {
    pub total_reads: u64,
    pub total_writes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_read_time: Duration,
    pub avg_write_time: Duration,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_reads: 0,
            total_writes: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_read_time: Duration::ZERO,
            avg_write_time: Duration::ZERO,
        }
    }
}

impl Metrics {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// MetricsCollector - 指标收集器
pub struct MetricsCollector {
    metrics: Arc<Mutex<Metrics>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Metrics::default())),
        }
    }
    
    pub fn record_read(&self, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_reads += 1;
            metrics.avg_read_time = 
                (metrics.avg_read_time * (metrics.total_reads - 1) as u32 + duration)
                / metrics.total_reads as u32;
        }
    }
    
    pub fn record_write(&self, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_writes += 1;
            metrics.avg_write_time = 
                (metrics.avg_write_time * (metrics.total_writes - 1) as u32 + duration)
                / metrics.total_writes as u32;
        }
    }
    
    pub fn record_cache_hit(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.cache_hits += 1;
        }
    }
    
    pub fn record_cache_miss(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.cache_misses += 1;
        }
    }
    
    pub fn get_metrics(&self) -> Metrics {
        self.metrics.lock().unwrap().clone()
    }
}
```

## 10. 测试策略

### 10.1 单元测试

```rust
// tests/repository_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_repository_save_and_get() {
        let data_source = MemoryDataSource::new();
        let mut repo = BaseRepository::new(data_source);
        
        let data = TestData {
            id: "test_001".to_string(),
            value: 42,
        };
        
        // 保存数据
        assert!(repo.save(&data).is_ok());
        
        // 获取数据
        let result = repo.get(&"test_001".to_string()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, 42);
    }
    
    #[test]
    fn test_repository_update() {
        let data_source = MemoryDataSource::new();
        let mut repo = BaseRepository::new(data_source);
        
        let data = TestData {
            id: "test_001".to_string(),
            value: 42,
        };
        repo.save(&data).unwrap();
        
        // 更新数据
        let updated_data = TestData {
            id: "test_001".to_string(),
            value: 100,
        };
        assert!(repo.update(&"test_001".to_string(), &updated_data).is_ok());
        
        // 验证更新
        let result = repo.get(&"test_001".to_string()).unwrap().unwrap();
        assert_eq!(result.value, 100);
    }
    
    #[test]
    fn test_repository_delete() {
        let data_source = MemoryDataSource::new();
        let mut repo = BaseRepository::new(data_source);
        
        let data = TestData {
            id: "test_001".to_string(),
            value: 42,
        };
        repo.save(&data).unwrap();
        
        // 删除数据
        assert!(repo.delete(&"test_001".to_string()).is_ok());
        
        // 验证删除
        let result = repo.get(&"test_001".to_string()).unwrap();
        assert!(result.is_none());
    }
}
```

### 10.2 缓存测试

```rust
// tests/cache_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LRUCache::new(3);
        
        cache.set("key1", "value1");
        cache.set("key2", "value2");
        cache.set("key3", "value3");
        
        // 缓存已满，添加新项会驱逐最久未使用的
        cache.set("key4", "value4");
        
        assert!(cache.get(&"key1").is_none()); // key1 被驱逐
        assert!(cache.get(&"key2").is_some());
        assert!(cache.get(&"key3").is_some());
        assert!(cache.get(&"key4").is_some());
    }
    
    #[test]
    fn test_cache_hit_rate() {
        let data_source = MemoryDataSource::new();
        let cache = LRUCache::new(10);
        let mut repo = CachedRepository::new(data_source, cache);
        
        let data = TestData {
            id: "test_001".to_string(),
            value: 42,
        };
        repo.save(&data).unwrap();
        
        // 第一次获取：缓存未命中
        repo.get(&"test_001".to_string()).unwrap();
        
        // 第二次获取：缓存命中
        repo.get(&"test_001".to_string()).unwrap();
        
        // 验证缓存工作正常
    }
}
```

### 10.3 验证器测试

```rust
// tests/validator_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_range_validator() {
        let validator = RangeValidator::new("temperature")
            .with_min(0.0)
            .with_max(100.0);
        
        // 有效值
        assert!(validator.validate_value(&50.0).is_ok());
        
        // 低于最小值
        assert!(validator.validate_value(&-10.0).is_err());
        
        // 高于最大值
        assert!(validator.validate_value(&150.0).is_err());
    }
    
    #[test]
    fn test_composite_validator() {
        let validator1 = RangeValidator::new("field1")
            .with_min(0.0)
            .with_max(100.0);
        
        let validator2 = RangeValidator::new("field2")
            .with_min(0.0)
            .with_max(50.0);
        
        let composite = CompositeValidator::new()
            .add(Box::new(validator1))
            .add(Box::new(validator2));
        
        // 测试组合验证器
    }
}
```

## 11. 配置和依赖

### 11.1 Cargo.toml

```toml
[package]
name = "crane-data-layer"
version = "0.1.0"
edition = "2021"
authors = ["Crane Team"]
description = "Data layer abstraction for industrial monitoring and IoT projects"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-org/crane-data-layer"
keywords = ["data", "repository", "cache", "iot", "industrial"]
categories = ["data-structures", "caching"]

[dependencies]
# 核心依赖
thiserror = "1.0"           # 错误处理
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }

# 异步支持（可选）
tokio = { version = "1.0", features = ["full"], optional = true }
async-trait = { version = "0.1", optional = true }

[dev-dependencies]
proptest = "1.4"            # 属性测试
criterion = "0.5"           # 性能测试

[features]
default = []
serde = ["dep:serde", "dep:serde_json"]
async = ["dep:tokio", "dep:async-trait"]
full = ["serde", "async"]

[[example]]
name = "basic_repository"
path = "examples/basic_repository.rs"

[[example]]
name = "cached_repository"
path = "examples/cached_repository.rs"

[[example]]
name = "multi_source"
path = "examples/multi_source.rs"
```

### 11.2 lib.rs

```rust
// src/lib.rs

//! # crane-data-layer
//!
//! 通用数据层抽象库，提供 Repository 模式、缓存管理和数据验证。
//!
//! ## 特性
//!
//! - Repository 模式实现
//! - 多种缓存策略（LRU、TTL）
//! - 数据验证框架
//! - 数据源抽象
//! - 错误处理
//!
//! ## 快速开始
//!
//! ```rust
//! use crane_data_layer::prelude::*;
//!
//! // 创建 Repository
//! let data_source = MemoryDataSource::new();
//! let mut repo = BaseRepository::new(data_source);
//!
//! // 保存和获取数据
//! repo.save(&my_data)?;
//! let data = repo.get(&id)?;
//! ```

pub mod repository;
pub mod data_source;
pub mod cache;
pub mod validator;
pub mod error;
pub mod utils;

/// Prelude - 常用类型导出
pub mod prelude {
    pub use crate::repository::traits::*;
    pub use crate::repository::base_repository::*;
    pub use crate::repository::cached_repository::*;
    
    pub use crate::data_source::traits::*;
    pub use crate::data_source::memory_source::*;
    
    pub use crate::cache::traits::*;
    pub use crate::cache::lru_cache::*;
    pub use crate::cache::ttl_cache::*;
    
    pub use crate::validator::traits::*;
    pub use crate::validator::range_validator::*;
    
    pub use crate::error::*;
}
```

## 12. 最佳实践

### 12.1 Repository 设计原则

1. **单一职责**: 每个 Repository 只负责一种数据类型
2. **接口隔离**: 使用 trait 定义清晰的接口
3. **依赖注入**: 通过构造函数注入数据源和缓存
4. **错误处理**: 使用 Result 类型，提供详细错误信息

### 12.2 缓存策略选择

- **LRU**: 适用于访问模式不确定的场景
- **TTL**: 适用于数据有时效性的场景
- **组合策略**: 同时使用 LRU + TTL

### 12.3 数据验证

- **早期验证**: 在数据进入系统时立即验证
- **组合验证器**: 使用多个验证器组合复杂规则
- **详细错误**: 提供清晰的验证失败原因

### 12.4 性能优化

- **批量操作**: 使用 `save_batch` 减少 I/O
- **缓存预热**: 启动时加载常用数据
- **异步操作**: 对于耗时操作使用异步 API
- **指标监控**: 使用 MetricsCollector 监控性能

## 13. 与其他库的集成

### 13.1 与 sensor-simulator 集成

```rust
use sensor_simulator::SineSimulator;
use crane_data_layer::prelude::*;

struct SensorDataSourceAdapter {
    simulator: SineSimulator,
}

impl DataSource<f64> for SensorDataSourceAdapter {
    fn read(&self) -> DataResult<f64> {
        Ok(self.simulator.generate())
    }
    // ...
}
```

### 13.2 与 qt-threading-utils 集成

```rust
use qt_threading_utils::collector::DataCollector;
use crane_data_layer::prelude::*;

// 在后台线程定期保存数据
let collector = DataCollector::new(
    Duration::from_secs(1),
    move || {
        if let Ok(data) = sensor_source.read() {
            repo.save(&data).ok();
        }
    }
);
```

## 14. 路线图

### 14.1 第一版本 (v0.1.0)
- ✅ 基础 Repository trait
- ✅ 内存数据源
- ✅ LRU 缓存
- ✅ 范围验证器
- ✅ 错误处理

### 14.2 第二版本 (v0.2.0)
- ⬜ 异步 Repository 支持
- ⬜ TTL 缓存实现
- ⬜ 文件数据源
- ⬜ 更多验证器类型

### 14.3 第三版本 (v0.3.0)
- ⬜ SQLite 数据源
- ⬜ 事务支持
- ⬜ 数据迁移工具
- ⬜ 性能优化

## 15. 总结

`crane-data-layer` 提供了一个强大而灵活的数据层抽象，具有以下优势：

- **通用性**: 适用于多种工业监控和 IoT 项目
- **可扩展**: 易于添加新的数据源和缓存策略
- **可测试**: 清晰的接口便于单元测试和 mock
- **高性能**: 内置缓存和性能监控
- **类型安全**: Rust 类型系统保证数据正确性

通过使用这个库，可以大大简化数据访问层的开发，提高代码质量和可维护性。
