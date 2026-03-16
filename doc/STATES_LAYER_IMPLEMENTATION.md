# States 层实现文档

## 概述

本文档记录了 Crane 监控系统 MVI 架构中 States 层的实现细节。

## 实现日期

2026-03-16

## 文件结构

```
src/states/
├── mod.rs                    # 模块入口，导出所有状态类型
├── monitoring_state.rs       # 监控视图状态（119 行）
├── chart_state.rs           # 图表视图状态（173 行）
├── alarm_state.rs           # 报警记录状态（293 行）
└── common_state.rs          # 通用状态定义（287 行）

总计：885 行代码
```

## 各模块详细说明

### 1. monitoring_state.rs - 监控视图状态

**功能**: 定义实时监控界面的状态

**核心结构**:
```rust
pub struct MonitoringState {
    pub current_load: f64,           // 当前载荷（吨）
    pub rated_load: f64,             // 额定载荷（吨）
    pub working_radius: f64,         // 工作半径（米）
    pub boom_angle: f64,             // 吊臂角度（度）
    pub boom_length: f64,            // 臂长（米）
    pub moment_percentage: f64,      // 力矩百分比
    pub is_danger: bool,             // 是否处于危险状态
    pub sensor_connected: bool,      // 传感器连接状态
    pub error_message: Option<String>, // 错误信息
    pub last_update_time: SystemTime,  // 最后更新时间
}
```

**辅助方法**:
- `new()`: 创建新的监控状态
- `should_alarm()`: 检查是否需要报警（≥90%）
- `alarm_level()`: 获取报警级别（0: 正常, 1: 预警, 2: 危险）

**测试覆盖**:
- ✅ 默认状态测试
- ✅ 报警判断测试
- ✅ 报警级别测试

### 2. chart_state.rs - 图表视图状态

**功能**: 管理历史数据图表的状态

**核心结构**:
```rust
pub struct ChartDataPoint {
    pub timestamp: SystemTime,       // 时间戳
    pub load: f64,                   // 载荷值（吨）
    pub moment_percentage: f64,      // 力矩百分比
    pub radius: f64,                 // 工作半径（米）
}

pub struct ChartState {
    pub data_points: Vec<ChartDataPoint>,  // 历史数据点列表
    pub time_range: u32,                   // 显示时间范围（秒）
    pub is_loading: bool,                  // 是否正在加载
    pub error_message: Option<String>,     // 错误信息
    pub max_data_points: usize,            // 最大数据点数量（默认100）
}
```

**辅助方法**:
- `new()`: 创建新的图表状态
- `add_data_point()`: 添加数据点（自动限制数量）
- `clear_data_points()`: 清空数据点
- `get_data_points_in_range()`: 获取指定时间范围内的数据
- `max_load()`: 获取最大载荷值
- `average_load()`: 获取平均载荷值

**测试覆盖**:
- ✅ 默认状态测试
- ✅ 添加数据点测试
- ✅ 数据点数量限制测试
- ✅ 统计功能测试（最大值、平均值）

### 3. alarm_state.rs - 报警记录状态

**功能**: 管理报警记录列表和过滤状态

**核心结构**:
```rust
pub enum AlarmType {
    Warning,  // 预警（90-100%）
    Danger,   // 危险（>100%）
}

pub struct AlarmRecord {
    pub id: u64,                     // 报警 ID
    pub timestamp: SystemTime,       // 报警时间
    pub alarm_type: AlarmType,       // 报警类型
    pub moment_percentage: f64,      // 力矩百分比
    pub current_load: f64,           // 当前载荷
    pub working_radius: f64,         // 工作半径
    pub boom_angle: f64,             // 吊臂角度
    pub is_read: bool,               // 是否已读
}

pub struct AlarmState {
    pub records: Vec<AlarmRecord>,           // 报警记录列表
    pub selected_record_id: Option<u64>,     // 当前选中的记录 ID
    pub filter_type: Option<AlarmType>,      // 过滤器：报警类型
    pub show_unread_only: bool,              // 是否只显示未读
    pub is_loading: bool,                    // 是否正在加载
    pub error_message: Option<String>,       // 错误信息
    pub total_count: usize,                  // 总记录数
}
```

**辅助方法**:
- `new()`: 创建新的报警状态
- `add_record()`: 添加报警记录
- `unread_count()`: 获取未读记录数量
- `danger_count()`: 获取危险级别记录数量
- `warning_count()`: 获取预警级别记录数量
- `mark_all_as_read()`: 标记所有记录为已读
- `find_record()`: 根据 ID 查找记录
- `filtered_records()`: 获取过滤后的记录
- `clear_records()`: 清空所有记录

**测试覆盖**:
- ✅ 报警类型测试
- ✅ 报警记录测试
- ✅ 报警状态测试
- ✅ 标记已读测试
- ✅ 过滤功能测试
- ✅ 查找记录测试

### 4. common_state.rs - 通用状态定义

**功能**: 提供跨模块使用的通用状态类型

**核心结构**:

#### LoadingState - 加载状态
```rust
pub enum LoadingState {
    Idle,      // 空闲状态
    Loading,   // 加载中
    Success,   // 加载成功
    Failed,    // 加载失败
}
```

#### ErrorState - 错误状态
```rust
pub struct ErrorState {
    pub message: String,              // 错误消息
    pub code: Option<i32>,            // 错误代码
    pub is_retryable: bool,           // 是否可重试
    pub details: Option<String>,      // 错误详情
}
```

#### PaginationState - 分页状态
```rust
pub struct PaginationState {
    pub current_page: usize,          // 当前页码（从 1 开始）
    pub page_size: usize,             // 每页数量
    pub total_count: usize,           // 总记录数
}
```

#### ConnectionState - 网络连接状态
```rust
pub enum ConnectionState {
    Connected,     // 已连接
    Connecting,    // 连接中
    Disconnected,  // 已断开
    Error,         // 连接错误
}
```

**测试覆盖**:
- ✅ 加载状态测试
- ✅ 错误状态测试
- ✅ 分页状态测试（包括翻页逻辑）
- ✅ 连接状态测试

## 设计原则

### 1. 不可变性（Immutability）
所有状态结构都实现了 `Clone` trait，支持不可变更新模式：
```rust
let new_state = MonitoringState {
    current_load: 20.0,
    ..old_state  // 使用结构体更新语法
};
```

### 2. 类型安全
- 使用强类型枚举（如 `AlarmType`、`LoadingState`）而非字符串
- 使用 `Option<T>` 表示可选值
- 使用 `SystemTime` 表示时间戳

### 3. 可测试性
- 每个状态模块都包含完整的单元测试
- 测试覆盖了核心功能和边界情况
- 使用 `#[cfg(test)]` 模块组织测试代码

### 4. 文档化
- 所有公共结构和方法都有中文文档注释
- 字段含义清晰，包含单位说明（如"吨"、"米"、"度"）

## 与 MVI 架构的集成

### 在 Reducer 中使用
```rust
// src/reducers/monitoring_reducer.rs
pub fn reduce(&self, state: MonitoringState, intent: MonitoringIntent) -> MonitoringState {
    match intent {
        MonitoringIntent::ClearError => {
            MonitoringState {
                error_message: None,
                ..state  // 不可变更新
            }
        }
        // ... 其他 intent 处理
    }
}
```

### 在 ViewModel 中使用
```rust
// src/viewmodels/monitoring_viewmodel.rs
pub struct MonitoringViewModelRust {
    state: MonitoringState,  // 持有状态
    // ...
}

impl MonitoringViewModel {
    fn update_state(mut self: Pin<&mut Self>, new_state: MonitoringState) {
        // 只更新变化的属性
        if self.state.current_load != new_state.current_load {
            self.as_mut().set_current_load(new_state.current_load);
        }
        // ...
    }
}
```

## 编译验证

```bash
$ cargo check
   Compiling qt-rust-demo v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.45s

✅ 编译通过，无错误
⚠️  24 个警告（未使用的导入和方法，待后续模块实现后自然消除）
```

## 代码统计

| 文件 | 行数 | 说明 |
|------|------|------|
| monitoring_state.rs | 119 | 监控状态 + 3 个测试 |
| chart_state.rs | 173 | 图表状态 + 4 个测试 |
| alarm_state.rs | 293 | 报警状态 + 6 个测试 |
| common_state.rs | 287 | 通用状态 + 4 个测试 |
| mod.rs | 13 | 模块导出 |
| **总计** | **885** | **17 个单元测试** |

## 下一步工作

根据 MVI 架构规范，接下来需要实现：

1. **Intents 层** (`src/intents/`)
   - monitoring_intent.rs
   - chart_intent.rs
   - alarm_intent.rs

2. **Reducers 层** (`src/reducers/`)
   - monitoring_reducer.rs
   - chart_reducer.rs
   - alarm_reducer.rs

3. **Models 层** (`src/models/`)
   - sensor_data.rs
   - alarm_record.rs
   - crane_config.rs

4. **Data Sources 层** (`src/data_sources/`)
   - sensor_data_source.rs
   - history_data_source.rs
   - config_data_source.rs

5. **Repository 层** (`src/repositories/`)
   - crane_data_repository.rs

6. **ViewModels 层** (`src/viewmodels/`)
   - monitoring_viewmodel.rs
   - chart_viewmodel.rs
   - alarm_viewmodel.rs

## 参考文档

- [MVI 架构规范](.kiro/steering/mvi-architecture.md)
- [Crane 数据层设计](CRANE_DATA_LAYER_DESIGN.md)

## 变更历史

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-03-16 | 1.0.0 | 初始实现，完成 4 个状态模块和 17 个单元测试 |
