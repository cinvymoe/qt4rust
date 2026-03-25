// 诊断存储管道卡住的问题

use qt_rust_demo::repositories::CraneDataRepository;
use qt_rust_demo::pipeline::PipelineManager;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // 初始化日志
    qt_rust_demo::logging::init_default_logging();
    
    tracing::info!("=== 存储管道诊断工具 ===");
    
    // 创建数据仓库
    let repository = Arc::new(CraneDataRepository::default());
    
    // 创建管道管理器
    let db_path = "crane_data.db";
    let mut manager = match PipelineManager::new_with_storage(repository, db_path).await {
        Ok(mgr) => {
            tracing::info!("✓ 管道管理器创建成功");
            mgr
        }
        Err(e) => {
            tracing::error!("✗ 管道管理器创建失败: {}", e);
            return;
        }
    };
    
    // 启动所有管道
    tracing::info!("启动所有管道...");
    manager.start_all();
    
    // 等待一段时间,观察日志
    tracing::info!("等待 10 秒,观察管道运行情况...");
    tokio::time::sleep(Duration::from_secs(10)).await;
    
    // 检查 buffer 状态
    let buffer = manager.get_shared_buffer();
    if let Ok(buf) = buffer.read() {
        let history = buf.get_history(10);
        tracing::info!("Buffer 中最新 10 条数据:");
        for data in history.iter().rev().take(10) {
            tracing::info!("  seq={}, load={:.1}, moment={:.1}%", 
                data.sequence_number, data.ad1_load, data.moment_percentage);
        }
        
        if let Some(latest) = history.last() {
            tracing::info!("最新序列号: {}", latest.sequence_number);
        }
    }
    
    // 停止管道
    tracing::info!("停止所有管道...");
    manager.stop_all();
    
    tracing::info!("=== 诊断完成 ===");
}
