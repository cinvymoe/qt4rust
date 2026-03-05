use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // cxx-qt 0.8 构建配置 - 使用 QML 模块
    CxxQtBuilder::new_qml_module(
        QmlModule::new("qt.rust.demo")
            .qml_file("qml/main.qml")
    )
    .qt_module("Network")  // macOS 需要
    .files(["src/counter.rs"])
    .build();
}
