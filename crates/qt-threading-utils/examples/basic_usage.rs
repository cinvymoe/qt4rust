// 基本使用示例

use qt_threading_utils::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("=== Qt Threading Utils 示例 ===\n");

    // 1. 使用全局运行时
    println!("1. 全局运行时示例");
    let runtime = global_runtime();
    runtime.spawn(async {
        println!("   后台任务执行中...");
    });

    // 2. 异步周期定时器
    println!("\n2. 异步周期定时器示例");
    let timer = PeriodicTimer::new(Duration::from_millis(500));
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter_clone = counter.clone();

    timer
        .start(move || {
            let count = counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!("   定时器触发 #{}", count + 1);
        })
        .await;

    tokio::time::sleep(Duration::from_secs(2)).await;
    timer.stop().await;
    println!("   定时器已停止");

    // 3. 单次定时器
    println!("\n3. 单次定时器示例");
    let oneshot = OneShotTimer::new(Duration::from_secs(1));
    oneshot
        .start(|| {
            println!("   延迟任务执行！");
        })
        .await;

    // 4. 异步数据采集器
    println!("\n4. 异步数据采集器示例");
    let collector = DataCollector::new(Duration::from_millis(300));

    let data_counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let data_counter_clone = data_counter.clone();

    collector
        .start(move || {
            let count = data_counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!("   采集数据 #{}", count + 1);
        })
        .await;

    tokio::time::sleep(Duration::from_secs(1)).await;
    collector.stop().await;
    println!("   采集器已停止");

    println!("\n=== 示例完成 ===");
}
