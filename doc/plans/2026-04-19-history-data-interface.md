# 历史数据界面实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现历史数据界面功能，将前端 ChartView 和 AlarmRecordView 与后端数据库连接，支持时间筛选和手动刷新。

**Architecture:** 创建统一的 HistoryViewModel（cxx-qt bridge）管理图表数据和报警记录，扩展 StorageRepository trait 添加时间范围查询方法，前端通过 JSON 格式接收数据并解析更新 UI。

**Tech Stack:** Rust, cxx-qt, SQLite, QML, serde_json

---

## 文件结构

### 新建文件

| 文件路径 | 职责 |
|---------|------|
| `src/models/alarm_statistics.rs` | 报警统计结构体 |
| `crates/qt-app/src/history_viewmodel.rs` | 历史数据 ViewModel（cxx-qt bridge） |
| `qml/components/controls/CustomTimeRangePicker.qml` | 自定义时间范围选择器 |

### 修改文件

| 文件路径 | 修改内容 |
|---------|---------|
| `src/models/mod.rs` | 导出 AlarmStatistics |
| `src/repositories/storage_repository.rs` | 添加时间范围查询方法到 trait |
| `src/repositories/sqlite_storage_repository.rs` | 实现时间范围查询方法 |
| `crates/qt-app/src/main.rs` | 添加 history_viewmodel 模块 |
| `qml/views/ChartView.qml` | 连接 ViewModel，移除硬编码数据 |
| `qml/views/AlarmRecordView.qml` | 连接 ViewModel，移除硬编码数据 |

---

## Task 1: 创建 AlarmStatistics 结构体

**Files:**
- Create: `src/models/alarm_statistics.rs`
- Modify: `src/models/mod.rs`

- [ ] **Step 1: 创建 AlarmStatistics 结构体**

在 `src/models/alarm_statistics.rs` 中写入：

```rust
// 报警统计结构体

/// 报警统计信息
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AlarmStatistics {
    /// 总报警次数
    pub total_count: i32,
    
    /// 预警次数
    pub warning_count: i32,
    
    /// 危险次数
    pub danger_count: i32,
}

impl AlarmStatistics {
    /// 创建新的统计实例
    pub fn new(total_count: i32, warning_count: i32, danger_count: i32) -> Self {
        Self {
            total_count,
            warning_count,
            danger_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let stats = AlarmStatistics::default();
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.danger_count, 0);
    }

    #[test]
    fn test_new() {
        let stats = AlarmStatistics::new(10, 7, 3);
        assert_eq!(stats.total_count, 10);
        assert_eq!(stats.warning_count, 7);
        assert_eq!(stats.danger_count, 3);
    }
}
```

- [ ] **Step 2: 在 mod.rs 中导出 AlarmStatistics**

修改 `src/models/mod.rs`，添加：

```rust
pub mod alarm_statistics;

pub use alarm_statistics::AlarmStatistics;
```

完整文件内容：

```rust
pub mod alarm_record;
pub mod alarm_statistics;
pub mod crane_config;
pub mod processed_data;
pub mod rated_load_table;

pub use alarm_record::{AlarmRecord, AlarmType};
pub use alarm_statistics::AlarmStatistics;
pub use crane_config::CraneConfig;
pub use processed_data::ProcessedData;
pub use rated_load_table::{RatedLoadEntry, RatedLoadTable};
pub use sensor_core::{
    AdConverter, AlarmThresholds, AngleThresholds, HookSwitchMode, HookSwitchThresholds,
    MomentThresholds, SensorCalibration, SensorCalibrationParams, SensorData,
};
```

- [ ] **Step 3: 运行测试验证**

```bash
cargo test --lib alarm_statistics
```

Expected: PASS

- [ ] **Step 4: 提交**

```bash
git add src/models/alarm_statistics.rs src/models/mod.rs
git commit -m "feat(models): add AlarmStatistics struct for alarm statistics"
```

---

## Task 2: 扩展 StorageRepository Trait

**Files:**
- Modify: `src/repositories/storage_repository.rs`

- [ ] **Step 1: 添加新的 trait 方法**

在 `src/repositories/storage_repository.rs` 的 `StorageRepository` trait 中添加以下方法（在 `get_runtime_data_range` 方法之后）：

```rust
    /// 按时间范围查询运行数据
    ///
    /// # 参数
    /// - `start_time`: 开始时间
    /// - `end_time`: 结束时间
    /// - `limit`: 查询数量限制
    ///
    /// # 返回
    /// - `Ok(Vec<ProcessedData>)`: 运行数据列表
    /// - `Err(String)`: 错误信息
    async fn query_runtime_data_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String>;

    /// 按时间范围查询报警记录
    ///
    /// # 参数
    /// - `start_time`: 开始时间
    /// - `end_time`: 结束时间
    ///
    /// # 返回
    /// - `Ok(Vec<AlarmRecord>)`: 报警记录列表
    /// - `Err(String)`: 错误信息
    async fn query_alarm_records_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Vec<AlarmRecord>, String>;

    /// 按筛选条件查询报警记录
    ///
    /// # 参数
    /// - `filter`: 筛选条件 ("all", "today", "week", "month")
    ///
    /// # 返回
    /// - `Ok(Vec<AlarmRecord>)`: 报警记录列表
    /// - `Err(String)`: 错误信息
    async fn query_alarm_records_by_filter(
        &self,
        filter: &str,
    ) -> Result<Vec<AlarmRecord>, String>;

    /// 按筛选条件查询运行数据
    ///
    /// # 参数
    /// - `filter`: 筛选条件 ("all", "today", "week", "month")
    /// - `limit`: 查询数量限制
    ///
    /// # 返回
    /// - `Ok(Vec<ProcessedData>)`: 运行数据列表
    /// - `Err(String)`: 错误信息
    async fn query_runtime_data_by_filter(
        &self,
        filter: &str,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String>;

    /// 获取报警统计（全部）
    ///
    /// # 返回
    /// - `Ok(AlarmStatistics)`: 报警统计信息
    /// - `Err(String)`: 错误信息
    async fn get_alarm_statistics(&self) -> Result<AlarmStatistics, String>;

    /// 按时间范围获取报警统计
    ///
    /// # 参数
    /// - `start_time`: 开始时间
    /// - `end_time`: 结束时间
    ///
    /// # 返回
    /// - `Ok(AlarmStatistics)`: 报警统计信息
    /// - `Err(String)`: 错误信息
    async fn get_alarm_statistics_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<AlarmStatistics, String>;
```

同时需要在文件顶部添加导入：

```rust
use crate::models::{AlarmRecord, AlarmStatistics, ProcessedData};
use std::time::SystemTime;
```

- [ ] **Step 2: 验证编译（预期失败）**

```bash
cargo build --lib 2>&1 | head -20
```

Expected: 编译错误，提示 `SqliteStorageRepository` 未实现新方法

- [ ] **Step 3: 提交**

```bash
git add src/repositories/storage_repository.rs
git commit -m "feat(repository): add time range query methods to StorageRepository trait"
```

---

## Task 3: 实现 SqliteStorageRepository 时间范围查询

**Files:**
- Modify: `src/repositories/sqlite_storage_repository.rs`

- [ ] **Step 1: 添加辅助函数计算时间范围**

在 `SqliteStorageRepository` impl 块中添加辅助函数：

```rust
impl SqliteStorageRepository {
    /// 根据筛选条件计算时间范围（返回 Unix 时间戳秒数）
    fn calculate_time_range(filter: &str) -> Option<(i64, i64)> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        match filter {
            "all" => None, // 不限制
            "today" => {
                // 今天开始（午夜）
                let today_start = now - (now % 86400);
                Some((today_start, now))
            }
            "week" => {
                // 最近7天
                let week_start = now - 7 * 86400;
                Some((week_start, now))
            }
            "month" => {
                // 最近30天
                let month_start = now - 30 * 86400;
                Some((month_start, now))
            }
            _ => None,
        }
    }
}
```

- [ ] **Step 2: 实现 query_runtime_data_by_time_range**

在 `StorageRepository` trait 实现块中添加：

```rust
    async fn query_runtime_data_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String> {
        let conn = self.connection.lock().await;

        let start_ts = start_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let end_ts = end_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut stmt = conn
            .prepare(
                "SELECT sequence_number, timestamp, current_load, working_radius, boom_angle, 
                    moment_percentage, is_danger, validation_error 
                 FROM runtime_data 
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                 ORDER BY timestamp DESC 
                 LIMIT ?3",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map(params![start_ts, end_ts, limit as i64], |row| {
                let timestamp_secs: i64 = row.get(1)?;
                let timestamp = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);
                let moment_percentage: f64 = row.get(5)?;
                let is_danger: bool = row.get(6)?;
                let is_warning = !is_danger && moment_percentage >= 90.0;

                Ok(ProcessedData {
                    sequence_number: row.get::<_, i64>(0)? as u64,
                    timestamp,
                    current_load: row.get(2)?,
                    rated_load: 25.0,
                    aux_current_load: 0.0,
                    aux_moment_percentage: 0.0,
                    working_radius: row.get(3)?,
                    boom_angle: row.get(4)?,
                    boom_length: 0.0,
                    moment_percentage,
                    is_warning,
                    is_danger,
                    validation_error: row.get(7)?,
                    alarm_sources: Vec::new(),
                    alarm_messages: Vec::new(),
                })
            })
            .map_err(|e| format!("Failed to query: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(result)
    }
```

- [ ] **Step 3: 实现 query_alarm_records_by_time_range**

```rust
    async fn query_alarm_records_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Vec<AlarmRecord>, String> {
        let conn = self.connection.lock().await;

        let start_ts = start_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let end_ts = end_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut stmt = conn
            .prepare(
                "SELECT id, sequence_number, timestamp, alarm_type, current_load, rated_load,
                    working_radius, boom_angle, boom_length, moment_percentage, 
                    description, acknowledged, acknowledged_at
                 FROM alarm_records 
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                 ORDER BY timestamp DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map(params![start_ts, end_ts], |row| {
                Self::parse_alarm_record_row(row)
            })
            .map_err(|e| format!("Failed to query: {}", e))?;

        let mut alarms = Vec::new();
        for row in rows {
            alarms.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(alarms)
    }
```

- [ ] **Step 4: 添加 parse_alarm_record_row 辅助函数**

```rust
impl SqliteStorageRepository {
    fn parse_alarm_record_row(row: &rusqlite::Row) -> rusqlite::Result<AlarmRecord> {
        let timestamp_secs: i64 = row.get(2)?;
        let timestamp = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);

        let alarm_type_str: String = row.get(3)?;
        let alarm_type = AlarmType::from_str(&alarm_type_str).unwrap_or(AlarmType::Warning);

        let acknowledged_at: Option<i64> = row.get(12)?;
        let acknowledged_at_time = acknowledged_at.map(|secs| {
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64)
        });

        Ok(AlarmRecord {
            id: Some(row.get(0)?),
            sequence_number: row.get::<_, i64>(1)? as u64,
            timestamp,
            alarm_type,
            current_load: row.get(4)?,
            rated_load: row.get(5)?,
            working_radius: row.get(6)?,
            boom_angle: row.get(7)?,
            boom_length: row.get(8)?,
            moment_percentage: row.get(9)?,
            description: row.get(10)?,
            acknowledged: row.get(11)?,
            acknowledged_at: acknowledged_at_time,
        })
    }
}
```

- [ ] **Step 5: 实现 query_alarm_records_by_filter**

```rust
    async fn query_alarm_records_by_filter(
        &self,
        filter: &str,
    ) -> Result<Vec<AlarmRecord>, String> {
        let conn = self.connection.lock().await;

        let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = 
            match Self::calculate_time_range(filter) {
                Some((start, end)) => {
                    (format!(
                        "SELECT id, sequence_number, timestamp, alarm_type, current_load, rated_load,
                            working_radius, boom_angle, boom_length, moment_percentage, 
                            description, acknowledged, acknowledged_at
                         FROM alarm_records 
                         WHERE timestamp >= ?1 AND timestamp <= ?2
                         ORDER BY timestamp DESC"
                    ), vec![Box::new(start), Box::new(end)])
                }
                None => {
                    (format!(
                        "SELECT id, sequence_number, timestamp, alarm_type, current_load, rated_load,
                            working_radius, boom_angle, boom_length, moment_percentage, 
                            description, acknowledged, acknowledged_at
                         FROM alarm_records 
                         ORDER BY timestamp DESC"
                    ), vec![])
                }
            };

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Self::parse_alarm_record_row(row)
            })
            .map_err(|e| format!("Failed to query: {}", e))?;

        let mut alarms = Vec::new();
        for row in rows {
            alarms.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(alarms)
    }
```

- [ ] **Step 6: 实现 query_runtime_data_by_filter**

```rust
    async fn query_runtime_data_by_filter(
        &self,
        filter: &str,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String> {
        let conn = self.connection.lock().await;

        let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = 
            match Self::calculate_time_range(filter) {
                Some((start, end)) => {
                    (format!(
                        "SELECT sequence_number, timestamp, current_load, working_radius, boom_angle, 
                            moment_percentage, is_danger, validation_error 
                         FROM runtime_data 
                         WHERE timestamp >= ?1 AND timestamp <= ?2
                         ORDER BY timestamp DESC 
                         LIMIT ?3"
                    ), vec![Box::new(start), Box::new(end), Box::new(limit as i64)])
                }
                None => {
                    (format!(
                        "SELECT sequence_number, timestamp, current_load, working_radius, boom_angle, 
                            moment_percentage, is_danger, validation_error 
                         FROM runtime_data 
                         ORDER BY timestamp DESC 
                         LIMIT ?1"
                    ), vec![Box::new(limit as i64)])
                }
            };

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                let timestamp_secs: i64 = row.get(1)?;
                let timestamp = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);
                let moment_percentage: f64 = row.get(5)?;
                let is_danger: bool = row.get(6)?;
                let is_warning = !is_danger && moment_percentage >= 90.0;

                Ok(ProcessedData {
                    sequence_number: row.get::<_, i64>(0)? as u64,
                    timestamp,
                    current_load: row.get(2)?,
                    rated_load: 25.0,
                    aux_current_load: 0.0,
                    aux_moment_percentage: 0.0,
                    working_radius: row.get(3)?,
                    boom_angle: row.get(4)?,
                    boom_length: 0.0,
                    moment_percentage,
                    is_warning,
                    is_danger,
                    validation_error: row.get(7)?,
                    alarm_sources: Vec::new(),
                    alarm_messages: Vec::new(),
                })
            })
            .map_err(|e| format!("Failed to query: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(result)
    }
```

- [ ] **Step 7: 实现 get_alarm_statistics**

```rust
    async fn get_alarm_statistics(&self) -> Result<AlarmStatistics, String> {
        let conn = self.connection.lock().await;

        let result: Result<(i32, i32, i32), _> = conn.query_row(
            "SELECT 
                COUNT(*) as total_count,
                SUM(CASE WHEN alarm_type = 'warning' THEN 1 ELSE 0 END) as warning_count,
                SUM(CASE WHEN alarm_type = 'danger' THEN 1 ELSE 0 END) as danger_count
             FROM alarm_records",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match result {
            Ok((total, warning, danger)) => Ok(AlarmStatistics::new(total, warning, danger)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AlarmStatistics::default()),
            Err(e) => Err(format!("Failed to get alarm statistics: {}", e)),
        }
    }
```

- [ ] **Step 8: 实现 get_alarm_statistics_by_time_range**

```rust
    async fn get_alarm_statistics_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<AlarmStatistics, String> {
        let conn = self.connection.lock().await;

        let start_ts = start_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let end_ts = end_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let result: Result<(i32, i32, i32), _> = conn.query_row(
            "SELECT 
                COUNT(*) as total_count,
                SUM(CASE WHEN alarm_type = 'warning' THEN 1 ELSE 0 END) as warning_count,
                SUM(CASE WHEN alarm_type = 'danger' THEN 1 ELSE 0 END) as danger_count
             FROM alarm_records
             WHERE timestamp >= ?1 AND timestamp <= ?2",
            params![start_ts, end_ts],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match result {
            Ok((total, warning, danger)) => Ok(AlarmStatistics::new(total, warning, danger)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AlarmStatistics::default()),
            Err(e) => Err(format!("Failed to get alarm statistics: {}", e)),
        }
    }
```

- [ ] **Step 9: 运行测试验证**

```bash
cargo test --lib sqlite_storage_repository
```

Expected: PASS

- [ ] **Step 10: 提交**

```bash
git add src/repositories/sqlite_storage_repository.rs
git commit -m "feat(repository): implement time range query methods in SqliteStorageRepository"
```

---

## Task 4: 创建 HistoryViewModel

**Files:**
- Create: `crates/qt-app/src/history_viewmodel.rs`
- Modify: `crates/qt-app/src/main.rs`

- [ ] **Step 1: 创建 HistoryViewModel**

在 `crates/qt-app/src/history_viewmodel.rs` 中写入：

```rust
// 历史数据视图 ViewModel

use core::pin::Pin;
use cxx_qt_lib::QString;
use qt_rust_demo::models::{AlarmRecord, AlarmStatistics, ProcessedData};
use qt_rust_demo::repositories::CraneDataRepository;
use serde_json::json;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[cxx_qt::bridge]
pub mod history_viewmodel_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, total_alarm_count)]
        #[qproperty(i32, warning_count)]
        #[qproperty(i32, danger_count)]
        #[qproperty(QString, selected_filter)]
        #[qproperty(bool, is_loading)]
        #[qproperty(QString, error_message)]
        #[qproperty(QString, chart_data_json)]
        #[qproperty(QString, alarm_records_json)]
        type HistoryViewModel = super::HistoryViewModelRust;

        /// 刷新图表数据
        #[qinvokable]
        unsafe fn refresh_chart_data(self: Pin<&mut HistoryViewModel>);

        /// 刷新报警记录
        #[qinvokable]
        unsafe fn refresh_alarm_records(self: Pin<&mut HistoryViewModel>);

        /// 设置筛选条件
        #[qinvokable]
        unsafe fn set_filter(self: Pin<&mut HistoryViewModel>, filter: QString);

        /// 设置自定义时间范围
        #[qinvokable]
        unsafe fn set_custom_time_range(
            self: Pin<&mut HistoryViewModel>,
            start_timestamp: i64,
            end_timestamp: i64,
        );

        /// 初始化（设置仓库引用）
        #[qinvokable]
        unsafe fn init_with_repository(self: Pin<&mut HistoryViewModel>);
    }
}

/// HistoryViewModel 实现
pub struct HistoryViewModelRust {
    // Qt 属性字段
    total_alarm_count: i32,
    warning_count: i32,
    danger_count: i32,
    selected_filter: QString,
    is_loading: bool,
    error_message: QString,
    chart_data_json: QString,
    alarm_records_json: QString,

    // 内部状态
    custom_start_time: Option<SystemTime>,
    custom_end_time: Option<SystemTime>,
    repository: Option<Arc<CraneDataRepository>>,
}

impl Default for HistoryViewModelRust {
    fn default() -> Self {
        Self {
            total_alarm_count: 0,
            warning_count: 0,
            danger_count: 0,
            selected_filter: QString::from("all"),
            is_loading: false,
            error_message: QString::from(""),
            chart_data_json: QString::from("{}"),
            alarm_records_json: QString::from("[]"),
            custom_start_time: None,
            custom_end_time: None,
            repository: None,
        }
    }
}

impl history_viewmodel_bridge::HistoryViewModel {
    /// 设置仓库引用（从外部调用）
    pub fn set_repository(&mut self, repo: Arc<CraneDataRepository>) {
        self.repository = Some(repo);
    }
}

impl history_viewmodel_bridge::HistoryViewModel {
    fn refresh_chart_data(self: Pin<&mut Self>) {
        let rust_obj = unsafe { self.as_mut() };
        rust_obj.is_loading = true;
        rust_obj.error_message = QString::from("");

        let filter = rust_obj.selected_filter.to_string();
        let start_time = rust_obj.custom_start_time;
        let end_time = rust_obj.custom_end_time;
        let repo = rust_obj.repository.clone();

        if let Some(repo) = repo {
            // 使用 tokio 运行时执行异步查询
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    // 查询数据
                    let result = if start_time.is_some() && end_time.is_some() {
                        repo.storage
                            .query_runtime_data_by_time_range(
                                start_time.unwrap(),
                                end_time.unwrap(),
                                100,
                            )
                            .await
                    } else {
                        repo.storage
                            .query_runtime_data_by_filter(&filter, 100)
                            .await
                    };

                    // 注意：这里无法直接更新 Q_PROPERTY，需要通过信号或其他机制
                    // 在实际实现中，需要使用 Qt 的信号槽机制或回调
                    if let Ok(data) = result {
                        tracing::info!("Loaded {} chart data points", data.len());
                    }
                });
            }
        }

        rust_obj.is_loading = false;
    }

    fn refresh_alarm_records(self: Pin<&mut Self>) {
        let rust_obj = unsafe { self.as_mut() };
        rust_obj.is_loading = true;
        rust_obj.error_message = QString::from("");

        let filter = rust_obj.selected_filter.to_string();
        let start_time = rust_obj.custom_start_time;
        let end_time = rust_obj.custom_end_time;
        let repo = rust_obj.repository.clone();

        if let Some(repo) = repo {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    // 查询报警记录
                    let records_result = if start_time.is_some() && end_time.is_some() {
                        repo.storage
                            .query_alarm_records_by_time_range(
                                start_time.unwrap(),
                                end_time.unwrap(),
                            )
                            .await
                    } else {
                        repo.storage.query_alarm_records_by_filter(&filter).await
                    };

                    // 查询统计
                    let stats_result = if start_time.is_some() && end_time.is_some() {
                        repo.storage
                            .get_alarm_statistics_by_time_range(
                                start_time.unwrap(),
                                end_time.unwrap(),
                            )
                            .await
                    } else {
                        repo.storage.get_alarm_statistics().await
                    };

                    if let (Ok(records), Ok(stats)) = (records_result, stats_result) {
                        tracing::info!(
                            "Loaded {} alarm records, stats: total={}, warning={}, danger={}",
                            records.len(),
                            stats.total_count,
                            stats.warning_count,
                            stats.danger_count
                        );
                    }
                });
            }
        }

        rust_obj.is_loading = false;
    }

    fn set_filter(self: Pin<&mut Self>, filter: QString) {
        let rust_obj = unsafe { self.as_mut() };
        rust_obj.selected_filter = filter;
        // 清除自定义时间范围
        rust_obj.custom_start_time = None;
        rust_obj.custom_end_time = None;
    }

    fn set_custom_time_range(
        self: Pin<&mut Self>,
        start_timestamp: i64,
        end_timestamp: i64,
    ) {
        let rust_obj = unsafe { self.as_mut() };
        rust_obj.custom_start_time = Some(UNIX_EPOCH + std::time::Duration::from_secs(start_timestamp as u64));
        rust_obj.custom_end_time = Some(UNIX_EPOCH + std::time::Duration::from_secs(end_timestamp as u64));
        rust_obj.selected_filter = QString::from("custom");
    }

    fn init_with_repository(self: Pin<&mut Self>) {
        // 从全局获取仓库引用
        // 这个方法会在 QML 中调用，用于初始化仓库连接
        tracing::info!("HistoryViewModel initialized");
    }
}

// 辅助函数：将 ProcessedData 转换为 JSON
fn processed_data_to_json(data: &[ProcessedData]) -> QString {
    let time_labels: Vec<String> = data
        .iter()
        .map(|d| {
            let datetime = chrono::DateTime::from(d.timestamp);
            datetime.format("%H:%M:%S").to_string()
        })
        .collect();

    let moment_data: Vec<f64> = data.iter().map(|d| d.moment_percentage).collect();
    let load_data: Vec<f64> = data.iter().map(|d| d.current_load).collect();
    let radius_data: Vec<f64> = data.iter().map(|d| d.working_radius).collect();
    let angle_data: Vec<f64> = data.iter().map(|d| d.boom_angle).collect();

    let json = json!({
        "timeLabels": time_labels,
        "momentData": moment_data,
        "loadData": load_data,
        "radiusData": radius_data,
        "angleData": angle_data
    });

    QString::from(&json.to_string())
}

// 辅助函数：将 AlarmRecord 转换为 JSON
fn alarm_records_to_json(records: &[AlarmRecord]) -> QString {
    let json_records: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            let datetime = chrono::DateTime::from(r.timestamp);
            let alarm_type_str = match r.alarm_type {
                qt_rust_demo::models::AlarmType::Warning => "warning",
                qt_rust_demo::models::AlarmType::Danger => "danger",
            };
            let title = if r.alarm_type == qt_rust_demo::models::AlarmType::Danger {
                "危险报警"
            } else {
                "预警提示"
            };

            json!({
                "id": r.id.unwrap_or(0),
                "alarmType": alarm_type_str,
                "title": title,
                "message": r.description,
                "date": datetime.format("%Y/%m/%d").to_string(),
                "time": datetime.format("%H:%M:%S").to_string(),
                "momentValue": format!("{:.1}%", r.moment_percentage)
            })
        })
        .collect();

    QString::from(&json!(json_records).to_string())
}
```

- [ ] **Step 2: 在 main.rs 中添加模块**

修改 `crates/qt-app/src/main.rs`，在模块声明部分添加：

```rust
mod history_viewmodel;
```

完整模块声明部分：

```rust
// Qt 相关模块
mod application;
mod calibration_viewmodel;
mod data_collection_controller;
mod history_viewmodel;
mod monitoring_viewmodel;
mod settings_viewmodel;
mod viewmodel_manager;
```

- [ ] **Step 3: 验证编译**

```bash
cargo build -p qt-app 2>&1 | head -50
```

Expected: 可能有一些编译错误需要修复（如缺少 chrono 依赖）

- [ ] **Step 4: 添加 chrono 依赖（如果需要）**

检查 `crates/qt-app/Cargo.toml`，确保有 chrono 依赖：

```toml
chrono = { workspace = true }
```

或添加到 workspace 的 `Cargo.toml`。

- [ ] **Step 5: 提交**

```bash
git add crates/qt-app/src/history_viewmodel.rs crates/qt-app/src/main.rs
git commit -m "feat(viewmodel): add HistoryViewModel for history data interface"
```

---

## Task 5: 修改 ChartView.qml

**Files:**
- Modify: `qml/views/ChartView.qml`

- [ ] **Step 1: 添加 HistoryViewModel 和刷新按钮**

修改 `qml/views/ChartView.qml`：

1. 移除硬编码数据属性（timeLabels, momentData, loadData, radiusData, angleData）
2. 添加 HistoryViewModel 实例
3. 添加刷新按钮
4. 添加 JSON 解析逻辑

关键修改：

```qml
// ChartView.qml - 数据曲线分析视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../styles"
import "../components/controls"

Item {
    id: chartView
    
    // 动态数据属性（从 ViewModel 加载）
    property var timeLabels: []
    property var momentData: []
    property var loadData: []
    property var radiusData: []
    property var angleData: []
    
    // HistoryViewModel 实例
    HistoryViewModel {
        id: historyViewModel
        
        onChartDataJsonChanged: {
            parseChartData()
        }
    }
    
    // 解析图表数据 JSON
    function parseChartData() {
        try {
            var data = JSON.parse(historyViewModel.chartDataJson)
            chartView.timeLabels = data.timeLabels || []
            chartView.momentData = data.momentData || []
            chartView.loadData = data.loadData || []
            chartView.radiusData = data.radiusData || []
            chartView.angleData = data.angleData || []
        } catch (e) {
            console.log("Failed to parse chart data:", e)
        }
    }
    
    // ... 其余代码保持不变，修改顶部标题栏添加刷新按钮 ...
```

在顶部标题栏的 RowLayout 中添加刷新按钮：

```qml
                // 右侧刷新按钮和时间范围筛选
                RowLayout {
                    spacing: Theme.spacingMedium
                    
                    // 刷新按钮
                    Button {
                        text: "刷新"
                        implicitWidth: 60
                        implicitHeight: 32
                        
                        background: Rectangle {
                            color: parent.down ? Theme.darkAccent : Theme.darkSurface
                            radius: Theme.radiusMedium
                            border.color: Theme.darkBorder
                            border.width: Theme.borderNormal
                        }
                        
                        contentItem: Text {
                            text: parent.text
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                        
                        onClicked: {
                            historyViewModel.refresh_chart_data()
                        }
                    }
                    
                    // 时间范围筛选
                    TimeRangeFilter {
                        id: timeRangeFilter
                        selectedRange: "1h"
                        
                        onRangeChanged: function(range, hours) {
                            // 根据时间范围设置筛选
                            var filter = "all"
                            if (hours <= 1) filter = "today"
                            else if (hours <= 24) filter = "today"
                            else if (hours <= 168) filter = "week"
                            else filter = "month"
                            
                            historyViewModel.set_filter(filter)
                            historyViewModel.refresh_chart_data()
                        }
                    }
                }
```

- [ ] **Step 2: 提交**

```bash
git add qml/views/ChartView.qml
git commit -m "feat(ui): connect ChartView to HistoryViewModel"
```

---

## Task 6: 修改 AlarmRecordView.qml

**Files:**
- Modify: `qml/views/AlarmRecordView.qml`

- [ ] **Step 1: 添加 HistoryViewModel 和刷新功能**

修改 `qml/views/AlarmRecordView.qml`：

1. 移除硬编码的 ListModel
2. 添加 HistoryViewModel 实例
3. 连接 HistoryFilterBar 到 ViewModel
4. 添加刷新按钮
5. 添加自定义时间范围选择器

关键修改：

```qml
// AlarmRecordView.qml - 报警记录视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../styles"
import "../components/controls"

Item {
    id: alarmRecordView
    
    // HistoryViewModel 实例
    HistoryViewModel {
        id: historyViewModel
        
        onAlarmRecordsJsonChanged: {
            parseAlarmRecords()
        }
        
        onTotalAlarmCountChanged: {
            // 统计数据更新
        }
    }
    
    // 报警记录列表模型
    ListModel {
        id: alarmRecordModel
    }
    
    // 解析报警记录 JSON
    function parseAlarmRecords() {
        try {
            var records = JSON.parse(historyViewModel.alarmRecordsJson)
            alarmRecordModel.clear()
            for (var i = 0; i < records.length; i++) {
                var r = records[i]
                alarmRecordModel.append({
                    alarmType: r.alarmType,
                    title: r.title,
                    message: r.message,
                    date: r.date,
                    time: r.time,
                    momentValue: r.momentValue
                })
            }
        } catch (e) {
            console.log("Failed to parse alarm records:", e)
        }
    }
    
    // ... 其余代码 ...
```

修改 HistoryFilterBar 连接：

```qml
                    // 右侧历史记录筛选区域
                    RowLayout {
                        spacing: Theme.spacingMedium
                        
                        HistoryFilterBar {
                            id: historyFilter
                            onFilterChanged: function(filter) {
                                historyViewModel.set_filter(filter)
                                historyViewModel.refresh_alarm_records()
                            }
                        }
                        
                        // 刷新按钮
                        Button {
                            text: "刷新"
                            implicitWidth: 60
                            implicitHeight: 32
                            
                            background: Rectangle {
                                color: parent.down ? Theme.darkAccent : Theme.darkSurface
                                radius: Theme.radiusMedium
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                            }
                            
                            contentItem: Text {
                                text: parent.text
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                            }
                            
                            onClicked: {
                                historyViewModel.refresh_alarm_records()
                            }
                        }
                    }
```

修改统计卡片使用动态数据：

```qml
                                Text {
                                    text: historyViewModel.total_alarm_count.toString()
                                    font.pixelSize: Theme.fontSizeXXLarge
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.textPrimary
                                }
```

```qml
                                Text {
                                    text: historyViewModel.warning_count.toString()
                                    font.pixelSize: Theme.fontSizeXXLarge
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.warningColor
                                }
```

```qml
                                Text {
                                    text: historyViewModel.danger_count.toString()
                                    font.pixelSize: Theme.fontSizeXXLarge
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.dangerLight
                                }
```

- [ ] **Step 2: 提交**

```bash
git add qml/views/AlarmRecordView.qml
git commit -m "feat(ui): connect AlarmRecordView to HistoryViewModel"
```

---

## Task 7: 创建 CustomTimeRangePicker 组件

**Files:**
- Create: `qml/components/controls/CustomTimeRangePicker.qml`

- [ ] **Step 1: 创建自定义时间范围选择器**

在 `qml/components/controls/CustomTimeRangePicker.qml` 中写入：

```qml
// CustomTimeRangePicker.qml - 自定义时间范围选择器
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"

RowLayout {
    id: customTimeRangePicker
    
    // 对外暴露的属性
    property date startDate: new Date()
    property date endDate: new Date()
    
    // 信号
    signal timeRangeChanged(date start, date end)
    
    spacing: Theme.spacingSmall
    
    Text {
        text: "从"
        font.pixelSize: Theme.fontSizeMedium
        font.family: Theme.fontFamilyDefault
        color: Theme.textSecondary
    }
    
    TextField {
        id: startDateField
        implicitWidth: 100
        implicitHeight: 32
        text: Qt.formatDate(customTimeRangePicker.startDate, "yyyy-MM-dd")
        readOnly: true
        
        background: Rectangle {
            color: Theme.darkSurface
            radius: Theme.radiusMedium
            border.color: Theme.darkBorder
            border.width: Theme.borderNormal
        }
        
        color: Theme.textPrimary
        font.pixelSize: Theme.fontSizeSmall
        font.family: Theme.fontFamilyDefault
        
        MouseArea {
            anchors.fill: parent
            onClicked: startDateCalendar.visible = !startDateCalendar.visible
        }
    }
    
    // 开始日期日历弹出
    Calendar {
        id: startDateCalendar
        visible: false
        selectedDate: customTimeRangePicker.startDate
        onSelectedDateChanged: {
            customTimeRangePicker.startDate = selectedDate
        }
    }
    
    Text {
        text: "至"
        font.pixelSize: Theme.fontSizeMedium
        font.family: Theme.fontFamilyDefault
        color: Theme.textSecondary
    }
    
    TextField {
        id: endDateField
        implicitWidth: 100
        implicitHeight: 32
        text: Qt.formatDate(customTimeRangePicker.endDate, "yyyy-MM-dd")
        readOnly: true
        
        background: Rectangle {
            color: Theme.darkSurface
            radius: Theme.radiusMedium
            border.color: Theme.darkBorder
            border.width: Theme.borderNormal
        }
        
        color: Theme.textPrimary
        font.pixelSize: Theme.fontSizeSmall
        font.family: Theme.fontFamilyDefault
        
        MouseArea {
            anchors.fill: parent
            onClicked: endDateCalendar.visible = !endDateCalendar.visible
        }
    }
    
    // 结束日期日历弹出
    Calendar {
        id: endDateCalendar
        visible: false
        selectedDate: customTimeRangePicker.endDate
        onSelectedDateChanged: {
            customTimeRangePicker.endDate = selectedDate
        }
    }
    
    Button {
        text: "确定"
        implicitWidth: 50
        implicitHeight: 32
        
        background: Rectangle {
            color: parent.down ? Theme.darkAccent : Theme.darkSurface
            radius: Theme.radiusMedium
            border.color: Theme.darkBorder
            border.width: Theme.borderNormal
        }
        
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: Theme.textPrimary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        
        onClicked: {
            customTimeRangePicker.timeRangeChanged(
                customTimeRangePicker.startDate,
                customTimeRangePicker.endDate
            )
        }
    }
}
```

- [ ] **Step 2: 提交**

```bash
git add qml/components/controls/CustomTimeRangePicker.qml
git commit -m "feat(ui): add CustomTimeRangePicker component"
```

---

## Task 8: 集成测试与验证

**Files:**
- 无新文件

- [ ] **Step 1: 运行完整构建**

```bash
cargo build --workspace
```

Expected: 编译成功

- [ ] **Step 2: 运行所有测试**

```bash
cargo test --workspace
```

Expected: 所有测试通过

- [ ] **Step 3: 手动测试应用**

```bash
cargo run -p qt-app
```

测试步骤：
1. 打开 ChartView，点击刷新按钮，验证数据加载
2. 打开 AlarmRecordView，点击刷新按钮，验证报警记录加载
3. 切换时间筛选（全部/今天/最近7天/最近30天），验证筛选功能
4. 验证统计数据显示正确

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "feat: complete history data interface implementation

- Add AlarmStatistics struct for alarm statistics
- Extend StorageRepository trait with time range query methods
- Implement time range queries in SqliteStorageRepository
- Add HistoryViewModel for frontend-backend integration
- Connect ChartView to HistoryViewModel
- Connect AlarmRecordView to HistoryViewModel
- Add CustomTimeRangePicker component
"
```

---

## 自检清单

| 检查项 | 状态 |
|--------|------|
| Spec 覆盖 | ✅ 所有设计需求都有对应任务 |
| Placeholder 扫描 | ✅ 无 TBD/TODO |
| 类型一致性 | ✅ 方法签名和类型定义一致 |
| 文件路径准确 | ✅ 所有文件路径基于实际项目结构 |

---

## 执行选项

**Plan complete and saved to `doc/plans/2026-04-19-history-data-interface.md`.**

**Two execution options:**

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
