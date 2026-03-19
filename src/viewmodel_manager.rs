// ViewModel 管理器 - 管理全局 ViewModel 实例和数据采集

use crate::repositories::CraneDataRepository;
use crate::collector::DataCollector;
use std::sync::Mutex;

/// ViewModel 管理器
pub struct ViewModelManager {
    /// 数据采集器
    data_collector: Option<DataCollector>,
    
    /// ViewModel 是否已准备好
    viewmodel_ready: bool,
}

impl ViewModelManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            data_collector: None,
            viewmodel_ready: false,
        }
    }
    
    /// 标记 ViewModel 已准备好
    pub fn mark_viewmodel_ready(&mut self) {
        eprintln!("[INFO] ViewModel marked as ready");
        self.viewmodel_ready = true;
    }
    
    /// 启动数据采集
    pub fn start_data_collection(&mut self) {
        if !self.viewmodel_ready {
            eprintln!("[WARN] ViewModel not ready, cannot start data collection");
            return;
        }
        
        eprintln!("[INFO] Starting data collection...");
        
        // 创建数据仓库
        let repository = CraneDataRepository::new();
        
        // 创建数据采集器
        let mut collector = DataCollector::new(repository);
        
        // 启动采集 - 暂时只打印日志
        // TODO: 实现与 ViewModel 的通信机制
        collector.start(move |intent| {
            eprintln!("[DATA] Collected: {:?}", intent);
            // TODO: 通过某种机制更新 ViewModel
        });
        
        self.data_collector = Some(collector);
        eprintln!("[INFO] Data collection started");
    }
    
    /// 停止数据采集
    pub fn stop_data_collection(&mut self) {
        if let Some(mut collector) = self.data_collector.take() {
            collector.stop();
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
