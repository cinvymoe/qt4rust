//! CSV 配置文件解析器
//!
//! 专门用于解析额定负载表 CSV 文件

use crate::error::HotReloadError;
use qt_rust_demo::models::rated_load_table::{RatedLoadEntry, RatedLoadTable};
use std::fs;
use std::path::Path;

/// 解析额定负载表 CSV 文件
///
/// # 文件格式
/// ```csv
/// # 注释行以 # 开头
/// moment_warning_threshold,85.0
/// moment_alarm_threshold,95.0
///
/// boom_length_m,working_radius_m,rated_load_ton
/// 10.0,3.0,50.0
/// 10.0,5.0,40.0
/// ...
/// ```
///
/// # 参数
/// - `path`: CSV 文件路径
///
/// # 返回
/// - `Ok(RatedLoadTable)`: 解析成功，返回额定负载表
/// - `Err(HotReloadError)`: 解析失败，返回错误信息
///
/// # 错误处理
/// - 文件读取失败: 返回 `FileRead` 错误
/// - 编码错误: 返回 `EncodingError` 错误
/// - CSV 解析失败: 返回 `ParseError` 错误，包含详细的错误原因和行号信息
pub fn parse_rated_load_table(path: &Path) -> Result<RatedLoadTable, HotReloadError> {
    // 1. 读取文件内容
    let bytes = fs::read(path).map_err(|source| HotReloadError::FileRead {
        path: path.to_path_buf(),
        source,
    })?;

    // 2. 验证 UTF-8 编码
    let content = String::from_utf8(bytes).map_err(|source| HotReloadError::EncodingError {
        path: path.to_path_buf(),
        source,
    })?;

    // 3. 解析 CSV
    parse_csv_content(&content, path)
}

/// 解析 CSV 内容
fn parse_csv_content(content: &str, path: &Path) -> Result<RatedLoadTable, HotReloadError> {
    let mut table = RatedLoadTable::new();
    let mut moment_warning_threshold: Option<f64> = None;
    let mut moment_alarm_threshold: Option<f64> = None;
    let mut header_found = false;
    let mut line_number = 0;

    for line in content.lines() {
        line_number += 1;
        let line = line.trim();

        // 跳过空行和注释行
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 解析阈值配置
        if line.starts_with("moment_warning_threshold,") {
            moment_warning_threshold =
                parse_threshold_line(line, "moment_warning_threshold", path, line_number)?;
            continue;
        }

        if line.starts_with("moment_alarm_threshold,") {
            moment_alarm_threshold =
                parse_threshold_line(line, "moment_alarm_threshold", path, line_number)?;
            continue;
        }

        // 跳过表头
        if line.starts_with("boom_length_m,") {
            header_found = true;
            continue;
        }

        // 解析数据行
        if header_found {
            let entry = parse_data_line(line, path, line_number)?;
            table.add_entry(entry.boom_length, entry.working_radius, entry.rated_load);
        }
    }

    // 设置阈值
    if let Some(warning) = moment_warning_threshold {
        table.moment_warning_threshold = warning;
    }
    if let Some(alarm) = moment_alarm_threshold {
        table.moment_alarm_threshold = alarm;
    }

    // 验证表格不为空
    if table.is_empty() {
        return Err(HotReloadError::ParseError {
            path: path.to_path_buf(),
            reason: "CSV 文件中没有找到有效的数据行".to_string(),
        });
    }

    Ok(table)
}

/// 解析阈值配置行
fn parse_threshold_line(
    line: &str,
    key: &str,
    path: &Path,
    line_number: usize,
) -> Result<Option<f64>, HotReloadError> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() != 2 {
        return Err(HotReloadError::ParseError {
            path: path.to_path_buf(),
            reason: format!(
                "第 {} 行: {} 配置格式错误，应为 '{},<value>'",
                line_number, key, key
            ),
        });
    }

    let value = parts[1]
        .trim()
        .parse::<f64>()
        .map_err(|_| HotReloadError::ParseError {
            path: path.to_path_buf(),
            reason: format!(
                "第 {} 行: {} 值无法解析为数字: '{}'",
                line_number, key, parts[1]
            ),
        })?;

    Ok(Some(value))
}

/// 解析数据行
fn parse_data_line(
    line: &str,
    path: &Path,
    line_number: usize,
) -> Result<RatedLoadEntry, HotReloadError> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() != 3 {
        return Err(HotReloadError::ParseError {
            path: path.to_path_buf(),
            reason: format!(
                "第 {} 行: 数据行格式错误，应为 'boom_length,working_radius,rated_load'，实际列数: {}",
                line_number,
                parts.len()
            ),
        });
    }

    let boom_length = parts[0]
        .trim()
        .parse::<f64>()
        .map_err(|_| HotReloadError::ParseError {
            path: path.to_path_buf(),
            reason: format!(
                "第 {} 行: 臂长值无法解析为数字: '{}'",
                line_number, parts[0]
            ),
        })?;

    let working_radius =
        parts[1]
            .trim()
            .parse::<f64>()
            .map_err(|_| HotReloadError::ParseError {
                path: path.to_path_buf(),
                reason: format!(
                    "第 {} 行: 工作幅度值无法解析为数字: '{}'",
                    line_number, parts[1]
                ),
            })?;

    let rated_load = parts[2]
        .trim()
        .parse::<f64>()
        .map_err(|_| HotReloadError::ParseError {
            path: path.to_path_buf(),
            reason: format!(
                "第 {} 行: 额定载荷值无法解析为数字: '{}'",
                line_number, parts[2]
            ),
        })?;

    Ok(RatedLoadEntry {
        boom_length,
        working_radius,
        rated_load,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_rated_load_table_success() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("rated_load_table.csv");

        let content = r#"
# 额定负载表
moment_warning_threshold,85.0
moment_alarm_threshold,95.0

boom_length_m,working_radius_m,rated_load_ton
10.0,3.0,50.0
10.0,5.0,40.0
10.0,8.0,30.0
15.0,3.0,48.0
15.0,5.0,38.0
"#;

        fs::write(&csv_path, content).unwrap();

        let result = parse_rated_load_table(&csv_path);
        assert!(result.is_ok());

        let table = result.unwrap();
        assert_eq!(table.moment_warning_threshold, 85.0);
        assert_eq!(table.moment_alarm_threshold, 95.0);
        assert_eq!(table.get_rated_load(10.0, 5.0), 40.0);
        assert_eq!(table.get_rated_load(15.0, 3.0), 48.0);
    }

    #[test]
    fn test_parse_rated_load_table_with_comments() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("rated_load_table.csv");

        let content = r#"
# 这是注释行
# 另一个注释
moment_warning_threshold,85.0
moment_alarm_threshold,95.0

# 表头
boom_length_m,working_radius_m,rated_load_ton

# 数据行
10.0,3.0,50.0
# 中间的注释
10.0,5.0,40.0
"#;

        fs::write(&csv_path, content).unwrap();

        let result = parse_rated_load_table(&csv_path);
        assert!(result.is_ok());

        let table = result.unwrap();
        assert_eq!(table.get_rated_load(10.0, 5.0), 40.0);
    }

    #[test]
    fn test_parse_rated_load_table_empty() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("empty.csv");

        let content = r#"
# 只有注释
moment_warning_threshold,85.0
moment_alarm_threshold,95.0

boom_length_m,working_radius_m,rated_load_ton
"#;

        fs::write(&csv_path, content).unwrap();

        let result = parse_rated_load_table(&csv_path);
        assert!(result.is_err());

        match result {
            Err(HotReloadError::ParseError { reason, .. }) => {
                assert!(reason.contains("没有找到有效的数据行"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_rated_load_table_invalid_format() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("invalid.csv");

        let content = r#"
moment_warning_threshold,85.0
moment_alarm_threshold,95.0

boom_length_m,working_radius_m,rated_load_ton
10.0,3.0
"#;

        fs::write(&csv_path, content).unwrap();

        let result = parse_rated_load_table(&csv_path);
        assert!(result.is_err());

        match result {
            Err(HotReloadError::ParseError { reason, .. }) => {
                assert!(reason.contains("数据行格式错误"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_rated_load_table_invalid_number() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("invalid_number.csv");

        let content = r#"
moment_warning_threshold,85.0
moment_alarm_threshold,95.0

boom_length_m,working_radius_m,rated_load_ton
10.0,not_a_number,50.0
"#;

        fs::write(&csv_path, content).unwrap();

        let result = parse_rated_load_table(&csv_path);
        assert!(result.is_err());

        match result {
            Err(HotReloadError::ParseError { reason, .. }) => {
                assert!(reason.contains("无法解析为数字"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_rated_load_table_file_not_found() {
        let result = parse_rated_load_table(Path::new("/nonexistent/file.csv"));
        assert!(result.is_err());

        match result {
            Err(HotReloadError::FileRead { .. }) => {}
            _ => panic!("Expected FileRead error"),
        }
    }

    #[test]
    fn test_parse_rated_load_table_invalid_utf8() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("invalid_utf8.csv");

        // 写入无效的 UTF-8 字节序列
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        fs::write(&csv_path, invalid_utf8).unwrap();

        let result = parse_rated_load_table(&csv_path);
        assert!(result.is_err());

        match result {
            Err(HotReloadError::EncodingError { .. }) => {}
            _ => panic!("Expected EncodingError"),
        }
    }

    #[test]
    fn test_parse_threshold_invalid_format() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("invalid_threshold.csv");

        let content = r#"
moment_warning_threshold,85.0,extra
moment_alarm_threshold,95.0

boom_length_m,working_radius_m,rated_load_ton
10.0,3.0,50.0
"#;

        fs::write(&csv_path, content).unwrap();

        let result = parse_rated_load_table(&csv_path);
        assert!(result.is_err());

        match result {
            Err(HotReloadError::ParseError { reason, .. }) => {
                assert!(reason.contains("配置格式错误"));
            }
            _ => panic!("Expected ParseError"),
        }
    }
}
