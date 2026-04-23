// SQLite 存储仓库实现

use crate::models::{AlarmRecord, AlarmStatistics, AlarmType, ProcessedData, SensorData};
use crate::repositories::sensor_data_repository::SensorDataRepository;
use crate::repositories::storage_repository::StorageRepository;
use async_trait::async_trait;
use rusqlite::{params, Connection, ToSql};
use sensor_core::error::StorageError;
use sensor_core::pipeline::aggregator::AggregatedSensorData;
use sensor_core::pipeline::data_source::DataSourceId;
use sensor_core::storage::repository::StorageRepository as SensorCoreStorageRepository;
use sensor_core::DatabaseSchema;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

/// SQLite 存储仓库
pub struct SqliteStorageRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteStorageRepository {
    /// 创建新的 SQLite 存储仓库
    ///
    /// # 参数
    /// - `db_path`: 数据库文件路径（使用 ":memory:" 创建内存数据库）
    ///
    /// # 返回
    /// - `Ok(SqliteStorageRepository)`: 创建成功
    /// - `Err(String)`: 错误信息
    pub async fn new(db_path: &str) -> Result<Self, String> {
        let conn =
            Connection::open(db_path).map_err(|e| format!("Failed to open database: {}", e))?;

        let repo = Self {
            connection: Arc::new(Mutex::new(conn)),
        };

        // 初始化表
        repo.init_tables().await?;

        Ok(repo)
    }

    /// 初始化数据库表
    async fn init_tables(&self) -> Result<(), String> {
        let conn = self.connection.lock().await;

        // 创建运行数据表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS runtime_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sequence_number INTEGER NOT NULL UNIQUE,
                timestamp INTEGER NOT NULL,
                current_load REAL NOT NULL,
                working_radius REAL NOT NULL,
                boom_angle REAL NOT NULL,
                boom_length REAL NOT NULL,
                moment_percentage REAL NOT NULL,
                is_danger BOOLEAN NOT NULL,
                validation_error TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )
        .map_err(|e| format!("Failed to create runtime_data table: {}", e))?;

        // 创建报警信息表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS alarm_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sequence_number INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                alarm_type TEXT NOT NULL,
                current_load REAL NOT NULL,
                rated_load REAL NOT NULL,
                working_radius REAL NOT NULL,
                boom_angle REAL NOT NULL,
                boom_length REAL NOT NULL,
                moment_percentage REAL NOT NULL,
                description TEXT,
                acknowledged BOOLEAN NOT NULL DEFAULT 0,
                acknowledged_at INTEGER,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )
        .map_err(|e| format!("Failed to create alarm_records table: {}", e))?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runtime_timestamp ON runtime_data(timestamp)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runtime_sequence ON runtime_data(sequence_number)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_alarm_timestamp ON alarm_records(timestamp)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_alarm_acknowledged ON alarm_records(acknowledged)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;

        // 创建传感器数据表
        conn.execute(&SensorData::create_table_sql(), [])
            .map_err(|e| format!("Failed to create sensor_data table: {}", e))?;

        tracing::info!("Database tables initialized");
        Ok(())
    }

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

    fn parse_alarm_record_row(row: &rusqlite::Row) -> rusqlite::Result<AlarmRecord> {
        let timestamp_secs: i64 = row.get(2)?;
        let timestamp =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);

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

#[async_trait]
impl StorageRepository for SqliteStorageRepository {
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String> {
        if data.is_empty() {
            return Ok(0);
        }

        tracing::debug!(
            "save_runtime_data_batch: attempting to save {} records",
            data.len()
        );
        tracing::debug!(
            "Sequence numbers: {:?}",
            data.iter().map(|d| d.sequence_number).collect::<Vec<_>>()
        );

        let conn = self.connection.lock().await;

        // 开始事务
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let mut saved_count = 0;
        let mut ignored_count = 0;

        for item in data {
            let timestamp = item
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let result = conn.execute(
                "INSERT OR IGNORE INTO runtime_data 
                 (sequence_number, timestamp, current_load, working_radius, 
                  boom_angle, boom_length, moment_percentage, is_danger, validation_error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    item.sequence_number as i64,
                    timestamp,
                    item.current_load,
                    item.working_radius,
                    item.boom_angle,
                    item.boom_length,
                    item.moment_percentage,
                    item.is_danger,
                    item.validation_error.as_ref().map(|s| s.as_str()),
                ],
            );

            match result {
                Ok(rows) => {
                    if rows > 0 {
                        saved_count += rows;
                    } else {
                        ignored_count += 1;
                        tracing::debug!(
                            "Ignored duplicate sequence_number: {}",
                            item.sequence_number
                        );
                    }
                }
                Err(e) => {
                    // 回滚事务
                    let _ = conn.execute("ROLLBACK", []);
                    return Err(format!("Failed to insert runtime data: {}", e));
                }
            }
        }

        // 提交事务
        conn.execute("COMMIT", [])
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        tracing::info!(
            "Saved {} runtime records to database (ignored {} duplicates)",
            saved_count,
            ignored_count
        );
        Ok(saved_count)
    }

    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String> {
        let conn = self.connection.lock().await;

        let timestamp = data
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // 根据报警来源和消息生成描述
        let description = if !data.alarm_messages.is_empty() {
            // 使用报警消息（包含角度、力矩等所有报警类型）
            data.alarm_messages.join("; ")
        } else if data.moment_percentage >= 90.0 {
            // 向后兼容：如果没有报警消息，使用力矩百分比
            format!(
                "力矩百分比 {:.1}% 超过阈值，当前载荷 {:.1}t，工作半径 {:.1}m",
                data.moment_percentage, data.current_load, data.working_radius
            )
        } else {
            // 默认描述
            format!(
                "报警触发，当前载荷 {:.1}t，工作半径 {:.1}m，吊臂角度 {:.1}°",
                data.current_load, data.working_radius, data.boom_angle
            )
        };

        let alarm_type = if data.moment_percentage >= 100.0 {
            "danger"
        } else {
            "warning"
        };

        conn.execute(
            "INSERT INTO alarm_records 
             (sequence_number, timestamp, alarm_type, current_load, rated_load, 
              working_radius, boom_angle, boom_length, moment_percentage, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                data.sequence_number as i64,
                timestamp,
                alarm_type,
                data.current_load,
                25.0, // TODO: 从配置获取额定载荷
                data.working_radius,
                data.boom_angle,
                0.0, // TODO: 从传感器数据获取臂长
                data.moment_percentage,
                description,
            ],
        )
        .map_err(|e| format!("Failed to insert alarm record: {}", e))?;

        let alarm_id = conn.last_insert_rowid();

        tracing::info!(
            "💾 报警记录已保存: {} (id: {}, 描述: {})",
            alarm_type,
            alarm_id,
            description
        );
        Ok(alarm_id)
    }

    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String> {
        let conn = self.connection.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT sequence_number, timestamp, current_load, working_radius, 
                    boom_angle, moment_percentage, is_danger, validation_error
             FROM runtime_data 
             ORDER BY timestamp DESC 
             LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map([limit], |row| {
                let timestamp_secs: i64 = row.get(1)?;
                let timestamp =
                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);

                let moment_percentage: f64 = row.get(6)?;
                let is_danger: bool = row.get(7)?;
                let is_warning = !is_danger && moment_percentage >= 90.0;

                Ok(ProcessedData {
                    sequence_number: row.get::<_, i64>(0)? as u64,
                    timestamp,
                    current_load: row.get(2)?,
                    rated_load: 25.0, // 从数据库读取时使用默认值
                    aux_current_load: 0.0,
                    aux_moment_percentage: 0.0,
                    working_radius: row.get(3)?,
                    boom_angle: row.get(4)?,
                    boom_length: row.get(5)?,
                    moment_percentage,
                    is_warning,
                    is_danger,
                    validation_error: row.get(8)?,
                    alarm_sources: Vec::new(),
                    alarm_messages: Vec::new(),
                })
            })
            .map_err(|e| format!("Failed to query: {}", e))?;

        let mut data = Vec::new();
        for row in rows {
            data.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(data)
    }

    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String> {
        let conn = self.connection.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, sequence_number, timestamp, alarm_type, current_load, rated_load,
                    working_radius, boom_angle, boom_length, moment_percentage, 
                    description, acknowledged, acknowledged_at
             FROM alarm_records 
             WHERE acknowledged = 0 
             ORDER BY timestamp DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                let timestamp_secs: i64 = row.get(2)?;
                let timestamp =
                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);

                let alarm_type_str: String = row.get(3)?;
                let alarm_type = AlarmType::from_str(&alarm_type_str).unwrap_or(AlarmType::Warning);

                let acknowledged_at: Option<i64> = row.get(12)?;
                let acknowledged_at_time = acknowledged_at.map(|secs| {
                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64)
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
            })
            .map_err(|e| format!("Failed to query: {}", e))?;

        let mut alarms = Vec::new();
        for row in rows {
            alarms.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(alarms)
    }

    async fn acknowledge_alarm(&self, alarm_id: i64) -> Result<(), String> {
        let conn = self.connection.lock().await;

        conn.execute(
            "UPDATE alarm_records 
             SET acknowledged = 1, acknowledged_at = strftime('%s', 'now')
             WHERE id = ?1",
            params![alarm_id],
        )
        .map_err(|e| format!("Failed to acknowledge alarm: {}", e))?;

        Ok(())
    }

    async fn get_last_stored_sequence(&self) -> Result<u64, String> {
        let conn = self.connection.lock().await;

        let result: Result<Option<i64>, _> =
            conn.query_row("SELECT MAX(sequence_number) FROM runtime_data", [], |row| {
                row.get(0)
            });

        match result {
            Ok(Some(seq)) => Ok(seq as u64),
            Ok(None) => Ok(0), // MAX() returns NULL when table is empty
            Err(e) => Err(format!("Failed to get last sequence: {}", e)),
        }
    }

    async fn health_check(&self) -> Result<(), String> {
        let conn = self.connection.lock().await;

        conn.query_row("SELECT 1", [], |_| Ok(()))
            .map_err(|e| format!("Health check failed: {}", e))?;

        Ok(())
    }

    async fn purge_old_records(
        &self,
        max_records: usize,
        purge_threshold: usize,
    ) -> Result<usize, String> {
        if max_records == 0 {
            return Ok(0);
        }

        let conn = self.connection.lock().await;

        // 获取当前记录数
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM runtime_data", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count records: {}", e))?;

        // 计算清理阈值
        let threshold = if purge_threshold > 0 {
            purge_threshold
        } else {
            // 默认：超过 max_records 的 110% 或超过 max_records + 1000 时才删除
            // 两者取较小值，避免 max_records 很小时频繁删除
            std::cmp::min(
                (max_records as f64 * 1.1) as usize,
                max_records.saturating_add(1000),
            )
        };

        if count as usize <= threshold {
            return Ok(0);
        }

        // 删除到 max_records（留出缓冲空间）
        let to_delete = count as usize - max_records;

        // 删除最早的记录（按 id 顺序）
        let deleted = conn
            .execute(
                "DELETE FROM runtime_data WHERE id IN (
                SELECT id FROM runtime_data ORDER BY id ASC LIMIT ?1
            )",
                params![to_delete as i64],
            )
            .map_err(|e| format!("Failed to purge old records: {}", e))?;

        tracing::info!(
            "Purged {} old records (current={}, threshold={}, max_records={})",
            deleted,
            count,
            threshold,
            max_records
        );
        Ok(deleted)
    }

    async fn purge_old_alarms(
        &self,
        alarm_max_records: usize,
        alarm_purge_threshold: usize,
    ) -> Result<usize, String> {
        if alarm_max_records == 0 {
            return Ok(0);
        }

        let conn = self.connection.lock().await;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM alarm_records", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count alarm records: {}", e))?;

        let threshold = if alarm_purge_threshold > 0 {
            alarm_purge_threshold
        } else {
            std::cmp::min(
                (alarm_max_records as f64 * 1.1) as usize,
                alarm_max_records.saturating_add(100),
            )
        };

        if count as usize <= threshold {
            return Ok(0);
        }

        let to_delete = count as usize - alarm_max_records;

        let deleted = conn
            .execute(
                "DELETE FROM alarm_records WHERE id IN (
                SELECT id FROM alarm_records ORDER BY id ASC LIMIT ?1
            )",
                params![to_delete as i64],
            )
            .map_err(|e| format!("Failed to purge old alarms: {}", e))?;

        tracing::info!(
            "Purged {} old alarms (current={}, threshold={}, alarm_max_records={})",
            deleted,
            count,
            threshold,
            alarm_max_records
        );
        Ok(deleted)
    }

    async fn get_runtime_data_count(&self) -> Result<i64, String> {
        let conn = self.connection.lock().await;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM runtime_data", [], |row| row.get(0))
            .map_err(|e| format!("Failed to get runtime data count: {}", e))?;

        Ok(count)
    }

    async fn get_runtime_data_range(
        &self,
        offset: i64,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String> {
        let conn = self.connection.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT sequence_number, timestamp, current_load, working_radius, boom_angle, 
                    moment_percentage, is_danger, validation_error 
             FROM runtime_data 
             ORDER BY id ASC 
             LIMIT ? OFFSET ?",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map(params![limit as i64, offset], |row| {
                let timestamp_secs: i64 = row.get(1)?;
                let timestamp =
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);
                let moment_percentage: f64 = row.get(6)?;
                let is_danger: bool = row.get(7)?;
                let is_warning = !is_danger && moment_percentage >= 90.0;

                Ok(ProcessedData {
                    sequence_number: row.get(0)?,
                    timestamp,
                    current_load: row.get(2)?,
                    rated_load: 25.0, // 从数据库读取时使用默认值
                    aux_current_load: 0.0,
                    aux_moment_percentage: 0.0,
                    working_radius: row.get(3)?,
                    boom_angle: row.get(4)?,
                    boom_length: row.get(5)?,
                    moment_percentage,
                    is_warning,
                    is_danger,
                    validation_error: row.get(8)?,
                    alarm_sources: Vec::new(),
                    alarm_messages: Vec::new(),
                })
            })
            .map_err(|e| format!("Failed to query runtime data: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(result)
    }

    async fn query_runtime_data_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String> {
        let conn = self.connection.lock().await;

        let start_timestamp = start_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let end_timestamp = end_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut stmt = conn
            .prepare(
                "SELECT sequence_number, timestamp, current_load, working_radius, boom_angle, 
                    boom_length, moment_percentage, is_danger, validation_error 
                 FROM runtime_data 
                 WHERE timestamp >= ?1 AND timestamp <= ?2 
                 ORDER BY timestamp DESC 
                 LIMIT ?3",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map(params![start_timestamp, end_timestamp, limit as i64], |row| {
                let timestamp_secs: i64 = row.get(1)?;
                let timestamp =
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);
                let moment_percentage: f64 = row.get(6)?;
                let is_danger: bool = row.get(7)?;
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
                    boom_length: row.get(5)?,
                    moment_percentage,
                    is_warning,
                    is_danger,
                    validation_error: row.get(8)?,
                    alarm_sources: Vec::new(),
                    alarm_messages: Vec::new(),
                })
            })
            .map_err(|e| format!("Failed to query runtime data: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(result)
    }

    async fn query_alarm_records_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Vec<AlarmRecord>, String> {
        let conn = self.connection.lock().await;

        let start_timestamp = start_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let end_timestamp = end_time
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
            .query_map(params![start_timestamp, end_timestamp], Self::parse_alarm_record_row)
            .map_err(|e| format!("Failed to query alarm records: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(result)
    }

    async fn query_alarm_records_by_filter(
        &self,
        filter: &str,
    ) -> Result<Vec<AlarmRecord>, String> {
        let conn = self.connection.lock().await;

        let time_range = Self::calculate_time_range(filter);

        let sql = if let Some((_start, _end)) = time_range {
            "SELECT id, sequence_number, timestamp, alarm_type, current_load, rated_load,
                working_radius, boom_angle, boom_length, moment_percentage, 
                description, acknowledged, acknowledged_at
             FROM alarm_records 
             WHERE timestamp >= ?1 AND timestamp <= ?2 
             ORDER BY timestamp DESC"
        } else {
            "SELECT id, sequence_number, timestamp, alarm_type, current_load, rated_load,
                working_radius, boom_angle, boom_length, moment_percentage, 
                description, acknowledged, acknowledged_at
             FROM alarm_records 
             ORDER BY timestamp DESC"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = if let Some((start, end)) = time_range {
            stmt.query_map(params![start, end], Self::parse_alarm_record_row)
        } else {
            stmt.query_map([], Self::parse_alarm_record_row)
        }
        .map_err(|e| format!("Failed to query alarm records: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(result)
    }

    async fn query_runtime_data_by_filter(
        &self,
        filter: &str,
        limit: usize,
    ) -> Result<Vec<ProcessedData>, String> {
        let conn = self.connection.lock().await;

        let time_range = Self::calculate_time_range(filter);

        let parse_row = |row: &rusqlite::Row| -> rusqlite::Result<ProcessedData> {
            let timestamp_secs: i64 = row.get(1)?;
            let timestamp =
                SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);
            let moment_percentage: f64 = row.get(6)?;
            let is_danger: bool = row.get(7)?;
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
                boom_length: row.get(5)?,
                moment_percentage,
                is_warning,
                is_danger,
                validation_error: row.get(8)?,
                alarm_sources: Vec::new(),
                alarm_messages: Vec::new(),
            })
        };

        let result = if let Some((start, end)) = time_range {
            let mut stmt = conn
                .prepare(
                    "SELECT sequence_number, timestamp, current_load, working_radius, boom_angle, 
                        boom_length, moment_percentage, is_danger, validation_error 
                     FROM runtime_data 
                     WHERE timestamp >= ?1 AND timestamp <= ?2 
                     ORDER BY timestamp DESC 
                     LIMIT ?3",
                )
                .map_err(|e| format!("Failed to prepare statement: {}", e))?;

            let rows = stmt
                .query_map(params![start, end, limit as i64], parse_row)
                .map_err(|e| format!("Failed to query runtime data: {}", e))?;

            let mut data = Vec::new();
            for row in rows {
                data.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
            }
            data
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT sequence_number, timestamp, current_load, working_radius, boom_angle, 
                        boom_length, moment_percentage, is_danger, validation_error 
                     FROM runtime_data 
                     ORDER BY timestamp DESC 
                     LIMIT ?1",
                )
                .map_err(|e| format!("Failed to prepare statement: {}", e))?;

            let rows = stmt
                .query_map(params![limit as i64], parse_row)
                .map_err(|e| format!("Failed to query runtime data: {}", e))?;

            let mut data = Vec::new();
            for row in rows {
                data.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
            }
            data
        };

        Ok(result)
    }

    async fn get_alarm_statistics(&self) -> Result<AlarmStatistics, String> {
        let conn = self.connection.lock().await;

        let stats: AlarmStatistics = conn
            .query_row(
                "SELECT 
                    COUNT(*) as total_count,
                    SUM(CASE WHEN alarm_type = 'warning' THEN 1 ELSE 0 END) as warning_count,
                    SUM(CASE WHEN alarm_type = 'danger' THEN 1 ELSE 0 END) as danger_count
                 FROM alarm_records",
                [],
                |row| {
                    Ok(AlarmStatistics {
                        total_count: row.get::<_, i64>(0)? as i32,
                        warning_count: row.get::<_, i64>(1)? as i32,
                        danger_count: row.get::<_, i64>(2)? as i32,
                    })
                },
            )
            .map_err(|e| format!("Failed to get alarm statistics: {}", e))?;

        Ok(stats)
    }

    async fn get_alarm_statistics_by_time_range(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<AlarmStatistics, String> {
        let conn = self.connection.lock().await;

        let start_timestamp = start_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let end_timestamp = end_time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let stats: AlarmStatistics = conn
            .query_row(
                "SELECT 
                    COUNT(*) as total_count,
                    SUM(CASE WHEN alarm_type = 'warning' THEN 1 ELSE 0 END) as warning_count,
                    SUM(CASE WHEN alarm_type = 'danger' THEN 1 ELSE 0 END) as danger_count
                 FROM alarm_records
                 WHERE timestamp >= ?1 AND timestamp <= ?2",
                params![start_timestamp, end_timestamp],
                |row| {
                    Ok(AlarmStatistics {
                        total_count: row.get::<_, i64>(0)? as i32,
                        warning_count: row.get::<_, i64>(1)? as i32,
                        danger_count: row.get::<_, i64>(2)? as i32,
                    })
                },
            )
            .map_err(|e| format!("Failed to get alarm statistics: {}", e))?;

        Ok(stats)
    }
}

#[async_trait]
impl SensorDataRepository for SqliteStorageRepository {
    async fn save_sensor_data_batch(&self, data: &[SensorData]) -> Result<usize, String> {
        if data.is_empty() {
            return Ok(0);
        }

        let conn = self.connection.lock().await;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let mut saved_count = 0;
        let sql = SensorData::insert_sql();

        for item in data {
            let values = item.field_values();
            let params: Vec<&dyn ToSql> = values.iter().map(|v| v.as_ref()).collect();

            match conn.execute(&sql, params.as_slice()) {
                Ok(rows) => {
                    saved_count += rows;
                }
                Err(e) => {
                    let _ = conn.execute("ROLLBACK", []);
                    return Err(format!("Failed to insert sensor data: {}", e));
                }
            }
        }

        conn.execute("COMMIT", [])
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        tracing::info!("Saved {} sensor records to database", saved_count);
        Ok(saved_count)
    }

    async fn query_recent_sensor_data(&self, limit: usize) -> Result<Vec<SensorData>, String> {
        let conn = self.connection.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT analog_json, digital_json
                 FROM sensor_data
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map([limit], |row| {
                let analog_json: String = row.get(0)?;
                let digital_json: String = row.get(1)?;
                let analog: std::collections::HashMap<String, f64> = serde_json::from_str(&analog_json).unwrap_or_default();
                let digital: std::collections::HashMap<String, bool> = serde_json::from_str(&digital_json).unwrap_or_default();
                Ok(SensorData::new(analog, digital))
            })
            .map_err(|e| format!("Failed to query: {}", e))?;

        let mut data = Vec::new();
        for row in rows {
            data.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }

        Ok(data)
    }

    async fn get_latest_sensor_data(&self) -> Result<Option<SensorData>, String> {
        let conn = self.connection.lock().await;

        let result: Result<SensorData, _> = conn.query_row(
            "SELECT analog_json, digital_json
             FROM sensor_data
             ORDER BY created_at DESC
             LIMIT 1",
            [],
            |row| {
                let analog_json: String = row.get(0)?;
                let digital_json: String = row.get(1)?;
                let analog: std::collections::HashMap<String, f64> = serde_json::from_str(&analog_json).unwrap_or_default();
                let digital: std::collections::HashMap<String, bool> = serde_json::from_str(&digital_json).unwrap_or_default();
                Ok(SensorData::new(analog, digital))
            },
        );

        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get latest sensor data: {}", e)),
        }
    }

    async fn get_sensor_data_count(&self) -> Result<i64, String> {
        let conn = self.connection.lock().await;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sensor_data", [], |row| row.get(0))
            .map_err(|e| format!("Failed to get sensor data count: {}", e))?;

        Ok(count)
    }

    async fn purge_old_sensor_data(&self, max_records: usize) -> Result<usize, String> {
        if max_records == 0 {
            return Ok(0);
        }

        let conn = self.connection.lock().await;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sensor_data", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count sensor data: {}", e))?;

        let threshold = std::cmp::min(
            (max_records as f64 * 1.1) as usize,
            max_records.saturating_add(1000),
        );

        if count as usize <= threshold {
            return Ok(0);
        }

        let to_delete = count as usize - max_records;

        let deleted = conn
            .execute(
                "DELETE FROM sensor_data WHERE id IN (
                    SELECT id FROM sensor_data ORDER BY id ASC LIMIT ?1
                )",
                params![to_delete as i64],
            )
            .map_err(|e| format!("Failed to purge old sensor data: {}", e))?;

        Ok(deleted)
    }

    async fn health_check(&self) -> Result<(), String> {
        let conn = self.connection.lock().await;

        conn.query_row("SELECT 1", [], |_| Ok(()))
            .map_err(|e| format!("Health check failed: {}", e))?;

        Ok(())
    }
}

// Implementation of sensor-core StorageRepository trait
#[async_trait]
impl SensorCoreStorageRepository for SqliteStorageRepository {
    async fn save_aggregated_data_batch(
        &self,
        data: Vec<AggregatedSensorData>,
    ) -> Result<(), StorageError> {
        if data.is_empty() {
            return Ok(());
        }

        // Convert AggregatedSensorData to ProcessedData and save
        let records: Vec<ProcessedData> = data
            .iter()
            .enumerate()
            .map(|(idx, agg)| self.convert_aggregated_to_processed(agg, idx as u64))
            .collect();

        self.save_runtime_data_batch(&records)
            .await
            .map_err(StorageError::Database)?;

        Ok(())
    }

    async fn query_recent_aggregated_data(
        &self,
        _limit: usize,
    ) -> Result<Vec<AggregatedSensorData>, StorageError> {
        // For now, UI uses existing query methods
        // Return empty vector as the main UI path uses query_recent_runtime_data
        Ok(Vec::new())
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        StorageRepository::health_check(self)
            .await
            .map_err(|e| StorageError::Database(e))
    }

    async fn get_last_sequence(&self) -> Result<u64, StorageError> {
        self.get_last_stored_sequence()
            .await
            .map_err(StorageError::Database)
    }
}

impl SqliteStorageRepository {
    /// Converts AggregatedSensorData to ProcessedData for storage.
    ///
    /// Uses primary source (Modbus) if available, otherwise falls back to first available source.
    fn convert_aggregated_to_processed(
        &self,
        aggregated: &AggregatedSensorData,
        sequence_number: u64,
    ) -> ProcessedData {
        // Prefer Modbus source, fallback to first available
        let sensor_data = aggregated
            .get_source(DataSourceId::Modbus)
            .or_else(|| aggregated.sources.values().next())
            .cloned()
            .unwrap_or_else(|| SensorData::from_tuple(0.0, 0.0, 0.0, false, false));

        ProcessedData::from_sensor_data(sensor_data, sequence_number)
    }
}

impl SqliteStorageRepository {
    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<String>, String> {
        let conn = self.connection.lock().await;

        let mut stmt = conn
            .prepare(&format!(
                "SELECT name FROM pragma_table_info('{}')",
                table_name
            ))
            .map_err(|e| format!("Failed to prepare table_info query: {}", e))?;

        let columns = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query table columns: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect columns: {}", e))?;

        Ok(columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensor_core::SensorData;

    #[tokio::test]
    async fn test_new() {
        let repo = SqliteStorageRepository::new(":memory:").await;
        assert!(repo.is_ok());
    }

    #[tokio::test]
    async fn test_save_and_query_runtime_data() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // 创建测试数据
        let sensor_data = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);

        // 保存数据
        let result = repo.save_runtime_data_batch(&[processed]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // 查询数据
        let data = repo.query_recent_runtime_data(10).await.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].sequence_number, 1);
    }

    #[tokio::test]
    async fn test_save_alarm_record() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // 创建报警数据
        let sensor_data = SensorData::from_tuple(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);

        // 保存报警
        let result = repo.save_alarm_record(&processed).await;
        assert!(result.is_ok());

        // 查询未确认报警
        let alarms = repo.query_unacknowledged_alarms().await.unwrap();
        assert_eq!(alarms.len(), 1);
        assert_eq!(alarms[0].sequence_number, 1);
    }

    #[tokio::test]
    async fn test_save_angle_alarm_record() {
        use crate::alarm::alarm_type::AlarmSource;

        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // 创建角度报警数据
        let sensor_data = SensorData::from_tuple(15.0, 10.0, 95.0, false, false); // 角度超限
        let mut processed = ProcessedData::from_sensor_data(sensor_data, 1);

        // 模拟角度报警
        processed.is_danger = true;
        processed.alarm_sources.push(AlarmSource::Angle);
        processed
            .alarm_messages
            .push("吊臂角度 95.0° 超过最大角度 85.0°".to_string());

        // 保存报警
        let result = repo.save_alarm_record(&processed).await;
        assert!(result.is_ok());

        // 查询未确认报警
        let alarms = repo.query_unacknowledged_alarms().await.unwrap();
        assert_eq!(alarms.len(), 1);
        assert_eq!(alarms[0].sequence_number, 1);

        // 验证描述包含角度信息
        assert!(alarms[0].description.contains("角度"));
        assert!(alarms[0].description.contains("95.0"));
    }

    #[tokio::test]
    async fn test_acknowledge_alarm() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // 创建并保存报警
        let sensor_data = SensorData::from_tuple(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        let alarm_id = repo.save_alarm_record(&processed).await.unwrap();

        // 确认报警
        assert!(repo.acknowledge_alarm(alarm_id).await.is_ok());

        // 查询未确认报警（应该为空）
        let alarms = repo.query_unacknowledged_alarms().await.unwrap();
        assert_eq!(alarms.len(), 0);
    }

    #[tokio::test]
    async fn test_get_last_stored_sequence() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // 初始应该是 0
        let seq = repo.get_last_stored_sequence().await.unwrap();
        assert_eq!(seq, 0);

        // 保存数据
        let sensor_data = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 5);
        repo.save_runtime_data_batch(&[processed]).await.unwrap();

        // 应该返回 5
        let seq = repo.get_last_stored_sequence().await.unwrap();
        assert_eq!(seq, 5);
    }

    #[tokio::test]
    async fn test_health_check() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();
        assert!(
            crate::repositories::storage_repository::StorageRepository::health_check(&repo)
                .await
                .is_ok()
        );
    }

    // ==================== SensorDataRepository Example Tests ====================
    // These tests demonstrate how to persist raw sensor data (AD1/AD2/AD3) to SQL

    #[tokio::test]
    async fn test_sensor_data_save_and_query() {
        // Example: Save raw sensor data to SQLite and query it back
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Create sensor data (AD1=load, AD2=radius, AD3=angle)
        let sensor_data = vec![
            SensorData::from_tuple(20.0, 10.0, 60.0, false, false),
            SensorData::from_tuple(21.0, 11.0, 61.0, false, false),
            SensorData::from_tuple(22.0, 12.0, 62.0, false, false),
        ];

        // Save batch to database
        let saved = repo.save_sensor_data_batch(&sensor_data).await.unwrap();
        assert_eq!(saved, 3);

        // Query recent data
        let retrieved = repo.query_recent_sensor_data(10).await.unwrap();
        assert_eq!(retrieved.len(), 3);

        // Verify data integrity (most recent first due to ORDER BY timestamp DESC)
        assert_eq!(retrieved[0].ad1_load(), 22.0);
        assert_eq!(retrieved[1].ad1_load(), 21.0);
        assert_eq!(retrieved[2].ad1_load(), 20.0);
    }

    #[tokio::test]
    async fn test_sensor_data_get_latest() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Initially no data
        let latest = repo.get_latest_sensor_data().await.unwrap();
        assert!(latest.is_none());

        // Save some data
        let sensor_data = vec![
            SensorData::from_tuple(10.0, 5.0, 45.0, false, false),
            SensorData::from_tuple(20.0, 10.0, 60.0, false, false),
        ];
        repo.save_sensor_data_batch(&sensor_data).await.unwrap();

        // Get latest
        let latest = repo.get_latest_sensor_data().await.unwrap();
        assert!(latest.is_some());
        let latest = latest.unwrap();
        assert_eq!(latest.ad1_load(), 20.0);
        assert_eq!(latest.ad2_radius(), 10.0);
        assert_eq!(latest.ad3_angle(), 60.0);
    }

    #[tokio::test]
    async fn test_sensor_data_count_and_purge() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Initial count is 0
        let count = repo.get_sensor_data_count().await.unwrap();
        assert_eq!(count, 0);

        // Save 5 records
        let sensor_data = vec![
            SensorData::from_tuple(10.0, 5.0, 45.0, false, false),
            SensorData::from_tuple(11.0, 5.5, 46.0, false, false),
            SensorData::from_tuple(12.0, 6.0, 47.0, false, false),
            SensorData::from_tuple(13.0, 6.5, 48.0, false, false),
            SensorData::from_tuple(14.0, 7.0, 49.0, false, false),
        ];
        repo.save_sensor_data_batch(&sensor_data).await.unwrap();

        let count = repo.get_sensor_data_count().await.unwrap();
        assert_eq!(count, 5);

        // Purge old data, keep only 3
        let purged = repo.purge_old_sensor_data(3).await.unwrap();
        assert_eq!(purged, 2);

        let count = repo.get_sensor_data_count().await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_sensor_data_health_check() {
        use crate::repositories::sensor_data_repository::SensorDataRepository;
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();
        assert!(SensorDataRepository::health_check(&repo).await.is_ok());
    }

    // ==================== Time Range Query Tests ====================

    #[tokio::test]
    async fn test_query_runtime_data_by_time_range() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Create and save test data
        let sensor_data1 = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
        let processed1 = ProcessedData::from_sensor_data(sensor_data1, 1);

        let sensor_data2 = SensorData::from_tuple(21.0, 11.0, 61.0, false, false);
        let processed2 = ProcessedData::from_sensor_data(sensor_data2, 2);

        repo.save_runtime_data_batch(&[processed1.clone(), processed2.clone()])
            .await
            .unwrap();

        // Query all data by time range
        let now = SystemTime::now();
        let start = now - std::time::Duration::from_secs(3600);
        let end = now + std::time::Duration::from_secs(3600);

        let data = repo
            .query_runtime_data_by_time_range(start, end, 10)
            .await
            .unwrap();
        assert_eq!(data.len(), 2);
    }

    #[tokio::test]
    async fn test_query_alarm_records_by_time_range() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Create and save alarm
        let sensor_data = SensorData::from_tuple(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        repo.save_alarm_record(&processed).await.unwrap();

        // Query by time range
        let now = SystemTime::now();
        let start = now - std::time::Duration::from_secs(3600);
        let end = now + std::time::Duration::from_secs(3600);

        let alarms = repo
            .query_alarm_records_by_time_range(start, end)
            .await
            .unwrap();
        assert_eq!(alarms.len(), 1);
    }

    #[tokio::test]
    async fn test_query_alarm_records_by_filter() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Create and save alarm
        let sensor_data = SensorData::from_tuple(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        repo.save_alarm_record(&processed).await.unwrap();

        // Query with "all" filter
        let alarms = repo.query_alarm_records_by_filter("all").await.unwrap();
        assert_eq!(alarms.len(), 1);

        // Query with "today" filter
        let alarms = repo.query_alarm_records_by_filter("today").await.unwrap();
        assert_eq!(alarms.len(), 1);
    }

    #[tokio::test]
    async fn test_query_runtime_data_by_filter() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Create and save test data
        let sensor_data = SensorData::from_tuple(20.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        repo.save_runtime_data_batch(&[processed]).await.unwrap();

        // Query with "all" filter
        let data = repo
            .query_runtime_data_by_filter("all", 10)
            .await
            .unwrap();
        assert_eq!(data.len(), 1);

        // Query with "today" filter
        let data = repo
            .query_runtime_data_by_filter("today", 10)
            .await
            .unwrap();
        assert_eq!(data.len(), 1);
    }

    #[tokio::test]
    async fn test_get_alarm_statistics() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Create and save alarms
        let sensor_data1 = SensorData::from_tuple(23.0, 10.0, 60.0, false, false);
        let processed1 = ProcessedData::from_sensor_data(sensor_data1, 1);
        repo.save_alarm_record(&processed1).await.unwrap();

        let sensor_data2 = SensorData::from_tuple(26.0, 10.0, 60.0, false, false);
        let processed2 = ProcessedData::from_sensor_data(sensor_data2, 2);
        repo.save_alarm_record(&processed2).await.unwrap();

        let stats = repo.get_alarm_statistics().await.unwrap();
        assert_eq!(stats.total_count, 2);
        assert!(stats.warning_count >= 1);
    }

    #[tokio::test]
    async fn test_get_alarm_statistics_by_time_range() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();

        // Create and save alarm
        let sensor_data = SensorData::from_tuple(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        repo.save_alarm_record(&processed).await.unwrap();

        // Query stats by time range
        let now = SystemTime::now();
        let start = now - std::time::Duration::from_secs(3600);
        let end = now + std::time::Duration::from_secs(3600);

        let stats = repo
            .get_alarm_statistics_by_time_range(start, end)
            .await
            .unwrap();
        assert_eq!(stats.total_count, 1);
    }
}
