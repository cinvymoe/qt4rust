// ViewModel 管理器 - 管理全局 ViewModel 实例和数据采集

use config_hot_reload::{register_all_subscribers, HotReloadConfigManager, SharedConfigRefs};
use qt_rust_demo::pipeline::shared_buffer::SharedBuffer;
use qt_rust_demo::pipeline::shared_sensor_buffer::SharedSensorBuffer;
use qt_rust_demo::pipeline::PipelineManager;
use qt_rust_demo::repositories::CraneDataRepository;
use sensor_core::{DataSourceId, PipelineConfig, SensorPipelineManager};
use sensor_simulator::prelude::SimulatedDataSource;
use std::sync::{Arc, Mutex};

/// ViewModel 管理器
pub struct ViewModelManager {
    pipeline_manager: Option<PipelineManager>,
    shared_buffer: Option<SharedBuffer>,
    shared_sensor_buffer: Option<SharedSensorBuffer>,
    viewmodel_ready: bool,
    hot_reload_manager: Option<HotReloadConfigManager>,
    shared_config_refs: Option<SharedConfigRefs>,
    sensor_manager: Option<SensorPipelineManager>,
}

impl ViewModelManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            pipeline_manager: None,
            shared_buffer: None,
            shared_sensor_buffer: None,
            viewmodel_ready: false,
            hot_reload_manager: None,
            shared_config_refs: None,
            sensor_manager: None,
        }
    }

    /// 标记 ViewModel 已准备好
    #[allow(dead_code)]
    pub fn mark_viewmodel_ready(&mut self) {
        tracing::info!("ViewModel marked as ready");
        self.viewmodel_ready = true;
    }

    /// 启动数据采集
    pub fn start_data_collection(&mut self) {
        tracing::info!("Starting three-pipeline data collection...");

        // 创建数据仓库（使用 Default 实现，自动创建 ConfigManager）
        let repository = Arc::new(CraneDataRepository::default());

        // 初始化热加载配置管理器
        let config_dir = std::path::PathBuf::from("config");
        let hot_reload_config = match HotReloadConfigManager::new(config_dir) {
            Ok(mut hot_reload_manager) => {
                tracing::info!("热加载配置管理器初始化成功");

                // 创建共享配置引用（使用默认配置初始化）
                let shared_refs = SharedConfigRefs::default();

                // 注册所有订阅者
                qt_threading_utils::runtime::global_runtime().block_on(async {
                    register_all_subscribers(&mut hot_reload_manager, &shared_refs).await;
                });

                // 启动热加载服务
                qt_threading_utils::runtime::global_runtime().block_on(async {
                    if let Err(e) = hot_reload_manager.start().await {
                        tracing::error!("启动热加载服务失败: {}", e);
                    } else {
                        tracing::info!("热加载服务已启动，监控配置文件变化");
                    }
                });

                self.hot_reload_manager = Some(hot_reload_manager);
                self.shared_config_refs = Some(shared_refs.clone());

                Some(shared_refs)
            }
            Err(e) => {
                tracing::error!("热加载配置管理器初始化失败: {}", e);
                tracing::warn!("继续使用静态配置");
                None
            }
        };

        // 创建管道管理器（带存储支持）
        let db_path = "crane_data.db";
        tracing::info!("Initializing storage system with database: {}", db_path);

        let manager = match qt_threading_utils::runtime::global_runtime()
            .block_on(async { PipelineManager::new_with_storage(repository, db_path).await })
        {
            Ok(mut mgr) => {
                tracing::info!("Storage system initialized successfully");

                // 如果有热重载配置，设置到PipelineManager（在启动管道之前）
                if let Some(ref shared_refs) = hot_reload_config {
                    tracing::info!("🚀 [ViewModelManager] 正在设置热重载配置到PipelineManager...");
                    mgr.set_hot_reload_config(
                        Arc::clone(&shared_refs.sensor_calibration),
                        Arc::clone(&shared_refs.rated_load_table),
                        Arc::clone(&shared_refs.alarm_thresholds),
                    );
                    tracing::info!("✅ [ViewModelManager] 热重载配置已成功设置到PipelineManager");
                } else {
                    tracing::warn!("⚠️  [ViewModelManager] 没有热重载配置可设置");
                }

                // 启动所有管道（在设置热重载配置之后）
                tracing::info!("🚀 [ViewModelManager] 正在启动所有管道...");
                mgr.start_all();
                tracing::info!("✅ [ViewModelManager] 所有管道已启动");

                mgr
            }
            Err(e) => {
                tracing::error!("Failed to initialize storage system: {}", e);
                tracing::warn!("Falling back to collection-only mode");
                let repository = Arc::new(CraneDataRepository::default());
                let mut mgr = PipelineManager::new(repository);
                mgr.start_all();
                mgr
            }
        };

        // 管道已经启动，不需要再次启动
        let manager = manager;

        // 获取共享缓冲区供 ViewModel 使用
        let shared_buffer = manager.get_shared_buffer();

        // 获取传感器原始数据缓冲区（如果 PipelineManager 支持）
        let shared_sensor_buffer = manager.get_shared_sensor_buffer();

        self.pipeline_manager = Some(manager);
        self.shared_buffer = Some(shared_buffer.clone());
        self.shared_sensor_buffer = shared_sensor_buffer;

        // Initialize the new SensorPipelineManager
        self.initialize_sensor_pipeline_manager(db_path);

        self.viewmodel_ready = true;

        tracing::info!("Three-pipeline data collection started");
        tracing::info!("Backend Thread 1 (Collection Pipeline) is now running at 10Hz");
        tracing::info!("Backend Thread 2 (Storage Pipeline) is now running at 1Hz");
        tracing::info!("Shared buffer created and ready for display pipeline");

        // 打印热加载状态
        if self.hot_reload_manager.is_some() {
            tracing::info!("配置热加载已启用 - 修改配置文件后立即生效");
        }
    }

    fn initialize_sensor_pipeline_manager(&mut self, db_path: &str) {
        tracing::info!("Initializing SensorPipelineManager...");

        let mut sensor_manager = SensorPipelineManager::new();

        // Register simulated data source
        let simulated_source = Arc::new(SimulatedDataSource::new());
        let config = PipelineConfig::default();
        sensor_manager.register_source(DataSourceId::Simulator, simulated_source, config);

        // Set up storage repository
        let storage_result = qt_threading_utils::runtime::global_runtime().block_on(async {
            qt_rust_demo::repositories::sqlite_storage_repository::SqliteStorageRepository::new(
                db_path,
            )
            .await
        });

        match storage_result {
            Ok(storage) => {
                let storage_arc: Arc<dyn sensor_core::StorageRepository> = Arc::new(storage);
                sensor_manager.set_storage_repository(storage_arc);

                // Start all pipelines
                if let Err(e) = sensor_manager.start_all() {
                    tracing::error!("Failed to start SensorPipelineManager: {}", e);
                } else {
                    tracing::info!(
                        "SensorPipelineManager started successfully with {} sensor(s)",
                        sensor_manager.sensor_count()
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create storage repository for SensorPipelineManager: {}",
                    e
                );
            }
        }

        self.sensor_manager = Some(sensor_manager);
    }

    /// 获取共享缓冲区（用于初始化 ViewModel 的显示管道）
    pub fn get_shared_buffer(&self) -> Option<SharedBuffer> {
        self.shared_buffer.clone()
    }

    /// 获取传感器原始数据共享缓冲区
    pub fn get_shared_sensor_buffer(&self) -> Option<SharedSensorBuffer> {
        self.shared_sensor_buffer.clone()
    }

    /// 停止数据采集
    pub fn stop_data_collection(&mut self) {
        // Stop the legacy pipeline manager
        if let Some(mut manager) = self.pipeline_manager.take() {
            manager.stop_all();
            tracing::info!("Legacy pipeline manager stopped");
        }

        // Stop the new sensor pipeline manager
        if let Some(mut sensor_manager) = self.sensor_manager.take() {
            sensor_manager.stop_all();
            tracing::info!("SensorPipelineManager stopped");
        }

        self.shared_buffer = None;
        self.viewmodel_ready = false;
        tracing::info!("Data collection stopped");
    }
}

impl Default for ViewModelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ViewModelManager {
    fn drop(&mut self) {
        self.stop_data_collection();
    }
}

/// 全局 ViewModel 管理器实例
static VIEWMODEL_MANAGER: Mutex<Option<ViewModelManager>> = Mutex::new(None);

/// 初始化全局管理器
pub fn init_viewmodel_manager() {
    let mut manager = VIEWMODEL_MANAGER.lock().unwrap();
    *manager = Some(ViewModelManager::new());
    tracing::info!("ViewModelManager initialized");
}

/// 标记 ViewModel 已准备好
#[allow(dead_code)]
pub fn mark_viewmodel_ready() {
    let mut manager = VIEWMODEL_MANAGER.lock().unwrap();
    if let Some(mgr) = manager.as_mut() {
        mgr.mark_viewmodel_ready();
    }
}

/// 启动全局数据采集
pub fn start_global_data_collection() {
    let mut manager = VIEWMODEL_MANAGER.lock().unwrap();
    if let Some(mgr) = manager.as_mut() {
        mgr.start_data_collection();
    }
}

/// 停止全局数据采集
pub fn stop_global_data_collection() {
    let mut manager = VIEWMODEL_MANAGER.lock().unwrap();
    if let Some(mgr) = manager.as_mut() {
        mgr.stop_data_collection();
    }
}

/// 获取全局共享缓冲区（供 ViewModel 初始化显示管道）
pub fn get_global_shared_buffer() -> Option<SharedBuffer> {
    let manager = VIEWMODEL_MANAGER.lock().unwrap();
    match manager.as_ref().and_then(|mgr| mgr.get_shared_buffer()) {
        Some(buffer) => {
            tracing::debug!("get_global_shared_buffer: returning buffer");
            Some(buffer)
        }
        None => {
            tracing::warn!("get_global_shared_buffer: NO BUFFER AVAILABLE");
            None
        }
    }
}

/// 获取全局传感器原始数据共享缓冲区
pub fn get_global_shared_sensor_buffer() -> Option<SharedSensorBuffer> {
    let manager = VIEWMODEL_MANAGER.lock().unwrap();
    match manager
        .as_ref()
        .and_then(|mgr| mgr.get_shared_sensor_buffer())
    {
        Some(buffer) => {
            tracing::debug!("get_global_shared_sensor_buffer: returning buffer");
            Some(buffer)
        }
        None => {
            tracing::warn!("get_global_shared_sensor_buffer: NO BUFFER AVAILABLE");
            None
        }
    }
}

/// 获取全局共享配置引用
pub fn get_global_shared_config_refs() -> Option<SharedConfigRefs> {
    let manager = VIEWMODEL_MANAGER.lock().unwrap();
    manager
        .as_ref()
        .and_then(|mgr| mgr.shared_config_refs.clone())
}

/// 手动重载所有配置
pub fn reload_all_configs() {
    let manager = VIEWMODEL_MANAGER.lock().unwrap();
    if let Some(mgr) = manager.as_ref() {
        if let Some(hot_reload_manager) = &mgr.hot_reload_manager {
            qt_threading_utils::runtime::global_runtime().block_on(async {
                if let Err(e) = hot_reload_manager.reload_all().await {
                    tracing::error!("手动重载配置失败: {}", e);
                } else {
                    tracing::info!("手动重载所有配置成功");
                }
            });
        } else {
            tracing::warn!("热加载服务未启用，无法重载配置");
        }
    }
}
