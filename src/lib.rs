// Qt Rust Demo - 库接口
// 导出公共 API 供示例和测试使用

// 公开导出模块（无 Qt 依赖）
pub mod config;
pub mod models;
pub mod repositories;
pub mod pipeline;
pub mod data_sources;
pub mod algorithms;

// 内部模块（现在也导出，供 qt-app 中的 cxx-qt 模块使用）
pub mod states;
pub mod intents;
pub mod reducers;
