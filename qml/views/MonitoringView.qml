// MonitoringView.qml - 监控主视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components/controls"
import "../components/layouts"
import "../styles"

Item {
    id: monitoringView
    
    // 顶部栏显示状态（从父组件传递）
    property bool headerVisible: true
    
    // 数据模型 - 外部定义
    ListModel {
        id: monitoringDataModel
        
        ListElement {
            type: "dataCard"
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-weight.png"
            label: "当前载荷"
            unit: "吨"
            description: "额定载荷"
            showProgress: true
            value: 17.0
            maxValue: 25.0
        }
        
        ListElement {
            type: "dataCard"
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-radius.png"
            label: "工作半径"
            unit: "米"
            description: "水平工作距离"
            showProgress: false
            value: 10.0
            maxValue: 0.0
        }
        
        ListElement {
            type: "dataCard"
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-angle.png"
            label: "吊臂角度"
            unit: "度"
            description: "与水平面夹角"
            showProgress: false
            value: 62.7
            maxValue: 0.0
        }
        
        ListElement {
            type: "boomLength"
            label: "臂长"
            value: 22.6
        }
    }
    
    Rectangle {
        anchors.fill: parent
        color: "transparent"
        
        Column {
            anchors.fill: parent
            spacing: 0
            
            // 顶部栏
            Header {
                id: header
                width: parent.width
                height: monitoringView.headerVisible ? Theme.headerHeight : 0
                visible: height > 0
                alertActive: true
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
                    
                    // 危险状态卡片
                    DangerCard {
                        id: dangerCard
                        anchors.fill: parent
                        opacity: carouselContainer.currentIndex === 0 ? 1 : 0
                        visible: opacity > 0
                        
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
                
                // 力矩百分比卡片
                MomentCard {
                    width: parent.width
                    height: 216
                    percentage: 94.8
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
                        
                        Loader {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMedium / 2
                            
                            property var itemData: model
                            
                            sourceComponent: {
                                if (model.type === "dataCard") {
                                    return dataCardComponent
                                } else if (model.type === "boomLength") {
                                    return boomLengthComponent
                                }
                                return null
                            }
                        }
                    }
                    
                    // DataCard 组件
                    Component {
                        id: dataCardComponent
                        
                        DataCard {
                            iconSource: itemData.iconSource
                            label: itemData.label
                            value: itemData.value.toFixed(1)
                            unit: itemData.unit
                            description: {
                                if (itemData.showProgress) {
                                    return "额定: " + itemData.maxValue.toFixed(1) + itemData.unit
                                }
                                return itemData.description
                            }
                            showProgress: itemData.showProgress
                            progress: itemData.showProgress ? (itemData.value / itemData.maxValue) : 0
                        }
                    }
                    
                    // 臂长组件
                    Component {
                        id: boomLengthComponent
                        
                        BoomLengthCard {
                            boomLength: itemData.value
                        }
                    }
                }
            }
        }
            }
        }
    }
}
