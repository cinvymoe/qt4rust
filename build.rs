use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // cxx-qt 0.8 构建配置 - 使用 QML 模块
    CxxQtBuilder::new_qml_module(
        QmlModule::new("qt.rust.demo")
            // 主文件
            .qml_file("qml/main.qml")
            // 视图
            .qml_file("qml/views/HomeView.qml")
            .qml_file("qml/views/SettingsView.qml")
            .qml_file("qml/views/MonitoringView.qml")
            .qml_file("qml/views/ChartView.qml")
            .qml_file("qml/views/AlarmRecordView.qml")
            // 控件组件
            .qml_file("qml/components/controls/CustomButton.qml")
            .qml_file("qml/components/controls/CustomInput.qml")
            .qml_file("qml/components/controls/DataCard.qml")
            .qml_file("qml/components/controls/MomentCard.qml")
            .qml_file("qml/components/controls/NavigationButton.qml")
            .qml_file("qml/components/controls/StatusCard.qml")
            .qml_file("qml/components/controls/ProgressBar.qml")
            .qml_file("qml/components/controls/AlarmRecordItem.qml")
            // 布局组件
            .qml_file("qml/components/layouts/MainLayout.qml")
            .qml_file("qml/components/layouts/Header.qml")
            .qml_file("qml/components/layouts/Navigation.qml")
            // 对话框
            .qml_file("qml/components/dialogs/InfoDialog.qml")
            // 样式系统
            .qml_file("qml/styles/Theme.qml")
    )
    // 添加图片资源到 Qt 资源系统
    .qrc_resources(vec![
        "qml/styles/qmldir",  // Theme singleton 配置
        "qml/assets/images/canvas-crane.png",
        "qml/assets/images/canvas.png",
        "qml/assets/images/icon-alarm-record.png",
        "qml/assets/images/icon-alarm-record.svg",
        "qml/assets/images/icon-alert.png",
        "qml/assets/images/icon-angle.png",
        "qml/assets/images/icon-chart.png",
        "qml/assets/images/icon-chart.svg",
        "qml/assets/images/icon-danger.png",
        "qml/assets/images/icon-gauge.png",
        "qml/assets/images/icon-home.png",
        "qml/assets/images/icon-home.svg",
        "qml/assets/images/icon-logo.png",
        "qml/assets/images/icon-moment.png",
        "qml/assets/images/icon-radius.png",
        "qml/assets/images/icon-settings.png",
        "qml/assets/images/icon-settings.svg",
        "qml/assets/images/icon-weight.png",
    ])
    .qt_module("Network")  // macOS 需要
    .qt_module("Charts")   // 图表模块
    .files(["src/counter.rs"])
    .build();
}
