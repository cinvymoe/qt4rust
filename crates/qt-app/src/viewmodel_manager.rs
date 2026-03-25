// ViewModel 管理器 - 管理全局 ViewModel 实例和数据采集

use qt_rust_demo::repositories::CraneDataRepository;
use qt_rust_demo::pipeline::PipelineManager;
use std::sync::{Arc, Mutex};

/// ViewModel 管理器
pub struct ViewModelManager {
    /// 管道管理器（三后台管道架构）
    pipeline_manager: Option<PipelineManager>,
    
    /// ViewModel 是否已准备好
    viewmodel_ready: bool,
}

impl ViewModelManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            pipeline_manager: None,
            viewmodel_ready: false,
        }
    }
    
    /// 标记 ViewModel 已准备好
    #[allow(dead_code)]
    pub fn mark_viewmodel_ready(&mut self) {
        eprintln!("[INFO] ViewModel marked as ready");
        self.viewmodel_ready = true;
    }
    
    /// 启动数据采集
    pub fn start_data_collection(&mut self) {
        eprintln!("[INFO] Starting three-pipeline data collection...");
        
        // 创建数据仓库（使用 Default 实现，自动创建 ConfigManager）
        let repository = Arc::new(CraneDataRepository::default());
        
        // 创建管道管理器（带存储支持）
        let db_path = "crane_data.db";
        eprintln!("[INFO] Initializing storage system with database: {}", db_path);
        
        let manager = match qt_threading_utils::runtime::global_runtime().block_on(async {
            PipelineManager::new_with_storage(repository, db_path).await
        }) {
            Ok(mgr) => {
                eprintln!("[INFO] Storage system initialized successfully");
                mgr
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to initialize storage system: {}", e);
                eprintln!("[WARN] Falling back to collection-only mode");
                let repository = Arc::new(CraneDataRepository::default());
                PipelineManager::new(repository)
            }
        };
        
        // 启动所有管道（管道1 + 管道2）
        let mut manager = manager;
        manager.start_all();
        
        self.pipeline_manager = Some(manager);
        self.viewmodel_ready = true;  // 标记为已准备好
        
        eprintln!("[INFO] Three-pipeline data collection started");
        eprintln!("[INFO] Backend Thread 1 (Collection Pipeline) is now running at 10Hz");
        eprintln!("[INFO] Backend Thread 2 (Storage Pipeline) is now running at 1Hz");
    }
    
    /// 停止数据采集
    pub fn stop_data_collection(&mut self) {
        if let Some(mut manager) = self.pipeline_manager.take() {
            manager.stop_all();
            eprintln!("[INFO] Data collection stopped");
        }
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
    eprintln!("[INFO] ViewModelManager initialized");
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
