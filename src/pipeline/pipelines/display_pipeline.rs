// 显示管道（主线程版本）

use crate::models::ProcessedData;
use crate::pipeline::infrastructure::SharedBuffer;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 显示管道配置
#[derive(Debug, Clone)]
pub struct DisplayPipelineConfig {
    /// 采集间隔
    pub interval: Duration,

    /// 管道大小
    pub pipeline_size: usize,

    /// 每次采集数量
    pub batch_size: usize,
}

impl Default for DisplayPipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(500),
            pipeline_size: 10,
            batch_size: 1,
        }
    }
}

impl DisplayPipelineConfig {
    pub fn from_display_config(config: &crate::config::pipeline_config::DisplayConfig) -> Self {
        Self {
            interval: Duration::from_millis(config.interval_ms),
            pipeline_size: config.pipeline_size,
            batch_size: config.batch_size,
        }
    }
}

/// 显示管道（主线程版本）
///
/// 这个管道设计用于在主线程（Qt事件循环）中运行
/// 不创建后台线程，而是提供 tick() 方法供主循环调用
pub struct DisplayPipeline {
    config: DisplayPipelineConfig,
    buffer: SharedBuffer,
    /// 内部显示缓冲区
    display_buffer: VecDeque<ProcessedData>,
    /// 上次更新时间
    last_update: Option<Instant>,
    /// 是否运行中
    running: bool,
}

impl DisplayPipeline {
    pub fn new(config: DisplayPipelineConfig, buffer: SharedBuffer) -> Self {
        let pipeline_size = config.pipeline_size;
        Self {
            config,
            buffer,
            display_buffer: VecDeque::with_capacity(pipeline_size),
            last_update: None,
            running: false,
        }
    }

    /// 启动管道
    pub fn start(&mut self) {
        self.running = true;
        tracing::info!("Display pipeline started (main thread mode)");
    }

    /// 停止管道
    pub fn stop(&mut self) {
        self.running = false;
        tracing::info!("Display pipeline stopped");
    }

    /// 检查是否运行中
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// 主循环调用：更新显示数据
    ///
    /// 应在主线程的定时器或事件循环中定期调用
    /// 返回是否需要刷新UI
    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }

        let now = Instant::now();

        // 检查是否到达更新时间（首次调用总是更新）
        if let Some(last) = self.last_update {
            if now.duration_since(last) < self.config.interval {
                return false;
            }
        }

        self.last_update = Some(now);

        // 从共享缓冲区读取数据
        if let Ok(buf) = self.buffer.read() {
            // 获取最新数据
            if let Some(latest) = buf.get_latest() {
                // 添加到显示缓冲区
                if self.display_buffer.len() >= self.config.pipeline_size {
                    self.display_buffer.pop_front();
                }
                self.display_buffer.push_back(latest);

                tracing::debug!(
                    "Display pipeline: fetched latest data, buffer size = {}",
                    self.display_buffer.len()
                );
                return true;
            }
        }

        false
    }

    /// 获取最新数据
    pub fn get_latest(&self) -> Option<ProcessedData> {
        self.display_buffer.back().cloned()
    }

    /// 获取历史数据
    pub fn get_history(&self, count: usize) -> Vec<ProcessedData> {
        let start = self.display_buffer.len().saturating_sub(count);
        self.display_buffer.iter().skip(start).cloned().collect()
    }

    /// 获取显示缓冲区大小
    pub fn len(&self) -> usize {
        self.display_buffer.len()
    }

    /// 检查显示缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.display_buffer.is_empty()
    }
}

impl Drop for DisplayPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SensorData;
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::Duration;

    fn create_test_data(seq: u64) -> ProcessedData {
        let sensor_data = SensorData::new(10.0 + seq as f64, 8.0, 60.0, false, false);
        ProcessedData::from_sensor_data(sensor_data, seq)
    }

    #[test]
    fn test_display_pipeline_default_config() {
        let config = DisplayPipelineConfig::default();
        assert_eq!(config.interval, Duration::from_millis(500));
        assert_eq!(config.pipeline_size, 10);
        assert_eq!(config.batch_size, 1);
    }

    #[test]
    fn test_display_pipeline_start_stop() {
        let buffer = Arc::new(RwLock::new(
            crate::pipeline::infrastructure::ProcessedDataBuffer::new(100),
        ));
        let config = DisplayPipelineConfig::default();
        let mut pipeline = DisplayPipeline::new(config, buffer);

        assert!(!pipeline.is_running());

        pipeline.start();
        assert!(pipeline.is_running());

        pipeline.stop();
        assert!(!pipeline.is_running());
    }

    #[test]
    fn test_display_pipeline_tick() {
        let buffer = Arc::new(RwLock::new(
            crate::pipeline::infrastructure::ProcessedDataBuffer::new(100),
        ));

        // 添加测试数据到缓冲区
        {
            let mut buf = buffer.write().unwrap();
            buf.push(create_test_data(1));
            buf.push(create_test_data(2));
        }

        let config = DisplayPipelineConfig::default();
        let mut pipeline = DisplayPipeline::new(config, buffer);

        pipeline.start();

        // 第一次 tick 应该获取数据
        let has_update = pipeline.tick();
        assert!(has_update);
        assert_eq!(pipeline.len(), 1);

        // 获取最新数据
        let latest = pipeline.get_latest();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().sequence_number, 2);

        pipeline.stop();
    }

    #[test]
    fn test_display_pipeline_buffer_overflow() {
        let buffer = Arc::new(RwLock::new(
            crate::pipeline::infrastructure::ProcessedDataBuffer::new(100),
        ));

        // 添加超过管道大小的数据
        {
            let mut buf = buffer.write().unwrap();
            for i in 1..=15 {
                buf.push(create_test_data(i));
            }
        }

        let mut config = DisplayPipelineConfig::default();
        config.pipeline_size = 10;
        let mut pipeline = DisplayPipeline::new(config, buffer);

        pipeline.start();

        // 触发多次 tick 以填充缓冲区
        for _ in 0..15 {
            pipeline.tick();
            thread::sleep(Duration::from_millis(10));
        }

        // 缓冲区大小应该被限制在 pipeline_size
        assert!(pipeline.len() <= 10);

        pipeline.stop();
    }
}
