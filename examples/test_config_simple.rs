// 简单的配置加载测试（不依赖库）

use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;

/// 管道系统完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PipelineConfig {
    collection: CollectionConfig,
    storage: StorageConfig,
    database: DatabaseConfig,
    simulator: SimulatorConfig,
    monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectionConfig {
    interval_ms: u64,
    buffer_size: usize,
    use_simulator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageConfig {
    interval_ms: u64,
    batch_size: usize,
    max_retries: u32,
    retry_delay_ms: u64,
    max_queue_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseConfig {
    path: String,
    enable_wal: bool,
    pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimulatorConfig {
    weight: SensorSimConfig,
    radius: SensorSimConfig,
    angle: SensorSimConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorSimConfig {
    amplitude: f64,
    frequency: f64,
    offset: f64,
    noise_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitoringConfig {
    enable: bool,
    stats_interval_sec: u64,
    verbose: bool,
}

impl PipelineConfig {
    fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: PipelineConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        Ok(config)
    }
    
    fn load() -> Self {
        let config_path = "config/pipeline_config.toml";
        
        match Self::from_file(config_path) {
            Ok(config) => {
                println!("[INFO] 已加载配置文件: {}", config_path);
                config
            }
            Err(e) => {
                eprintln!("[WARN] 配置加载失败: {}", e);
                eprintln!("[INFO] 使用默认配置");
                Self::default()
            }
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            collection: CollectionConfig {
                interval_ms: 100,
                buffer_size: 1000,
                use_simulator: true,
            },
            storage: StorageConfig {
                interval_ms: 1000,
                batch_size: 10,
                max_retries: 3,
                retry_delay_ms: 100,
                max_queue_size: 1000,
            },
            database: DatabaseConfig {
                path: "crane_data.db".to_string(),
                enable_wal: true,
                pool_size: 5,
            },
            simulator: SimulatorConfig {
                weight: SensorSimConfig {
                    amplitude: 5.0,
                    frequency: 0.5,
                    offset: 15.0,
                    noise_level: 0.1,
                },
                radius: SensorSimConfig {
                    amplitude: 3.0,
                    frequency: 0.3,
                    offset: 8.0,
                    noise_level: 0.05,
                },
                angle: SensorSimConfig {
                    amplitude: 10.0,
                    frequency: 0.2,
                    offset: 60.0,
                    noise_level: 0.5,
                },
            },
            monitoring: MonitoringConfig {
                enable: true,
                stats_interval_sec: 5,
                verbose: false,
            },
        }
    }
}

fn main() {
    println!("=== 管道配置加载测试 ===\n");
    
    // 加载配置
    let config = PipelineConfig::load();
    
    // 显示配置参数
    println!("\n[采集管道配置]");
    println!("  采集间隔: {}ms", config.collection.interval_ms);
    println!("  缓冲区大小: {}", config.collection.buffer_size);
    println!("  使用模拟器: {}", config.collection.use_simulator);
    
    println!("\n[存储管道配置]");
    println!("  存储间隔: {}ms", config.storage.interval_ms);
    println!("  批量大小: {}", config.storage.batch_size);
    println!("  最大重试: {}", config.storage.max_retries);
    println!("  重试延迟: {}ms", config.storage.retry_delay_ms);
    println!("  队列大小: {}", config.storage.max_queue_size);
    
    println!("\n[数据库配置]");
    println!("  数据库路径: {}", config.database.path);
    println!("  启用 WAL: {}", config.database.enable_wal);
    println!("  连接池大小: {}", config.database.pool_size);
    
    println!("\n[模拟器配置]");
    println!("  重量传感器:");
    println!("    振幅: {}", config.simulator.weight.amplitude);
    println!("    频率: {}", config.simulator.weight.frequency);
    println!("    偏移: {}", config.simulator.weight.offset);
    println!("    噪声: {}", config.simulator.weight.noise_level);
    
    println!("  半径传感器:");
    println!("    振幅: {}", config.simulator.radius.amplitude);
    println!("    频率: {}", config.simulator.radius.frequency);
    println!("    偏移: {}", config.simulator.radius.offset);
    println!("    噪声: {}", config.simulator.radius.noise_level);
    
    println!("  角度传感器:");
    println!("    振幅: {}", config.simulator.angle.amplitude);
    println!("    频率: {}", config.simulator.angle.frequency);
    println!("    偏移: {}", config.simulator.angle.offset);
    println!("    噪声: {}", config.simulator.angle.noise_level);
    
    println!("\n[性能监控配置]");
    println!("  启用监控: {}", config.monitoring.enable);
    println!("  统计间隔: {}秒", config.monitoring.stats_interval_sec);
    println!("  详细日志: {}", config.monitoring.verbose);
    
    // 测试 Duration 转换
    println!("\n[Duration 转换测试]");
    println!("  采集间隔: {:?}", Duration::from_millis(config.collection.interval_ms));
    println!("  存储间隔: {:?}", Duration::from_millis(config.storage.interval_ms));
    println!("  重试延迟: {:?}", Duration::from_millis(config.storage.retry_delay_ms));
    println!("  统计间隔: {:?}", Duration::from_secs(config.monitoring.stats_interval_sec));
    
    println!("\n[INFO] 配置加载成功！");
}
