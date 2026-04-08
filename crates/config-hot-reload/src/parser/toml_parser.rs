//! TOML 配置文件解析器

use std::fs;
use std::path::Path;
use serde::de::DeserializeOwned;
use crate::error::HotReloadError;

/// 解析 TOML 配置文件
///
/// # 参数
/// - `path`: 配置文件路径
///
/// # 返回
/// - `Ok(T)`: 解析成功，返回配置对象
/// - `Err(HotReloadError)`: 解析失败，返回错误信息
///
/// # 错误处理
/// - 文件读取失败: 返回 `FileRead` 错误
/// - 编码错误: 返回 `EncodingError` 错误
/// - TOML 解析失败: 返回 `ParseError` 错误，包含详细的错误原因和行号信息
///
/// # 示例
/// ```no_run
/// use config_hot_reload::parser::toml_parser;
/// use std::path::Path;
///
/// #[derive(serde::Deserialize)]
/// struct MyConfig {
///     value: i32,
/// }
///
/// let config: MyConfig = toml_parser::parse_toml(Path::new("config.toml")).unwrap();
/// ```
pub fn parse_toml<T: DeserializeOwned>(path: &Path) -> Result<T, HotReloadError> {
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
    
    // 3. 检查文件是否为空或只包含空白字符
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(HotReloadError::ParseError {
            path: path.to_path_buf(),
            reason: "文件为空或只包含空白字符（可能是编辑器保存过程中的临时状态）".to_string(),
        });
    }
    
    // 4. 解析 TOML
    toml::from_str(&content).map_err(|e| {
        // 提取详细的错误信息，包含行号
        let reason = format!("TOML 解析失败: {}", e);
        HotReloadError::ParseError {
            path: path.to_path_buf(),
            reason,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::fs;
    use tempfile::TempDir;
    
    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
        enabled: bool,
    }
    
    #[test]
    fn test_parse_toml_success() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        
        let content = r#"
name = "test"
value = 42
enabled = true
"#;
        
        fs::write(&config_path, content).unwrap();
        
        let result: Result<TestConfig, _> = parse_toml(&config_path);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
        assert_eq!(config.enabled, true);
    }
    
    #[test]
    fn test_parse_toml_file_not_found() {
        let result: Result<TestConfig, _> = parse_toml(Path::new("/nonexistent/file.toml"));
        assert!(result.is_err());
        
        match result {
            Err(HotReloadError::FileRead { path, .. }) => {
                assert_eq!(path, Path::new("/nonexistent/file.toml"));
            }
            _ => panic!("Expected FileRead error"),
        }
    }
    
    #[test]
    fn test_parse_toml_invalid_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");
        
        let content = "invalid toml [[[";
        fs::write(&config_path, content).unwrap();
        
        let result: Result<TestConfig, _> = parse_toml(&config_path);
        assert!(result.is_err());
        
        match result {
            Err(HotReloadError::ParseError { path, reason }) => {
                assert_eq!(path, config_path);
                assert!(reason.contains("TOML"));
            }
            _ => panic!("Expected ParseError"),
        }
    }
    
    #[test]
    fn test_parse_toml_invalid_utf8() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid_utf8.toml");
        
        // 写入无效的 UTF-8 字节序列
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        fs::write(&config_path, invalid_utf8).unwrap();
        
        let result: Result<TestConfig, _> = parse_toml(&config_path);
        assert!(result.is_err());
        
        match result {
            Err(HotReloadError::EncodingError { path, .. }) => {
                assert_eq!(path, config_path);
            }
            _ => panic!("Expected EncodingError"),
        }
    }
    
    #[test]
    fn test_parse_toml_type_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("type_mismatch.toml");
        
        let content = r#"
name = "test"
value = "not_a_number"
enabled = true
"#;
        
        fs::write(&config_path, content).unwrap();
        
        let result: Result<TestConfig, _> = parse_toml(&config_path);
        assert!(result.is_err());
        
        match result {
            Err(HotReloadError::ParseError { .. }) => {}
            _ => panic!("Expected ParseError"),
        }
    }
    
    #[test]
    fn test_parse_toml_missing_field() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("missing_field.toml");
        
        let content = r#"
name = "test"
value = 42
"#;
        
        fs::write(&config_path, content).unwrap();
        
        let result: Result<TestConfig, _> = parse_toml(&config_path);
        assert!(result.is_err());
        
        match result {
            Err(HotReloadError::ParseError { .. }) => {}
            _ => panic!("Expected ParseError"),
        }
    }
}
