//! 文件监控器实现
//!
//! 使用 notify crate 监控配置文件变化，过滤临时文件，
//! 并通过 debounce 机制避免重复事件。

use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{types::ConfigFileEvent, types::ConfigFileType, HotReloadError};

/// 文件监控器
///
/// 监控配置目录中的文件变化，过滤临时文件，
/// 并通过 channel 发送配置文件变更事件。
pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    config_dir: PathBuf,
    event_tx: mpsc::Sender<ConfigFileEvent>,
    debounce_duration: Duration,
}

impl FileWatcher {
    /// 创建新的文件监控器
    ///
    /// # 参数
    ///
    /// * `config_dir` - 配置文件目录路径
    /// * `event_tx` - 配置文件变更事件发送通道
    ///
    /// # 返回
    ///
    /// 返回 FileWatcher 实例或错误
    pub fn new(
        config_dir: PathBuf,
        event_tx: mpsc::Sender<ConfigFileEvent>,
    ) -> Result<Self, HotReloadError> {
        if !config_dir.exists() {
            return Err(HotReloadError::WatcherError(format!(
                "配置目录不存在: {}",
                config_dir.display()
            )));
        }

        if !config_dir.is_dir() {
            return Err(HotReloadError::WatcherError(format!(
                "路径不是目录: {}",
                config_dir.display()
            )));
        }

        Ok(Self {
            watcher: None,
            config_dir,
            event_tx,
            debounce_duration: Duration::from_millis(300),
        })
    }

    /// 启动文件监控
    ///
    /// 在独立的 tokio 任务中运行文件监控，不阻塞主线程。
    ///
    /// # 返回
    ///
    /// 成功返回 Ok(())，失败返回错误
    pub async fn start(&mut self) -> Result<(), HotReloadError> {
        let config_dir = self.config_dir.clone();
        let event_tx = self.event_tx.clone();
        let debounce_duration = self.debounce_duration;

        // 创建 channel 用于从 notify 回调发送事件到 tokio 任务
        let (notify_tx, mut notify_rx) = mpsc::unbounded_channel::<Event>();

        // 创建 notify watcher
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if notify_tx.send(event).is_err() {
                            error!("Failed to send file event to processing task");
                        }
                    }
                    Err(e) => {
                        error!("File watcher error: {}", e);
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| HotReloadError::WatcherError(format!("创建文件监控器失败: {}", e)))?;

        self.watcher = Some(watcher);

        // 监控配置目录
        if let Some(watcher) = &mut self.watcher {
            watcher
                .watch(&config_dir, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    HotReloadError::WatcherError(format!("监控目录失败: {}", e))
                })?;
        }

        info!("文件监控器已启动，监控目录: {}", config_dir.display());

        // 在独立的 tokio 任务中处理文件事件
        tokio::spawn(async move {
            let mut last_event_time: std::collections::HashMap<PathBuf, std::time::Instant> =
                std::collections::HashMap::new();

            while let Some(event) = notify_rx.recv().await {
                // 处理文件事件
                if let EventKind::Modify(_) | EventKind::Create(_) = event.kind {
                    for path in event.paths {
                        // 过滤临时文件
                        if !Self::should_watch_static(&path) {
                            debug!("忽略临时文件: {}", path.display());
                            continue;
                        }

                        // Debounce: 检查是否在 debounce 时间窗口内
                        let now = std::time::Instant::now();
                        if let Some(&last_time) = last_event_time.get(&path) {
                            if now.duration_since(last_time) < debounce_duration {
                                debug!("Debounce: 忽略重复事件 {}", path.display());
                                continue;
                            }
                        }
                        last_event_time.insert(path.clone(), now);

                        // 识别配置文件类型
                        if let Some(file_type) = Self::identify_config_file(&path) {
                            info!("检测到配置文件变更: {:?} - {}", file_type, path.display());

                            let config_event = ConfigFileEvent {
                                file_type,
                                path: path.clone(),
                                timestamp: std::time::SystemTime::now(),
                            };

                            if event_tx.send(config_event).await.is_err() {
                                error!("发送配置文件变更事件失败");
                                break;
                            }
                        }
                    }
                } else if let EventKind::Remove(_) = event.kind {
                    for path in event.paths {
                        warn!("配置文件被删除: {}", path.display());
                    }
                }
            }

            debug!("文件监控任务已停止");
        });

        Ok(())
    }

    /// 停止文件监控
    pub fn stop(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            if let Err(e) = watcher.unwatch(&self.config_dir) {
                error!("停止监控目录失败: {}", e);
            }
            info!("文件监控器已停止");
        }
    }

    /// 检查文件是否应该被监控（过滤临时文件）
    ///
    /// 过滤以下临时文件后缀：
    /// - .swp (Vim swap files)
    /// - .bak (Backup files)
    /// - .tmp (Temporary files)
    /// - .~ (Emacs backup files)
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// 如果文件应该被监控返回 true，否则返回 false
    pub fn should_watch(&self, path: &Path) -> bool {
        Self::should_watch_static(path)
    }

    /// 静态版本的 should_watch，用于在 tokio 任务中调用
    fn should_watch_static(path: &Path) -> bool {
        // 检查文件扩展名
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if matches!(ext, "swp" | "bak" | "tmp") {
                return false;
            }
        }

        // 检查文件名是否以 ~ 结尾
        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
            if file_name.ends_with('~') {
                return false;
            }
        }

        true
    }

    /// 根据文件名识别配置文件类型
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// 如果识别成功返回 Some(ConfigFileType)，否则返回 None
    fn identify_config_file(path: &Path) -> Option<ConfigFileType> {
        let file_name = path.file_name()?.to_str()?;

        match file_name {
            "sensor_calibration.toml" => Some(ConfigFileType::SensorCalibration),
            "alarm_thresholds.toml" => Some(ConfigFileType::AlarmThresholds),
            "logging.toml" => Some(ConfigFileType::Logging),
            "modbus_sensors.toml" => Some(ConfigFileType::ModbusSensors),
            "pipeline_config.toml" => Some(ConfigFileType::Pipeline),
            "rated_load_table.csv" => Some(ConfigFileType::RatedLoadTable),
            _ => None,
        }
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_watch_filters_temp_files() {
        // 测试过滤 .swp 文件
        assert!(!FileWatcher::should_watch_static(Path::new(
            "config/test.swp"
        )));

        // 测试过滤 .bak 文件
        assert!(!FileWatcher::should_watch_static(Path::new(
            "config/test.bak"
        )));

        // 测试过滤 .tmp 文件
        assert!(!FileWatcher::should_watch_static(Path::new(
            "config/test.tmp"
        )));

        // 测试过滤 ~ 结尾的文件
        assert!(!FileWatcher::should_watch_static(Path::new(
            "config/test.toml~"
        )));

        // 测试正常文件不被过滤
        assert!(FileWatcher::should_watch_static(Path::new(
            "config/sensor_calibration.toml"
        )));
    }

    #[test]
    fn test_identify_config_file() {
        assert_eq!(
            FileWatcher::identify_config_file(Path::new("config/sensor_calibration.toml")),
            Some(ConfigFileType::SensorCalibration)
        );

        assert_eq!(
            FileWatcher::identify_config_file(Path::new("config/alarm_thresholds.toml")),
            Some(ConfigFileType::AlarmThresholds)
        );

        assert_eq!(
            FileWatcher::identify_config_file(Path::new("config/logging.toml")),
            Some(ConfigFileType::Logging)
        );

        assert_eq!(
            FileWatcher::identify_config_file(Path::new("config/modbus_sensors.toml")),
            Some(ConfigFileType::ModbusSensors)
        );

        assert_eq!(
            FileWatcher::identify_config_file(Path::new("config/pipeline_config.toml")),
            Some(ConfigFileType::Pipeline)
        );

        assert_eq!(
            FileWatcher::identify_config_file(Path::new("config/rated_load_table.csv")),
            Some(ConfigFileType::RatedLoadTable)
        );

        assert_eq!(
            FileWatcher::identify_config_file(Path::new("config/unknown.toml")),
            None
        );
    }
}
