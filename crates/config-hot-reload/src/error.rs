//! 错误类型定义

use std::path::PathBuf;
use thiserror::Error;

/// 配置热加载错误类型
#[derive(Debug, Error)]
pub enum HotReloadError {
    #[error("文件读取失败: {path}, 原因: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("配置解析失败: {path}, 原因: {reason}")]
    ParseError { path: PathBuf, reason: String },

    #[error("配置验证失败: {file_type:?}, 原因: {source}")]
    ValidationFailed {
        file_type: crate::types::ConfigFileType,
        #[source]
        source: ValidationError,
    },

    #[error("文件监控失败: {0}")]
    WatcherError(String),

    #[error("配置更新失败: {0}")]
    UpdateFailed(String),

    #[error("编码错误: {path}, 原因: {source}")]
    EncodingError {
        path: PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },
}

/// 配置验证错误类型
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("字段 {field} 验证失败: {reason}")]
    FieldValidation { field: String, reason: String },

    #[error("配置不一致: {0}")]
    Inconsistency(String),
}
