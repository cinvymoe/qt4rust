fn main() {
    use cxx_qt_build::{CxxQtBuilder, QmlModule};

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
                // 设置子页面
                .qml_file("qml/views/SettingsView/SystemStatusView.qml")
                .qml_file("qml/views/SettingsView/CalibrationView.qml")
                .qml_file("qml/views/SettingsView/MomentCurveView.qml")
                .qml_file("qml/views/SettingsView/AboutSystemView.qml")
                // 校准内容组件
                .qml_file("qml/views/SettingsView/CalibrationContents/LoadCalibrationContent.qml")
                .qml_file("qml/views/SettingsView/CalibrationContents/AngleCalibrationContent.qml")
                .qml_file("qml/views/SettingsView/CalibrationContents/RadiusCalibrationContent.qml")
                .qml_file("qml/views/SettingsView/CalibrationContents/AlarmThresholdContent.qml")
                // 控件组件
                .qml_file("qml/components/controls/CustomButton.qml")
                .qml_file("qml/components/controls/CustomInput.qml")
                .qml_file("qml/components/controls/DataCard.qml")
                .qml_file("qml/components/controls/MomentCard.qml")
                .qml_file("qml/components/controls/NavigationButton.qml")
                .qml_file("qml/components/controls/StatusCard.qml")
                .qml_file("qml/components/controls/ProgressBar.qml")
                .qml_file("qml/components/controls/AlarmRecordItem.qml")
                .qml_file("qml/components/controls/SensorStatusCard.qml")
                .qml_file("qml/components/controls/LoadCurveChart.qml")
                .qml_file("qml/components/controls/HistoryFilterBar.qml")
                .qml_file("qml/components/controls/TimeRangeFilter.qml")
                .qml_file("qml/components/controls/MomentTrendChart.qml")
                .qml_file("qml/components/controls/LoadTrendChart.qml")
                .qml_file("qml/components/controls/MultiParamChart.qml")
                .qml_file("qml/components/controls/TimeCard.qml")
                .qml_file("qml/components/controls/DangerCard.qml")
                .qml_file("qml/components/controls/BoomLengthCard.qml")
                // 布局组件
                .qml_file("qml/components/layouts/MainLayout.qml")
                .qml_file("qml/components/layouts/Header.qml")
                .qml_file("qml/components/layouts/Navigation.qml")
                // 对话框
                .qml_file("qml/components/dialogs/InfoDialog.qml")
                .qml_file("qml/components/dialogs/CustomTimeRangeDialog.qml")
                // 样式系统
                .qml_file("qml/styles/Theme.qml"),
        )
        // 添加图片资源到 Qt 资源系统
        .qrc_resources(vec![
            "qml/styles/qmldir",
            "qml/views/SettingsView/qmldir",
            "qml/views/SettingsView/CalibrationContents/qmldir",
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
            "qml/assets/images/icon-system-status.svg",
            "qml/assets/images/icon-calibration.svg",
            "qml/assets/images/icon-moment-curve.svg",
            "qml/assets/images/icon-about-system.svg",
            "qml/assets/images/icon-sensor.svg",
            "qml/assets/images/icon-status-online.svg",
            "qml/assets/images/icon-phone.svg",
            "qml/assets/images/icon-email.svg",
            "qml/assets/images/icon-location.svg",
            "qml/assets/images/icon-boom-length.svg",
        ])
        .qt_module("Network")
        .file("src/monitoring_viewmodel.rs")
        .file("src/data_collection_controller.rs")
        .build();
}
