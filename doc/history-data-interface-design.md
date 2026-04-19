# 历史数据界面设计文档

## 概述

实现"查看历史数据界面"功能，将前端显示与后端数据库连接。包括：
- **ChartView（数据曲线分析）**：显示载荷、力矩、半径、角度的历史趋势图
- **AlarmRecordView（报警记录）**：显示报警历史列表和统计

## 需求

### 功能需求
1. **ChartView**：从数据库加载历史运行数据，显示趋势图
2. **AlarmRecordView**：从数据库加载报警记录，显示列表和统计
3. **时间筛选**：支持"全部/今天/最近7天/最近30天"筛选
4. **自定义时间范围**：用户可选择开始和结束日期
5. **手动刷新**：用户点击刷新按钮加载数据

### 非功能需求
- 遵循现有 cxx-qt 架构模式
- 复用现有组件（HistoryFilterBar、AlarmRecordItem）
- 保持与 MonitoringViewModel 一致的代码风格

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                        QML Frontend                          │
├─────────────────────────────────────────────────────────────┤
│  ChartView.qml          AlarmRecordView.qml                  │
│       │                        │                              │
│       └────────────┬───────────┘                              │
│                    ▼                                          │
│            HistoryViewModel (cxx-qt bridge)                   │
│                    │                                          │
│         ┌──────────┴──────────┐                               │
│         ▼                     ▼                               │
│   chartDataJson          alarmRecordsJson                     │
│   (Q_PROPERTY)           (Q_PROPERTY)                         │
└─────────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                      Rust Backend                            │
├─────────────────────────────────────────────────────────────┤
│  HistoryViewModelRust                                        │
│       │                                                       │
│       ▼                                                       │
│  StorageRepository trait                                      │
│       │                                                       │
│       ▼                                                       │
│  SqliteStorageRepository                                      │
│       │                                                       │
│       ▼                                                       │
│  SQLite Database (runtime_data, alarm_records)               │
└─────────────────────────────────────────────────────────────┘
```

## 后端实现

### 1. StorageRepository Trait 扩展

文件：`src/repositories/storage_repository.rs`

新增方法：

```rust
/// 按时间范围查询运行数据
async fn query_runtime_data_by_time_range(
    &self,
    start_time: SystemTime,
    end_time: SystemTime,
    limit: usize,
) -> Result<Vec<ProcessedData>, String>;

/// 按时间范围查询报警记录
async fn query_alarm_records_by_time_range(
    &self,
    start_time: SystemTime,
    end_time: SystemTime,
) -> Result<Vec<AlarmRecord>, String>;

/// 按筛选条件查询报警记录
/// filter: "all", "today", "week", "month"
async fn query_alarm_records_by_filter(
    &self,
    filter: &str,
) -> Result<Vec<AlarmRecord>, String>;

/// 按筛选条件查询运行数据
/// filter: "all", "today", "week", "month"
async fn query_runtime_data_by_filter(
    &self,
    filter: &str,
    limit: usize,
) -> Result<Vec<ProcessedData>, String>;

/// 获取报警统计
async fn get_alarm_statistics(&self) -> Result<AlarmStatistics, String>;

/// 按时间范围获取报警统计
async fn get_alarm_statistics_by_time_range(
    &self,
    start_time: SystemTime,
    end_time: SystemTime,
) -> Result<AlarmStatistics, String>;
```

### 2. AlarmStatistics 结构体

文件：`src/models/alarm_statistics.rs`（新建）

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AlarmStatistics {
    pub total_count: i32,
    pub warning_count: i32,
    pub danger_count: i32,
}
```

### 3. SqliteStorageRepository 实现

文件：`src/repositories/sqlite_storage_repository.rs`

实现上述 trait 方法，使用 SQL 查询：

```sql
-- 按时间范围查询运行数据
SELECT * FROM runtime_data 
WHERE timestamp >= ? AND timestamp <= ?
ORDER BY timestamp DESC
LIMIT ?

-- 按时间范围查询报警记录
SELECT * FROM alarm_records
WHERE timestamp >= ? AND timestamp <= ?
ORDER BY timestamp DESC

-- 按筛选条件查询（以 "today" 为例）
SELECT * FROM alarm_records
WHERE timestamp >= strftime('%s', 'now', 'start of day')
ORDER BY timestamp DESC

-- 获取报警统计
SELECT 
    COUNT(*) as total_count,
    SUM(CASE WHEN alarm_type = 'warning' THEN 1 ELSE 0 END) as warning_count,
    SUM(CASE WHEN alarm_type = 'danger' THEN 1 ELSE 0 END) as danger_count
FROM alarm_records
WHERE timestamp >= ? AND timestamp <= ?
```

### 4. HistoryViewModel

文件：`crates/qt-app/src/history_viewmodel.rs`（新建）

#### QML 属性

```rust
#[qproperty(i32, total_alarm_count)]      // 总报警次数
#[qproperty(i32, warning_count)]          // 预警次数
#[qproperty(i32, danger_count)]           // 危险次数
#[qproperty(QString, selected_filter)]    // 当前筛选
#[qproperty(bool, is_loading)]            // 加载状态
#[qproperty(QString, error_message)]      // 错误信息
#[qproperty(QString, chart_data_json)]    // 图表数据 JSON
#[qproperty(QString, alarm_records_json)] // 报警记录 JSON
```

#### QML 可调用方法

```rust
#[qinvokable]
fn refresh_chart_data(self: Pin<&mut Self>);

#[qinvokable]
fn refresh_alarm_records(self: Pin<&mut Self>);

#[qinvokable]
fn set_filter(self: Pin<&mut Self>, filter: QString);

#[qinvokable]
fn set_custom_time_range(self: Pin<&mut Self>, start: i64, end: i64);
```

#### 内部实现

- 使用 `Arc<CraneDataRepository>` 访问存储层
- 使用 `tokio::task::spawn_blocking` 执行异步查询
- 将查询结果序列化为 JSON 字符串供 QML 解析

### 5. 注册 ViewModel

文件：`crates/qt-app/src/viewmodel_manager.rs`

添加 `HistoryViewModel` 的初始化和注册。

## 前端实现

### 1. ChartView.qml 修改

- 移除硬编码的 `timeLabels`、`momentData`、`loadData`、`radiusData`、`angleData` 属性
- 添加 `HistoryViewModel` 实例
- 添加刷新按钮（右上角）
- 添加 `parseChartData()` 函数解析 JSON
- 更新子组件属性绑定

### 2. AlarmRecordView.qml 修改

- 移除硬编码的 `ListModel`
- 连接 `HistoryFilterBar.onFilterChanged` 到 `viewModel.setFilter(filter)`
- 添加刷新按钮
- 添加自定义时间范围选择器（当 filter === "custom" 时显示）
- 添加 `parseAlarmRecords()` 函数解析 JSON 并更新列表

### 3. 新增组件

文件：`qml/components/controls/CustomTimeRangePicker.qml`

```qml
RowLayout {
    property alias startDate: startDatePicker.selectedDate
    property alias endDate: endDatePicker.selectedDate
    
    signal timeRangeChanged(date start, date end)
    
    DatePicker { id: startDatePicker }
    Text { text: "至" }
    DatePicker { id: endDatePicker }
    Button { text: "确定"; onClicked: timeRangeChanged(startDate, endDate) }
}
```

### 4. JSON 数据格式

#### chart_data_json 格式

```json
{
  "timeLabels": ["14:33:49", "14:33:52", ...],
  "momentData": [85, 88, 92, ...],
  "loadData": [22, 28, 27, ...],
  "radiusData": [8, 10, 12, ...],
  "angleData": [75, 72, 70, ...]
}
```

#### alarm_records_json 格式

```json
[
  {
    "id": 1,
    "alarmType": "danger",
    "title": "危险报警",
    "message": "危险！力矩已达 95%",
    "date": "2026/4/19",
    "time": "15:48:32",
    "momentValue": "95.0%"
  },
  ...
]
```

## 数据流

```
用户操作 (点击刷新/选择筛选)
        │
        ▼
QML 调用 #[qinvokable] 方法
        │
        ▼
HistoryViewModel 调用 StorageRepository
        │
        ▼
SqliteStorageRepository 执行 SQL 查询
        │
        ▼
返回数据序列化为 JSON
        │
        ▼
Q_PROPERTY 更新触发 QML 绑定刷新
        │
        ▼
QML 解析 JSON 更新 UI
```

## 文件清单

### 新建文件

| 文件路径 | 说明 |
|---------|------|
| `src/models/alarm_statistics.rs` | 报警统计结构体 |
| `crates/qt-app/src/history_viewmodel.rs` | 历史数据 ViewModel |
| `qml/components/controls/CustomTimeRangePicker.qml` | 自定义时间范围选择器 |

### 修改文件

| 文件路径 | 修改内容 |
|---------|---------|
| `src/models/mod.rs` | 导出 AlarmStatistics |
| `src/repositories/storage_repository.rs` | 添加新查询方法 |
| `src/repositories/sqlite_storage_repository.rs` | 实现新查询方法 |
| `crates/qt-app/src/viewmodel_manager.rs` | 注册 HistoryViewModel |
| `crates/qt-app/src/lib.rs` | 导出 history_viewmodel 模块 |
| `qml/views/ChartView.qml` | 连接 ViewModel，移除硬编码数据 |
| `qml/views/AlarmRecordView.qml` | 连接 ViewModel，移除硬编码数据 |

## 测试计划

### 单元测试

1. `StorageRepository` 新方法的单元测试
2. `HistoryViewModel` 的状态管理测试
3. JSON 序列化/反序列化测试

### 集成测试

1. 从 QML 调用 ViewModel 方法验证数据返回
2. 时间筛选功能验证
3. 自定义时间范围验证

## 风险与缓解

| 风险 | 缓解措施 |
|------|---------|
| 大数据量查询性能 | 添加 LIMIT 限制，考虑分页 |
| QML JSON 解析性能 | 限制返回数据条数（图表最多 100 点） |
| 异步查询阻塞 UI | 使用 tokio 异步执行 |

## 实现顺序

1. 后端：StorageRepository trait 扩展 + SqliteStorageRepository 实现
2. 后端：AlarmStatistics 结构体
3. 后端：HistoryViewModel 实现
4. 前端：ChartView.qml 修改
5. 前端：AlarmRecordView.qml 修改
6. 前端：CustomTimeRangePicker.qml 新增
7. 测试与验证
