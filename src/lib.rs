// Qt Rust Demo - 库接口
// 导出公共 API 供示例和测试使用

// 公开导出模块（无 Qt 依赖）
pub mod alarm;
pub mod algorithms;
pub mod config;
pub mod data_sources;
pub mod logging;
pub mod models;
pub mod pipeline;
pub mod repositories;

// 内部模块（现在也导出，供 qt-app 中的 cxx-qt 模块使用）
pub mod intents;
pub mod reducers;
pub mod states;
