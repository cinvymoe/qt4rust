# crane-data-layer

数据层抽象库，提供 Repository 模式和数据管理功能。

## 功能特性

- **Repository 模式**: 统一的数据访问接口
- **DataSource 抽象**: 支持多种数据源（内存、文件等）
- **Cache 管理**: LRU 缓存策略
- **Validator 框架**: 数据验证支持
- **错误处理**: 统一的错误类型

## 使用示例

```rust
use crane_data_layer::prelude::*;

// 创建内存数据源
let data_source = MemoryDataSource::new();

// 创建 LRU 缓存
let cache = LRUCache::new(100);

// 使用 Repository
// let mut repo = BaseRepository::new(data_source);
```

## 文档

详细设计文档请参考: `doc/CRANE_DATA_LAYER_DESIGN.md`
