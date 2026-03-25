// 数据采集控制器 - 暴露给 QML 的控制接口

#[cxx_qt::bridge]
pub mod data_collection_controller_bridge {
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type DataCollectionController = super::DataCollectionControllerRust;

        /// 启动数据采集
        #[qinvokable]
        unsafe fn start_collection(self: Pin<&mut DataCollectionController>);

        /// 停止数据采集
        #[qinvokable]
        unsafe fn stop_collection(self: Pin<&mut DataCollectionController>);

        /// 检查是否正在采集
        #[qinvokable]
        unsafe fn is_collecting(self: &DataCollectionController) -> bool;
    }
}

use core::pin::Pin;

/// 数据采集控制器实现
pub struct DataCollectionControllerRust {
    // 不需要存储状态，因为状态在全局管理器中
}

impl Default for DataCollectionControllerRust {
    fn default() -> Self {
        Self {}
    }
}

impl data_collection_controller_bridge::DataCollectionController {
    /// 启动数据采集
    pub fn start_collection(self: Pin<&mut Self>) {
        tracing::info!("Starting data collection from QML...");

        // 调用全局管理器启动数据采集
        crate::viewmodel_manager::start_global_data_collection();

        tracing::info!("Data collection started");
    }

    /// 停止数据采集
    pub fn stop_collection(self: Pin<&mut Self>) {
        tracing::info!("Stopping data collection from QML...");

        // 调用全局管理器停止数据采集
        crate::viewmodel_manager::stop_global_data_collection();

        tracing::info!("Data collection stopped");
    }

    /// 检查是否正在采集
    pub fn is_collecting(&self) -> bool {
        // 简化实现：总是返回 false
        // TODO: 从全局管理器查询状态
        false
    }
}
