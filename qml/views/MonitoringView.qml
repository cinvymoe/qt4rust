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
    
    // 模拟数据属性
    property real momentPercentage: 94.8
    property real currentLoad: 17.0
    property real ratedLoad: 25.0
    property real workRadius: 10.0
    property real boomAngle: 62.7
    
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
                
                // 危险状态卡片
                Rectangle {
                    width: parent.width
                    height: 96
                    color: "transparent"
                    border.color: Theme.dangerColor
                    border.width: Theme.borderThick
                    radius: Theme.radiusMedium
                    
                    // 半透明背景
                    Rectangle {
                        anchors.fill: parent
                        color: Theme.dangerBackground
                        opacity: 0.3
                        radius: Theme.radiusMedium
                    }
                    
                    Row {
                        anchors.centerIn: parent
                        spacing: Theme.spacingMedium
                        
                        Image {
                            source: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-danger.png"
                            width: Theme.iconSizeLarge
                            height: Theme.iconSizeLarge
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Column {
                            spacing: Theme.spacingTiny
                            anchors.verticalCenter: parent.verticalCenter
                            
                            Text {
                                text: "危险状态"
                                font.pixelSize: Theme.fontSizeXLarge
                                font.family: Theme.fontFamilyDefault
                                font.weight: Font.Medium
                                color: Theme.textPrimary
                            }
                            
                            Text {
                                text: "力矩超限！立即减载或降低幅度"
                                font.pixelSize: Theme.fontSizeMedium
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textSecondary
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
                    percentage: monitoringView.momentPercentage
                }
                
                // 数据网格 - 使用 GridView (每行2列，可滑动)
                GridView {
                    id: dataGridView
                    width: parent.width
                    height: parent.height - 216 - Theme.spacingMedium
                    cellWidth: parent.width / 2
                    cellHeight: 180 + Theme.spacingMedium
                    clip: true
                    
                    // 数据模型
                    model: ListModel {
                        id: dataGridModel
                        
                        ListElement {
                            type: "dataCard"
                            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-weight.png"
                            label: "当前载荷"
                            unit: "吨"
                            description: "额定载荷"
                            showProgress: true
                        }
                        
                        ListElement {
                            type: "dataCard"
                            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-radius.png"
                            label: "工作半径"
                            unit: "米"
                            description: "水平工作距离"
                            showProgress: false
                        }
                        
                        ListElement {
                            type: "dataCard"
                            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-angle.png"
                            label: "吊臂角度"
                            unit: "度"
                            description: "与水平面夹角"
                            showProgress: false
                        }
                        
                        ListElement {
                            type: "loadComparison"
                            label: "载荷对比"
                        }
                    }
                    
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
                                } else if (model.type === "loadComparison") {
                                    return loadComparisonComponent
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
                            value: {
                                if (itemData.label === "当前载荷") {
                                    return monitoringView.currentLoad.toFixed(1)
                                } else if (itemData.label === "工作半径") {
                                    return monitoringView.workRadius.toFixed(1)
                                } else if (itemData.label === "吊臂角度") {
                                    return monitoringView.boomAngle.toFixed(1)
                                }
                                return "0.0"
                            }
                            unit: itemData.unit
                            description: {
                                if (itemData.label === "当前载荷") {
                                    return "额定: " + monitoringView.ratedLoad.toFixed(1) + "吨"
                                }
                                return itemData.description
                            }
                            showProgress: itemData.showProgress
                            progress: itemData.showProgress ? (monitoringView.currentLoad / monitoringView.ratedLoad) : 0
                        }
                    }
                    
                    // 载荷对比组件
                    Component {
                        id: loadComparisonComponent
                        
                        Rectangle {
                            color: Theme.darkSurface
                            border.color: Theme.darkBorder
                            border.width: Theme.borderThin
                            radius: Theme.radiusMedium
                            
                            Column {
                                anchors.fill: parent
                                anchors.margins: Theme.spacingLarge
                                spacing: Theme.spacingMedium
                                
                                Text {
                                    text: "载荷对比"
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.family: Theme.fontFamilyDefault
                                    color: Theme.textSecondary
                                }
                                
                                Rectangle {
                                    width: parent.width
                                    height: 56
                                    radius: Theme.radiusMedium
                                    color: Theme.darkBorder
                                    
                                    Rectangle {
                                        width: parent.width * (monitoringView.currentLoad / monitoringView.ratedLoad)
                                        height: parent.height
                                        radius: Theme.radiusMedium
                                        color: Theme.successColor
                                        
                                        Behavior on width {
                                            NumberAnimation {
                                                duration: Theme.animationDuration
                                            }
                                        }
                                    }
                                    
                                    Column {
                                        anchors.centerIn: parent
                                        spacing: 0
                                        
                                        Text {
                                            text: "实际 " + monitoringView.currentLoad.toFixed(1) + "t"
                                            font.pixelSize: Theme.fontSizeTiny
                                            font.family: Theme.fontFamilyMono
                                            color: Theme.textPrimary
                                        }
                                        
                                        Text {
                                            text: "额定 " + monitoringView.ratedLoad.toFixed(1) + "t"
                                            font.pixelSize: Theme.fontSizeTiny
                                            font.family: Theme.fontFamilyMono
                                            color: Theme.textSecondary
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
            }
        }
    }
}
