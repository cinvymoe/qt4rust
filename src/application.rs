// Application 应用程序入口 - 封装 Qt 应用初始化逻辑

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use std::fmt;
use crate::repositories::CraneDataRepository;
use crate::collector::DataCollector;

/// 应用程序错误类型
#[derive(Debug)]
pub enum ApplicationError {
    QmlLoadError(String),
    EngineInitError(String),
    ContextSetupError(String),
    ResourceError(String),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::QmlLoadError(msg) => write!(f, "QML load error: {}", msg),
            Self::EngineInitError(msg) => write!(f, "Engine init error: {}", msg),
            Self::ContextSetupError(msg) => write!(f, "Context setup error: {}", msg),
            Self::ResourceError(msg) => write!(f, "Resource error: {}", msg),
        }
    }
}

impl std::error::Error for ApplicationError {}

/// Application 结构体 - 管理应用程序生命周期
pub struct Application {
    qt_app: cxx::UniquePtr<QGuiApplication>,
    engine: cxx::UniquePtr<QQmlApplicationEngine>,
    data_collector: Option<DataCollector>,
}

impl Application {
    /// 创建新的应用程序实例
    /// 初始化 Qt 应用和 QML 引擎
    pub fn new() -> Result<Self, ApplicationError> {
        eprintln!("[INFO] Initializing Qt application...");
        
        // 创建 Qt 应用程序实例
        let qt_app = QGuiApplication::new();
        
        // 创建 QML 引擎
        let engine = QQmlApplicationEngine::new();
        
        Ok(Self { 
            qt_app, 
            engine,
            data_collector: None,
        })
    }

    /// 设置 QML 上下文
    /// 将 Rust 对象注册到 QML 上下文中
    fn setup_qml_context(&mut self) -> Result<(), ApplicationError> {
        eprintln!("[INFO] Setting up QML context...");
        
        // 注意: cxx-qt 0.8 使用不同的方式注册对象到 QML
        // Counter 对象通过 QML 模块系统自动注册
        // 这里暂时不需要手动 setContextProperty
        
        Ok(())
    }

    /// 加载 QML 文件
    fn load_qml(&mut self) -> Result<(), ApplicationError> {
        eprintln!("[INFO] Loading QML file...");
        
        // QML 文件路径 (使用 Qt 资源系统)
        let qml_path = "qrc:/qt/qml/qt/rust/demo/qml/main.qml";
        
        if let Some(engine) = self.engine.as_mut() {
            engine.load(&QUrl::from(qml_path));
            
            // Qt 会自动处理加载错误并输出到控制台
            // 如果加载失败，应用会继续运行但不显示窗口
        } else {
            return Err(ApplicationError::EngineInitError(
                "QML engine is not initialized".to_string()
            ));
        }
        
        eprintln!("[INFO] QML file loaded");
        Ok(())
    }

    /// 启动后台数据采集
    fn start_data_collection(&mut self) {
        eprintln!("[INFO] Starting background data collection...");
        
        // 创建数据仓库
        let repository = CraneDataRepository::new();
        
        // 创建数据采集器
        let mut collector = DataCollector::new(repository);
        
        // 启动采集（注意：这里需要通过 Qt 信号槽机制与 ViewModel 通信）
        // 暂时只打印日志，完整实现需要 Qt 的线程安全机制
        collector.start(|intent| {
            eprintln!("[DATA] Collected: {:?}", intent);
            // TODO: 通过 Qt 信号槽发送到 ViewModel
        });
        
        self.data_collector = Some(collector);
    }
    
    /// 运行应用程序
    /// 启动 Qt 事件循环并返回退出码
    pub fn run(&mut self) -> i32 {
        // 设置 QML 上下文
        if let Err(e) = self.setup_qml_context() {
            eprintln!("[ERROR] {}", e);
            return 5; // 上下文注册失败
        }
        
        // 加载 QML 文件
        if let Err(e) = self.load_qml() {
            eprintln!("[ERROR] {}", e);
            return match e {
                ApplicationError::QmlLoadError(_) => 3, // QML 文件加载失败
                ApplicationError::EngineInitError(_) => 2, // QML 引擎创建失败
                _ => 4, // QML 解析错误
            };
        }
        
        // 启动后台数据采集
        self.start_data_collection();
        
        eprintln!("[INFO] Starting Qt event loop...");
        
        // 启动事件循环
        if let Some(app) = self.qt_app.as_mut() {
            let exit_code = app.exec();
            eprintln!("[INFO] Application exiting with code: {}", exit_code);
            exit_code
        } else {
            eprintln!("[ERROR] Qt application is not initialized");
            1 // Qt 应用初始化失败
        }
    }
}
