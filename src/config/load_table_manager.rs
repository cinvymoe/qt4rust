// src/config/load_table_manager.rs

use std::fs;
use std::path::Path;
use crane_data_layer::error::{DataError, DataResult};
use crate::models::rated_load_table::{RatedLoadTable, RatedLoadEntry};

/// 额定载荷表管理器
/// 
/// 负责从 CSV 文件加载和保存额定载荷表配置
pub struct LoadTableManager {
    /// 配置文件路径
    config_path: String,
}

impl LoadTableManager {
    /// 创建新的载荷表管理器
    /// 
    /// # 参数
    /// - `config_path`: CSV 配置文件路径
    /// 
    /// # 示例
    /// ```
    /// use crate::config::load_table_manager::LoadTableManager;
    /// 
    /// let manager = LoadTableManager::new("config/rated_load_table.csv");
    /// ```
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
        }
    }
    
    /// 加载载荷表配置
    /// 
    /// 从 CSV 文件读取载荷表数据和阈值配置。
    /// 如果文件不存在，自动创建包含默认值的配置文件。
    /// 
    /// # CSV 格式
    /// ```csv
    /// # moment_warning_threshold,85.0
    /// # moment_alarm_threshold,95.0
    /// radius_m,rated_load_ton
    /// 3.0,50.0
    /// 5.0,40.0
    /// ```
    /// 
    /// # 返回
    /// - `Ok(RatedLoadTable)`: 加载成功，返回载荷表
    /// - `Err(DataError)`: 加载失败，返回错误信息
    /// 
    /// # 错误
    /// - `DataError::IoError`: 文件读取失败
    /// - `DataError::SerializationError`: CSV 解析失败
    /// - `DataError::ValidationError`: 载荷表验证失败
    pub fn load(&self) -> DataResult<RatedLoadTable> {
        let path = Path::new(&self.config_path);
        
        // 如果文件不存在，创建默认配置
        if !path.exists() {
            tracing::info!("载荷表文件不存在，创建默认配置: {}", self.config_path);
            let default_table = RatedLoadTable::default();
            self.save(&default_table)?;
            return Ok(default_table);
        }
        
        // 读取文件
        let content = fs::read_to_string(path)
            .map_err(|e| DataError::IoError(format!("无法读取载荷表文件 {}: {}", self.config_path, e)))?;
        
        // 解析阈值（从注释行）
        let mut moment_warning_threshold = 85.0;
        let mut moment_alarm_threshold = 95.0;
        
        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with('#') {
                break;
            }
            
            if line.contains("moment_warning_threshold") {
                if let Some(value_str) = line.split(',').nth(1) {
                    moment_warning_threshold = value_str.trim().parse()
                        .map_err(|e| DataError::SerializationError(
                            format!("解析 moment_warning_threshold 失败: {}", e)
                        ))?;
                }
            } else if line.contains("moment_alarm_threshold") {
                if let Some(value_str) = line.split(',').nth(1) {
                    moment_alarm_threshold = value_str.trim().parse()
                        .map_err(|e| DataError::SerializationError(
                            format!("解析 moment_alarm_threshold 失败: {}", e)
                        ))?;
                }
            }
        }
        
        // 使用 csv crate 解析数据行
        let mut reader = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .from_reader(content.as_bytes());
        
        let mut entries = Vec::new();
        
        for result in reader.deserialize() {
            let entry: RatedLoadEntry = result
                .map_err(|e| DataError::SerializationError(format!("CSV 解析失败: {}", e)))?;
            entries.push(entry);
        }
        
        // 构建载荷表
        #[allow(deprecated)]
        let table = RatedLoadTable {
            entries,
            moment_warning_threshold,
            moment_alarm_threshold,
            alarm_threshold: moment_warning_threshold,  // 兼容旧字段
            danger_threshold: moment_alarm_threshold,   // 兼容旧字段
        };
        
        // 验证载荷表
        table.validate()
            .map_err(DataError::ValidationError)?;
        
        tracing::info!("载荷表加载成功: {}", self.config_path);
        Ok(table)
    }
    
    /// 保存载荷表配置
    /// 
    /// 将载荷表数据和阈值配置写入 CSV 文件。
    /// 在保存前会验证载荷表的有效性。
    /// 
    /// # 参数
    /// - `table`: 要保存的载荷表
    /// 
    /// # 返回
    /// - `Ok(())`: 保存成功
    /// - `Err(DataError)`: 保存失败，返回错误信息
    /// 
    /// # 错误
    /// - `DataError::ValidationError`: 载荷表验证失败
    /// - `DataError::IoError`: 文件写入失败
    pub fn save(&self, table: &RatedLoadTable) -> DataResult<()> {
        // 验证载荷表
        table.validate()
            .map_err(DataError::ValidationError)?;
        
        // 生成 CSV 内容
        let mut content = String::new();
        
        // 添加阈值注释
        content.push_str(&format!("# moment_warning_threshold,{}\n", table.moment_warning_threshold));
        content.push_str(&format!("# moment_alarm_threshold,{}\n", table.moment_alarm_threshold));
        
        // 添加表头
        content.push_str("radius,rated_load\n");
        
        // 添加数据行
        for entry in &table.entries {
            content.push_str(&format!("{},{}\n", entry.radius, entry.rated_load));
        }
        
        // 确保目录存在
        if let Some(parent) = Path::new(&self.config_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DataError::IoError(format!("无法创建配置目录: {}", e)))?;
        }
        
        // 写入文件
        fs::write(&self.config_path, content)
            .map_err(|e| DataError::IoError(format!("无法写入载荷表文件 {}: {}", self.config_path, e)))?;
        
        tracing::info!("载荷表已保存: {}", self.config_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_new() {
        let manager = LoadTableManager::new("config/test.csv");
        assert_eq!(manager.config_path, "config/test.csv");
    }
    
    #[test]
    fn test_load_creates_default_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        let manager = LoadTableManager::new(config_path.to_str().unwrap());
        
        // 文件不存在时应该创建默认配置
        let result = manager.load();
        assert!(result.is_ok());
        
        let table = result.unwrap();
        assert!(!table.entries.is_empty());
        assert_eq!(table.moment_warning_threshold, 85.0);
        assert_eq!(table.moment_alarm_threshold, 95.0);
        
        // 验证文件已创建
        assert!(config_path.exists());
    }
    
    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        let manager = LoadTableManager::new(config_path.to_str().unwrap());
        
        // 创建测试载荷表
        #[allow(deprecated)]
        let table = RatedLoadTable {
            entries: vec![
                RatedLoadEntry { radius: 5.0, rated_load: 40.0 },
                RatedLoadEntry { radius: 10.0, rated_load: 25.0 },
            ],
            moment_warning_threshold: 80.0,
            moment_alarm_threshold: 90.0,
            alarm_threshold: 80.0,
            danger_threshold: 90.0,
        };
        
        // 保存
        let save_result = manager.save(&table);
        assert!(save_result.is_ok());
        
        // 加载
        let load_result = manager.load();
        assert!(load_result.is_ok());
        
        let loaded_table = load_result.unwrap();
        assert_eq!(loaded_table.entries.len(), 2);
        assert_eq!(loaded_table.entries[0].radius, 5.0);
        assert_eq!(loaded_table.entries[0].rated_load, 40.0);
        assert_eq!(loaded_table.moment_warning_threshold, 80.0);
        assert_eq!(loaded_table.moment_alarm_threshold, 90.0);
    }
    
    #[test]
    fn test_save_validation_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        let manager = LoadTableManager::new(config_path.to_str().unwrap());
        
        // 创建无效的载荷表（空表）
        #[allow(deprecated)]
        let invalid_table = RatedLoadTable {
            entries: vec![],
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            alarm_threshold: 85.0,
            danger_threshold: 95.0,
        };
        
        // 保存应该失败
        let result = manager.save(&invalid_table);
        assert!(result.is_err());
        
        if let Err(DataError::ValidationError(msg)) = result {
            assert!(msg.contains("载荷表不能为空"));
        } else {
            panic!("Expected ValidationError");
        }
    }
    
    #[test]
    fn test_load_invalid_csv() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        
        // 写入无效的 CSV 内容
        fs::write(&config_path, "invalid,csv,content\n1,2,3,4,5\n").unwrap();
        
        let manager = LoadTableManager::new(config_path.to_str().unwrap());
        let result = manager.load();
        
        // 应该返回解析错误
        assert!(result.is_err());
    }
    
    #[test]
    fn test_csv_format_with_comments() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        
        // 写入带注释的 CSV
        let content = "# moment_warning_threshold,88.0\n\
                       # moment_alarm_threshold,98.0\n\
                       radius,rated_load\n\
                       5.0,40.0\n\
                       10.0,25.0\n";
        fs::write(&config_path, content).unwrap();
        
        let manager = LoadTableManager::new(config_path.to_str().unwrap());
        let result = manager.load();
        
        assert!(result.is_ok());
        let table = result.unwrap();
        assert_eq!(table.moment_warning_threshold, 88.0);
        assert_eq!(table.moment_alarm_threshold, 98.0);
        assert_eq!(table.entries.len(), 2);
    }
    
    #[test]
    fn test_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        let manager = LoadTableManager::new(config_path.to_str().unwrap());
        
        // 使用默认载荷表
        let original_table = RatedLoadTable::default();
        
        // 保存
        manager.save(&original_table).unwrap();
        
        // 加载
        let loaded_table = manager.load().unwrap();
        
        // 验证数据一致性
        assert_eq!(loaded_table.entries.len(), original_table.entries.len());
        assert_eq!(loaded_table.moment_warning_threshold, original_table.moment_warning_threshold);
        assert_eq!(loaded_table.moment_alarm_threshold, original_table.moment_alarm_threshold);
        
        for (loaded, original) in loaded_table.entries.iter().zip(original_table.entries.iter()) {
            assert_eq!(loaded.radius, original.radius);
            assert_eq!(loaded.rated_load, original.rated_load);
        }
    }
}
