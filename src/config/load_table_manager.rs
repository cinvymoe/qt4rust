// src/config/load_table_manager.rs

use crate::models::rated_load_table::RatedLoadTable;
use crane_data_layer::error::{DataError, DataResult};
use std::fs;
use std::path::Path;

/// 额定载荷表管理器
///
/// 负责从 CSV 文件加载和保存额定载荷表配置
pub struct LoadTableManager {
    config_path: String,
}

impl LoadTableManager {
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
        }
    }

    pub fn load(&self) -> DataResult<RatedLoadTable> {
        let path = Path::new(&self.config_path);

        if !path.exists() {
            tracing::info!("载荷表文件不存在，创建默认配置: {}", self.config_path);
            let default_table = RatedLoadTable::default();
            self.save(&default_table)?;
            return Ok(default_table);
        }

        let content = fs::read_to_string(path).map_err(|e| {
            DataError::IoError(format!("无法读取载荷表文件 {}: {}", self.config_path, e))
        })?;

        let mut moment_warning_threshold = 85.0;
        let mut moment_alarm_threshold = 95.0;

        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with('#') {
                break;
            }

            if line.contains("moment_warning_threshold") {
                if let Some(value_str) = line.split(',').nth(1) {
                    moment_warning_threshold = value_str.trim().parse().map_err(|e| {
                        DataError::SerializationError(format!(
                            "解析 moment_warning_threshold 失败: {}",
                            e
                        ))
                    })?;
                }
            } else if line.contains("moment_alarm_threshold") {
                if let Some(value_str) = line.split(',').nth(1) {
                    moment_alarm_threshold = value_str.trim().parse().map_err(|e| {
                        DataError::SerializationError(format!(
                            "解析 moment_alarm_threshold 失败: {}",
                            e
                        ))
                    })?;
                }
            }
        }

        let has_header = content.lines().any(|l| {
            let l = l.trim();
            !l.starts_with('#')
                && !l.is_empty()
                && !l.contains("moment_warning_threshold")
                && !l.contains("moment_alarm_threshold")
        });

        let detected_columns = if has_header {
            content
                .lines()
                .find(|l| {
                    let l = l.trim();
                    !l.starts_with('#')
                        && !l.is_empty()
                        && !l.contains("moment_warning_threshold")
                        && !l.contains("moment_alarm_threshold")
                })
                .map(|l| l.split(',').count())
                .unwrap_or(2)
        } else {
            2
        };

        let mut table = RatedLoadTable::new();

        if detected_columns == 2 {
            tracing::warn!("检测到旧格式载荷表（2列），将自动转换为新格式");
            self.parse_old_format(&content, &mut table, has_header)?;
        } else {
            self.parse_new_format(&content, &mut table)?;
        }

        table.moment_warning_threshold = moment_warning_threshold;
        table.moment_alarm_threshold = moment_alarm_threshold;

        table.validate().map_err(DataError::ValidationError)?;

        tracing::info!("载荷表加载成功: {}", self.config_path);
        Ok(table)
    }

    fn parse_old_format(
        &self,
        content: &str,
        table: &mut RatedLoadTable,
        has_header: bool,
    ) -> DataResult<()> {
        let default_boom_length = 20.0;
        let mut skip_first = has_header;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if skip_first {
                skip_first = false;
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 2 {
                continue;
            }

            let radius: f64 = parts[0]
                .trim()
                .parse()
                .map_err(|e| DataError::SerializationError(format!("解析 radius 失败: {}", e)))?;
            let rated_load: f64 = parts[1].trim().parse().map_err(|e| {
                DataError::SerializationError(format!("解析 rated_load 失败: {}", e))
            })?;

            table.add_entry(default_boom_length, radius, rated_load);
        }

        Ok(())
    }

    fn parse_new_format(&self, content: &str, table: &mut RatedLoadTable) -> DataResult<()> {
        let lines: Vec<&str> = content.lines().collect();

        let header_idx = lines.iter().position(|l| {
            let l = l.trim();
            !l.starts_with('#') && !l.is_empty() && l.contains("boom_length_m")
        });

        let header_idx = match header_idx {
            Some(idx) => idx,
            None => {
                return Err(DataError::SerializationError(
                    "找不到载荷表头行（boom_length_m）".to_string(),
                ))
            }
        };

        let header_line = lines[header_idx];
        let headers: Vec<&str> = header_line.split(',').map(|s| s.trim()).collect();

        if headers.len() != 3 {
            return Err(DataError::SerializationError(format!(
                "新格式载荷表需要 3 列，当前有 {} 列",
                headers.len()
            )));
        }

        for line in lines.iter().skip(header_idx + 1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.contains("moment_warning_threshold") || line.contains("moment_alarm_threshold")
            {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                tracing::warn!("跳过列数不匹配的行: {}", line);
                continue;
            }

            let boom_length: f64 = match parts[0].parse() {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!("跳过无效的臂长 '{}': {}", parts[0], e);
                    continue;
                }
            };
            let working_radius: f64 = match parts[1].parse() {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!("跳过无效的工作幅度 '{}': {}", parts[1], e);
                    continue;
                }
            };
            let rated_load: f64 = match parts[2].parse() {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!("跳过无效的额定载荷 '{}': {}", parts[2], e);
                    continue;
                }
            };

            table.add_entry(boom_length, working_radius, rated_load);
        }

        Ok(())
    }

    pub fn save(&self, table: &RatedLoadTable) -> DataResult<()> {
        table.validate().map_err(DataError::ValidationError)?;

        let mut content = String::new();

        content.push_str(&format!(
            "# moment_warning_threshold,{}\n",
            table.moment_warning_threshold
        ));
        content.push_str(&format!(
            "# moment_alarm_threshold,{}\n",
            table.moment_alarm_threshold
        ));

        content.push_str("boom_length_m,working_radius_m,rated_load_ton\n");

        for boom_length in table.get_boom_lengths() {
            if let Some(entries) = table.get_entries_for_boom(boom_length) {
                for entry in entries {
                    content.push_str(&format!(
                        "{},{},{}\n",
                        entry.boom_length, entry.working_radius, entry.rated_load
                    ));
                }
            }
        }

        if let Some(parent) = Path::new(&self.config_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DataError::IoError(format!("无法创建配置目录: {}", e)))?;
        }

        fs::write(&self.config_path, content).map_err(|e| {
            DataError::IoError(format!("无法写入载荷表文件 {}: {}", self.config_path, e))
        })?;

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

        let result = manager.load();
        assert!(result.is_ok());

        let table = result.unwrap();
        assert!(table.get_boom_lengths().len() > 0);
        assert_eq!(table.moment_warning_threshold, 85.0);
        assert_eq!(table.moment_alarm_threshold, 95.0);

        assert!(config_path.exists());
    }

    #[test]
    fn test_save_and_load_new_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        let manager = LoadTableManager::new(config_path.to_str().unwrap());

        let mut table = RatedLoadTable::new();
        table.add_entry(10.0, 3.0, 50.0);
        table.add_entry(10.0, 5.0, 40.0);
        table.add_entry(15.0, 3.0, 45.0);
        table.moment_warning_threshold = 80.0;
        table.moment_alarm_threshold = 90.0;

        let save_result = manager.save(&table);
        assert!(save_result.is_ok());

        let load_result = manager.load();
        assert!(load_result.is_ok());

        let loaded_table = load_result.unwrap();
        assert_eq!(loaded_table.get_rated_load(10.0, 5.0), 40.0);
        assert_eq!(loaded_table.get_rated_load(15.0, 3.0), 45.0);
        assert_eq!(loaded_table.moment_warning_threshold, 80.0);
        assert_eq!(loaded_table.moment_alarm_threshold, 90.0);
    }

    #[test]
    fn test_load_old_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");

        let content = "# moment_warning_threshold,88.0\n\
                       # moment_alarm_threshold,98.0\n\
                       radius_m,rated_load_ton\n\
                       5.0,40.0\n\
                       10.0,25.0\n";
        fs::write(&config_path, content).unwrap();

        let manager = LoadTableManager::new(config_path.to_str().unwrap());
        let result = manager.load();

        assert!(result.is_ok());
        let table = result.unwrap();
        assert_eq!(table.moment_warning_threshold, 88.0);
        assert_eq!(table.moment_alarm_threshold, 98.0);
        assert_eq!(table.get_rated_load(20.0, 5.0), 40.0);
        assert_eq!(table.get_rated_load(20.0, 10.0), 25.0);
    }

    #[test]
    fn test_save_validation_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        let manager = LoadTableManager::new(config_path.to_str().unwrap());

        let invalid_table = RatedLoadTable::new();

        let result = manager.save(&invalid_table);
        assert!(result.is_err());

        if let Err(DataError::ValidationError(msg)) = result {
            assert!(msg.contains("载荷表不能为空"));
        }
    }

    #[test]
    fn test_load_invalid_csv() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");

        fs::write(&config_path, "invalid,csv,content\n1,2,3,4,5\n").unwrap();

        let manager = LoadTableManager::new(config_path.to_str().unwrap());
        let result = manager.load();

        assert!(result.is_err());
    }

    #[test]
    fn test_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("rated_load_table.csv");
        let manager = LoadTableManager::new(config_path.to_str().unwrap());

        let original_table = RatedLoadTable::default();

        manager.save(&original_table).unwrap();

        let loaded_table = manager.load().unwrap();

        assert_eq!(
            loaded_table.moment_warning_threshold,
            original_table.moment_warning_threshold
        );
        assert_eq!(
            loaded_table.moment_alarm_threshold,
            original_table.moment_alarm_threshold
        );

        for boom in original_table.get_boom_lengths() {
            if let Some(orig_entries) = original_table.get_entries_for_boom(boom) {
                if let Some(loaded_entries) = loaded_table.get_entries_for_boom(boom) {
                    assert_eq!(loaded_entries.len(), orig_entries.len());
                }
            }
        }
    }
}
