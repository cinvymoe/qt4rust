// 完整采集管道示例 - 演示全局运行时、数据采集和共享缓冲区

use qt_threading_utils::prelude::*;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// 模拟传感器数据
#[derive(Debug, Clone)]
struct SensorData {
    load: f64,
    radius: f64,
    #[allow(dead_code)]
    timestamp: std::time::SystemTime,
}

/// 共享数据缓冲区
struct DataBuffer {
    latest: Option<SensorData>,
    history: VecDeque<SensorData>,
    max_size: usize,
}

impl DataBuffer {
    fn new(max_size: usize) -> Self {
        Self {
            latest: None,
            history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn push(&mut self, data: SensorData) {
        self.latest = Some(data.clone());

        if self.history.len() >= self.max_size {
            self.history.pop_front();
        }
        self.history.push_back(data);
    }

    fn get_latest(&self) -> Option<SensorData> {
        self.latest.clone()
    }

    fn get_count(&self) -> usize {
        self.history.len()
    }
}

fn main() {
    println!("=== 采集管道示例 ===\n");

    // 1. 创建共享缓冲区
    let buffer = Arc::new(RwLock::new(DataBuffer::new(100)));
    println!("✓ 共享缓冲区已创建（容量: 100）");

    // 2. 创建数据采集器（使用全局运行时）
    let collector = BlockingDataCollector::new(Duration::from_millis(100));

    let buffer_clone = Arc::clone(&buffer);
    let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let counter_clone = Arc::clone(&counter);

    collector.start(move || {
        let count = counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // 模拟传感器数据采集
        let data = SensorData {
            load: 10.0 + 5.0 * (count as f64 * 0.1).sin(),
            radius: 8.0 + 2.0 * (count as f64 * 0.05).cos(),
            timestamp: std::time::SystemTime::now(),
        };

        // 写入共享缓冲区
        if let Ok(mut buf) = buffer_clone.write() {
            buf.push(data.clone());

            // 每 10 次采集打印一次
            if count % 10 == 0 {
                println!(
                    "[采集 #{}] 载荷: {:.2}t, 半径: {:.2}m, 缓冲区: {} 条",
                    count,
                    data.load,
                    data.radius,
                    buf.get_count()
                );
            }
        }
    });

    println!("✓ 数据采集器已启动（100ms 间隔）\n");

    // 3. 创建数据读取定时器（模拟 UI 更新）
    let reader_timer = BlockingPeriodicTimer::new(Duration::from_millis(500));

    let buffer_clone = Arc::clone(&buffer);
    reader_timer.start(move || {
        if let Ok(buf) = buffer_clone.read() {
            if let Some(latest) = buf.get_latest() {
                println!(
                    "[UI 更新] 最新数据: 载荷={:.2}t, 半径={:.2}m, 历史记录={} 条",
                    latest.load,
                    latest.radius,
                    buf.get_count()
                );
            }
        }
    });

    println!("✓ UI 更新定时器已启动（500ms 间隔）\n");
    println!("运行 5 秒后自动停止...\n");

    // 4. 运行一段时间
    std::thread::sleep(Duration::from_secs(5));

    // 5. 停止采集和定时器
    collector.stop();
    reader_timer.stop();

    // 6. 显示最终统计
    if let Ok(buf) = buffer.read() {
        println!("\n=== 采集完成 ===");
        println!("总采集次数: {}", buf.get_count());
        if let Some(latest) = buf.get_latest() {
            println!(
                "最后数据: 载荷={:.2}t, 半径={:.2}m",
                latest.load, latest.radius
            );
        }
    }

    println!("\n✓ 示例完成");
}
