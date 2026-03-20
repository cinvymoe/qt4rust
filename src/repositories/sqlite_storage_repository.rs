// SQLite 存储仓库实现

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::{Connection, params};
use crate::repositories::storage_repository::StorageRepository;
use crate::models::{ProcessedData, AlarmRecord, AlarmType};

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
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
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
                moment_percentage REAL NOT NULL,
                is_danger BOOLEAN NOT NULL,
                validation_error TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        ).map_err(|e| format!("Failed to create runtime_data table: {}", e))?;
        
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
        ).map_err(|e| format!("Failed to create alarm_records table: {}", e))?;
        
        // 创建索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_runtime_timestamp ON runtime_data(timestamp)", [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_runtime_sequence ON runtime_data(sequence_number)", [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_alarm_timestamp ON alarm_records(timestamp)", [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_alarm_acknowledged ON alarm_records(acknowledged)", [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
        
        eprintln!("[INFO] Database tables initialized");
        Ok(())
    }
}

#[async_trait]
impl StorageRepository for SqliteStorageRepository {
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String> {
        if data.is_empty() {
            return Ok(0);
        }
        
        let conn = self.connection.lock().await;
        
        // 开始事务
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;
        
        let mut saved_count = 0;
        
        for item in data {
            let timestamp = item.timestamp.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            
            let result = conn.execute(
                "INSERT OR IGNORE INTO runtime_data 
                 (sequence_number, timestamp, current_load, working_radius, 
                  boom_angle, moment_percentage, is_danger, validation_error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    item.sequence_number as i64,
                    timestamp,
                    item.current_load,
                    item.working_radius,
                    item.boom_angle,
                    item.moment_percentage,
                    item.is_danger,
                    item.validation_error.as_ref().map(|s| s.as_str()),
                ],
            );
            
            match result {
                Ok(rows) => saved_count += rows,
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
        
        eprintln!("[INFO] Saved {} runtime records to database", saved_count);
        Ok(saved_count)
    }
    
    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String> {
        let conn = self.connection.lock().await;
        
        let timestamp = data.timestamp.duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let alarm_type = if data.moment_percentage >= 100.0 {
            "danger"
        } else {
            "warning"
        };
        
        let description = format!(
            "力矩百分比 {:.1}% 超过阈值，当前载荷 {:.1}t，工作半径 {:.1}m",
            data.moment_percentage,
            data.current_load,
            data.working_radius
        );
        
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
                25.0,  // TODO: 从配置获取额定载荷
                data.working_radius,
                data.boom_angle,
                0.0,  // TODO: 从传感器数据获取臂长
                data.moment_percentage,
                description,
            ],
        ).map_err(|e| format!("Failed to insert alarm record: {}", e))?;
        
        let alarm_id = conn.last_insert_rowid();
        
        eprintln!("[INFO] Saved alarm record: {} (id: {})", alarm_type, alarm_id);
        Ok(alarm_id)
    }
    
    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String> {
        let conn = self.connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT sequence_number, timestamp, current_load, working_radius, 
                    boom_angle, moment_percentage, is_danger, validation_error
             FROM runtime_data 
             ORDER BY timestamp DESC 
             LIMIT ?1"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
        
        let rows = stmt.query_map([limit], |row| {
            let timestamp_secs: i64 = row.get(1)?;
            let timestamp = std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);
            
            Ok(ProcessedData {
                sequence_number: row.get::<_, i64>(0)? as u64,
                timestamp,
                current_load: row.get(2)?,
                working_radius: row.get(3)?,
                boom_angle: row.get(4)?,
                moment_percentage: row.get(5)?,
                is_danger: row.get(6)?,
                validation_error: row.get(7)?,
            })
        }).map_err(|e| format!("Failed to query: {}", e))?;
        
        let mut data = Vec::new();
        for row in rows {
            data.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
        }
        
        Ok(data)
    }
    
    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String> {
        let conn = self.connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT id, sequence_number, timestamp, alarm_type, current_load, rated_load,
                    working_radius, boom_angle, boom_length, moment_percentage, 
                    description, acknowledged, acknowledged_at
             FROM alarm_records 
             WHERE acknowledged = 0 
             ORDER BY timestamp DESC"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
        
        let rows = stmt.query_map([], |row| {
            let timestamp_secs: i64 = row.get(2)?;
            let timestamp = std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs as u64);
            
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
        }).map_err(|e| format!("Failed to query: {}", e))?;
        
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
        ).map_err(|e| format!("Failed to acknowledge alarm: {}", e))?;
        
        Ok(())
    }
    
    async fn get_last_stored_sequence(&self) -> Result<u64, String> {
        let conn = self.connection.lock().await;
        
        let result: Result<i64, _> = conn.query_row(
            "SELECT MAX(sequence_number) FROM runtime_data",
            [],
            |row| row.get(0),
        );
        
        match result {
            Ok(seq) => Ok(seq as u64),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(format!("Failed to get last sequence: {}", e)),
        }
    }
    
    async fn health_check(&self) -> Result<(), String> {
        let conn = self.connection.lock().await;
        
        conn.execute("SELECT 1", [])
            .map_err(|e| format!("Health check failed: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sensor_data::SensorData;
    
    #[tokio::test]
    async fn test_new() {
        let repo = SqliteStorageRepository::new(":memory:").await;
        assert!(repo.is_ok());
    }
    
    #[tokio::test]
    async fn test_save_and_query_runtime_data() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();
        
        // 创建测试数据
        let sensor_data = SensorData::new(20.0, 10.0, 60.0);
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
        let sensor_data = SensorData::new(23.0, 10.0, 60.0);
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
    async fn test_acknowledge_alarm() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();
        
        // 创建并保存报警
        let sensor_data = SensorData::new(23.0, 10.0, 60.0);
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
        let sensor_data = SensorData::new(20.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 5);
        repo.save_runtime_data_batch(&[processed]).await.unwrap();
        
        // 应该返回 5
        let seq = repo.get_last_stored_sequence().await.unwrap();
        assert_eq!(seq, 5);
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let repo = SqliteStorageRepository::new(":memory:").await.unwrap();
        assert!(repo.health_check().await.is_ok());
    }
}
