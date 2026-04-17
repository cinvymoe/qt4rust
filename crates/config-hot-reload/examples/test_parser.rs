//! 测试配置解析器示例
//!
//! 演示如何使用 ConfigParser 解析各种配置文件

use config_hot_reload::parser::ConfigParser;
use std::path::Path;

fn main() {
    println!("=== 配置解析器测试 ===\n");

    // 1. 解析传感器校准配置
    println!("1. 解析传感器校准配置...");
    match ConfigParser::parse_sensor_calibration(Path::new("config/sensor_calibration.toml")) {
        Ok(config) => {
            println!("   ✓ 成功解析传感器校准配置");
            println!(
                "     - 重量传感器: zero_ad={}, scale_ad={}, scale_value={}",
                config.weight().zero_ad,
                config.weight().scale_ad,
                config.weight().scale_value
            );
            println!(
                "     - 角度传感器: zero_ad={}, scale_ad={}, scale_value={}",
                config.angle().zero_ad,
                config.angle().scale_ad,
                config.angle().scale_value
            );
            println!(
                "     - 半径传感器: zero_ad={}, scale_ad={}, scale_value={}",
                config.radius().zero_ad,
                config.radius().scale_ad,
                config.radius().scale_value
            );
        }
        Err(e) => {
            println!("   ✗ 解析失败: {}", e);
        }
    }
    println!();

    // 2. 解析报警阈值配置
    println!("2. 解析报警阈值配置...");
    match ConfigParser::parse_alarm_thresholds(Path::new("config/alarm_thresholds.toml")) {
        Ok(config) => {
            println!("   ✓ 成功解析报警阈值配置");
            println!("     - 预警阈值: {}%", config.moment.warning_percentage);
            println!("     - 报警阈值: {}%", config.moment.alarm_percentage);
        }
        Err(e) => {
            println!("   ✗ 解析失败: {}", e);
        }
    }
    println!();

    // 3. 解析日志配置
    println!("3. 解析日志配置...");
    match ConfigParser::parse_logging_config(Path::new("config/logging.toml")) {
        Ok(config) => {
            println!("   ✓ 成功解析日志配置");
            println!("     - 默认级别: {:?}", config.default_level);
            println!("     - 控制台输出: {}", config.console_output);
            println!("     - 文件输出: {}", config.file_output);
            println!("     - 日志文件: {}", config.log_file);
            println!("     - 模块配置数量: {}", config.modules.len());
        }
        Err(e) => {
            println!("   ✗ 解析失败: {}", e);
        }
    }
    println!();

    // 4. 解析管道配置
    println!("4. 解析管道配置...");
    match ConfigParser::parse_pipeline_config(Path::new("config/pipeline_config.toml")) {
        Ok(config) => {
            println!("   ✓ 成功解析管道配置");
            println!("     - 采集间隔: {}ms", config.collection.interval_ms);
            println!("     - 存储间隔: {}ms", config.storage.interval_ms);
            println!("     - 批量大小: {}", config.storage.batch_size);
            println!("     - 使用模拟器: {}", config.collection.use_simulator);
        }
        Err(e) => {
            println!("   ✗ 解析失败: {}", e);
        }
    }
    println!();

    // 5. 解析额定负载表
    println!("5. 解析额定负载表...");
    match ConfigParser::parse_rated_load_table(Path::new("config/rated_load_table.csv")) {
        Ok(table) => {
            println!("   ✓ 成功解析额定负载表");
            println!("     - 预警阈值: {}%", table.moment_warning_threshold);
            println!("     - 报警阈值: {}%", table.moment_alarm_threshold);
            println!("     - 数据条目数: {}", table.len());
            println!("     - 臂长列表: {:?}", table.get_boom_lengths());

            // 测试查询
            let boom_length = 20.0;
            let working_radius = 10.0;
            let rated_load = table.get_rated_load(boom_length, working_radius);
            println!(
                "     - 查询示例: 臂长{}m, 幅度{}m -> 额定载荷{}吨",
                boom_length, working_radius, rated_load
            );
        }
        Err(e) => {
            println!("   ✗ 解析失败: {}", e);
        }
    }
    println!();

    println!("=== 测试完成 ===");
}
