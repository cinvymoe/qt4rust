// 阻塞式 API 使用示例（适用于 Qt 信号槽）

use qt_threading_utils::prelude::*;
use std::time::Duration;

fn main() {
    println!("=== Qt Threading Utils 阻塞式 API 示例 ===\n");

    // 1. 阻塞式周期定时器
    println!("1. 阻塞式周期定时器示例");
    let timer = BlockingPeriodicTimer::new(Duration::from_millis(200));
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter_clone = counter.clone();
    
    timer.start(move || {
        let count = counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        println!("   定时器触发 #{}", count + 1);
    });

    std::thread::sleep(Duration::from_secs(1));
    timer.stop();
    println!("   定时器已停止");

    // 2. 阻塞式数据采集器
    println!("\n2. 阻塞式数据采集器示例");
    let collector = BlockingDataCollector::new(Duration::from_millis(150));
    
    let data_counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let data_counter_clone = data_counter.clone();
    
    collector.start(move || {
        let count = data_counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        println!("   采集数据 #{}", count + 1);
    });

    std::thread::sleep(Duration::from_secs(1));
    collector.stop();
    println!("   采集器已停止");

    // 3. 使用全局运行时执行异步任务
    println!("\n3. 全局运行时阻塞执行示例");
    block_on(async {
        println!("   在全局运行时中执行异步任务");
        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("   异步任务完成");
    });

    println!("\n=== 示例完成 ===");
}
