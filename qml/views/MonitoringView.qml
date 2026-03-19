// MonitoringView.qml - 监控主视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo  // 导入 Rust ViewModel
import "../components/controls"
import "../components/layouts"
import "../components/dialogs"  // 导入对话框组件
import "../styles"

Item {
    id: monitoringView
    
    // 顶部栏显示状态（从父组件传递）
    property bool headerVisible: true
    
    // 绑定 ViewModel
    MonitoringViewModel {
        id: viewModel
        
        // 组件创建完成后注册到管理器
        Component.onCompleted: {
            console.log("[QML] MonitoringViewModel created")
            
            // 调试：列出所有可用的方法
            console.log("[QML] Available methods:")
            for (var prop in viewModel) {
                if (typeof viewModel[prop] === "function") {
                    console.log("[QML]   -", prop)
                }
            }
        }
    }
    
    // 使用 Timer 模拟数据更新（测试用）
    Timer {
        id: dataUpdateTimer
        interval: 1000  // 1秒更新一次（方便观察）
        running: true
        repeat: true
        
        property real loadValue: 15.0
        property real radiusValue: 8.0
        property real angleValue: 60.0
        
        onTriggered: {
            // 模拟数据变化
            loadValue = 10.0 + Math.random() * 15.0  // 10-25 吨
            radiusValue = 5.0 + Math.random() * 10.0  // 5-15 米
            angleValue = 30.0 + Math.random() * 50.0  // 30-80 度
            
            console.log("[QML] Simulating data update: load=" + loadValue.toFixed(1))
            
            // 调用 ViewModel 方法（注意：使用 snake_case）
            viewModel.update_test_data(loadValue, radiusValue, angleValue)
        }
    }
    
    // 数据模型 - 使用 ViewModel 数据动态更新
    ListModel {
        id: monitoringDataModel
    }
    
    // 初始化数据模型
    Component.onCompleted: {
        updateDataModel()
    }
    
    // 监听 ViewModel 属性变化，更新数据模型
    Connections {
        target: viewModel
        function onCurrent_loadChanged() { updateDataModel() }
        function onWorking_radiusChanged() { updateDataModel() }
        function onBoom_angleChanged() { updateDataModel() }
        function onBoom_lengthChanged() { updateDataModel() }
    }
    
    // 更新数据模型函数
    function updateDataModel() {
        monitoringDataModel.clear()
        
        // 确保值不为 undefined（注意：属性名使用 snake_case）
        var currentLoad = viewModel.current_load || 0
        var ratedLoad = viewModel.rated_load || 25
        var workingRadius = viewModel.working_radius || 0
        var boomAngle = viewModel.boom_angle || 0
        var boomLength = viewModel.boom_length || 22.6
        
        monitoringDataModel.append({
            type: "dataCard",
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-weight.png",
            label: "当前载荷",
            unit: "吨",
            description: "额定载荷",
            showProgress: true,
            value: currentLoad,
            maxValue: ratedLoad
        })
        
        monitoringDataModel.append({
            type: "dataCard",
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-radius.png",
            label: "工作半径",
            unit: "米",
            description: "水平工作距离",
            showProgress: false,
            value: workingRadius,
            maxValue: 0.0
        })
        
        monitoringDataModel.append({
            type: "dataCard",
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-angle.png",
            label: "吊臂角度",
            unit: "度",
            description: "与水平面夹角",
            showProgress: false,
            value: boomAngle,
            maxValue: 0.0
        })
        
        monitoringDataModel.append({
            type: "boomLength",
            label: "臂长",
            value: boomLength
        })
    }
    
    Rectangle {
        anchors.fill: parent
        color: "transparent"
        
        // 错误提示对话框
        InfoDialog {
            id: errorDialog
            visible: viewModel.error_message !== ""
            title: "数据异常"
            message: viewModel.error_message
            
            onAccepted: {
                viewModel.clear_error()
            }
        }
        
        // 传感器断连提示
        Rectangle {
            id: sensorDisconnectedBanner
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 40
            color: Theme.dangerBackground
            border.color: Theme.dangerColor
            border.width: Theme.borderNormal
            visible: !viewModel.sensor_connected
            z: 100
            
            Row {
                anchors.centerIn: parent
                spacing: Theme.spacingMedium
                
                Rectangle {
                    width: Theme.iconSizeSmall
                    height: Theme.iconSizeSmall
                    color: Theme.dangerColor
                    radius: width / 2
                    anchors.verticalCenter: parent.verticalCenter
                }
                
                Text {
                    text: "传感器连接断开"
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textPrimary
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }
        
        Column {
            anchors.fill: parent
            spacing: 0
            
            // 顶部栏 - 绑定危险状态
            Header {
                id: header
                width: parent.width
                height: monitoringView.headerVisible ? Theme.headerHeight : 0
                visible: height > 0
                alertActive: viewModel.is_danger
                clip: true
                
                Behavior on height {
                    NumberAnimation {
                        duration: Theme.animationDuration
                        easing.type: Easing.InOutQuad
                    }
                }
            }
            
            // 主内容区域
            Rectangle {
                width: parent.width
                height: parent.height - header.height
                color: "transparent"
                
                Row {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMedium
                    spacing: Theme.spacingMedium
            
            // 左列
            Column {
                width: (parent.width - Theme.spacingMedium) / 2
                height: parent.height
                spacing: Theme.spacingMedium
                
                // 轮播卡片容器
                Item {
                    id: carouselContainer
                    width: parent.width
                    height: 96
                    
                    // 当前显示的卡片索引 (0: 危险状态, 1: 时间卡片)
                    property int currentIndex: 0
                    
                    // 轮播定时器 (每5秒切换)
                    Timer {
                        id: carouselTimer
                        interval: 5000
                        running: true
                        repeat: true
                        onTriggered: {
                            carouselContainer.currentIndex = (carouselContainer.currentIndex + 1) % 2
                        }
                    }
                    
                    // 危险状态卡片 - 绑定 ViewModel
                    DangerCard {
                        id: dangerCard
                        anchors.fill: parent
                        opacity: carouselContainer.currentIndex === 0 ? 1 : 0
                        visible: opacity > 0
                        // 根据力矩百分比显示不同的消息
                        title: viewModel.moment_percentage >= 100 ? "严重危险" : "危险状态"
                        message: viewModel.moment_percentage >= 100 ? 
                            "力矩严重超限！立即停止作业" : 
                            "力矩超限！立即减载或降低幅度"
                        
                        Behavior on opacity {
                            NumberAnimation {
                                duration: Theme.animationDuration
                                easing.type: Easing.InOutQuad
                            }
                        }
                    }
                    
                    // 时间卡片
                    TimeCard {
                        id: timeCard
                        anchors.fill: parent
                        opacity: carouselContainer.currentIndex === 1 ? 1 : 0
                        visible: opacity > 0
                        
                        Behavior on opacity {
                            NumberAnimation {
                                duration: Theme.animationDuration
                                easing.type: Easing.InOutQuad
                            }
                        }
                    }
                }
                
                // 起重机臂架状态卡片
                Rectangle {
                    width: parent.width
                    height: parent.height - 96 - Theme.spacingMedium
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMedium
                        spacing: Theme.spacingMedium
                        
                        // 标题
                        Row {
                            spacing: Theme.spacingMedium
                            
                            Rectangle {
                                width: 4
                                height: Theme.fontSizeLarge
                                color: Theme.darkAccent
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "起重机臂架状态"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyDefault
                                font.weight: Font.Medium
                                color: Theme.textPrimary
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 臂架图
                        Rectangle {
                            width: parent.width
                            height: parent.height - Theme.fontSizeLarge - Theme.spacingMedium * 2
                            color: Theme.darkBackground
                            radius: Theme.radiusMedium
                            
                            Image {
                                anchors.centerIn: parent
                                source: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/canvas-crane.png"
                                fillMode: Image.PreserveAspectFit
                                width: parent.width - Theme.spacingMedium * 2
                                height: parent.height - Theme.spacingMedium * 2
                            }
                        }
                    }
                }
            }
            
            // 右列
            Column {
                width: (parent.width - Theme.spacingMedium) / 2
                height: parent.height
                spacing: Theme.spacingMedium
                
                // 力矩百分比卡片 - 绑定 ViewModel
                MomentCard {
                    width: parent.width
                    height: 216
                    percentage: viewModel.moment_percentage
                }
                
                // 数据网格 - 使用 GridView (每行2列，可滑动)
                GridView {
                    id: dataGridView
                    width: parent.width
                    height: parent.height - 216 - Theme.spacingMedium
                    cellWidth: parent.width / 2
                    cellHeight: 150 + Theme.spacingMedium
                    clip: true
                    
                    // 使用外部定义的数据模型
                    model: monitoringDataModel
                    
                    // 网格项目
                    delegate: Item {
                        width: dataGridView.cellWidth
                        height: dataGridView.cellHeight
                        
                        // 直接在 delegate 中渲染，不使用 Loader
                        DataCard {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMedium / 2
                            visible: model.type === "dataCard"
                            
                            iconSource: model.iconSource || ""
                            label: model.label || ""
                            value: (model.value || 0).toFixed(1)
                            unit: model.unit || ""
                            description: {
                                if (model.showProgress) {
                                    return "额定: " + (model.maxValue || 0).toFixed(1) + (model.unit || "")
                                }
                                return model.description || ""
                            }
                            showProgress: model.showProgress || false
                            progress: model.showProgress ? ((model.value || 0) / (model.maxValue || 1)) : 0
                        }
                        
                        BoomLengthCard {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMedium / 2
                            visible: model.type === "boomLength"
                            
                            boomLength: model.value || 0
                        }
                    }
                }
            }
        }
            }
        }
    }
}
