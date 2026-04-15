use qt_rust_demo::logging::init_logging_from_file;
use qt_rust_demo::pipeline::pipeline_manager::PipelineManager;
use qt_rust_demo::repositories::CraneDataRepository;
/// 测试 PipelineManager 日志输出
///
/// 运行方式:
/// ```bash
/// cargo run --example test_pipeline_logging
/// ```
use std::sync::Arc;

fn main() {
    // 初始化日志系统（只显示 PipelineManager 的日志）
    match init_logging_from_file("config/logging.toml") {
        Ok(_) => {
            println!("✅ 日志系统初始化成功");
            println!("📋 配置: 只显示 qt_rust_demo::pipeline::pipeline_manager 模块的日志\n");
        }
        Err(e) => {
            // 日志系统未初始化，使用 eprintln 是合理的
            eprintln!("❌ 日志系统初始化失败: {}", e);
            eprintln!("使用默认配置");
            qt_rust_demo::logging::init_default_logging();
        }
    }

    println!("=== 开始测试 PipelineManager 日志 ===\n");

    // 创建数据仓库
    let repository = Arc::new(CraneDataRepository::default());

    // 创建管道管理器（应该看到日志）
    println!("1. 创建 PipelineManager:");
    let mut manager = PipelineManager::new(repository);
    println!();

    // 启动采集管道（应该看到日志）
    println!("2. 启动采集管道:");
    manager.start_collection_pipeline();
    println!();

    // 等待一段时间
    println!("3. 等待 2 秒，让管道运行...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    println!();

    // 获取队列信息（应该看到 trace 日志）
    println!("4. 查询队列状态:");
    let queue_len = manager.get_storage_queue_len();
    println!("   存储队列长度: {:?}", queue_len);

    let last_seq = manager.get_last_stored_sequence();
    println!("   最后序列号: {:?}", last_seq);
    println!();

    // 停止采集管道（应该看到日志）
    println!("5. 停止采集管道:");
    manager.stop_collection_pipeline();
    println!();

    // 停止所有管道（应该看到日志）
    println!("6. 停止所有管道:");
    manager.stop_all();
    println!();

    println!("=== 测试完成 ===");
    println!("\n💡 提示:");
    println!("   - 你应该看到带时间戳的 tracing 日志");
    println!("   - 日志格式: 时间 级别 模块名 文件:行号 消息");
    println!("   - 其他模块的日志不会显示（因为配置为 off）");
}
