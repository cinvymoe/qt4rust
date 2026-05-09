// SystemStatusView.qml - 系统状态子页面
import qt.rust.demo
import QtQuick
import QtQuick.Controls
import "../../styles"
import "../../components/controls"

Flickable {
    id: systemStatusView
    width: parent.width
    height: parent.height
    contentHeight: systemStatusColumn.height
    clip: true

    
    Column {
        id: systemStatusColumn
        width: parent.width
        
        Item {
            width: parent.width
            height: childrenRect.height
            
            Column {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width - 200  // 左右各留 100px 边距
                spacing: Theme.spacingMedium
                topPadding: Theme.spacingMedium
                bottomPadding: Theme.spacingMedium
            
                // 1. 传感器连接状态
                Rectangle {
                    width: parent.width
                    height: childrenRect.height + Theme.borderThin
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingMedium
                        topPadding: 24.667
                        bottomPadding: Theme.borderThin
                        leftPadding: 24.667
                        rightPadding: 24.667
                        
                        // 标题区域
                        Item {
                            width: parent.width - 2 * 24.667
                            height: 28
                            
                            Rectangle {
                                width: 4
                                height: 24
                                color: Theme.darkAccent
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("systemStatus.sensorConnection") }
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                anchors.left: parent.left
                                anchors.leftMargin: 12
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 传感器网格 (2列3行)
                        Grid {
                            width: parent.width - 2 * 24.667
                            columns: 2
                            rowSpacing: Theme.spacingMedium
                            columnSpacing: Theme.spacingMedium
                            
                            Repeater {
                                model: [
                                    {name: "力矩传感器", status: "在线 - 正常", online: true},
                                    {name: "角度传感器", status: "在线 - 正常", online: true},
                                    {name: "长度传感器", status: "在线 - 正常", online: true},
                                    {name: "压力传感器", status: "在线 - 正常", online: true},
                                    {name: "倾角传感器", status: "在线 - 正常", online: true},
                                    {name: "风速传感器", status: "在线 - 正常", online: true}
                                ]
                                
                                SensorStatusCard {
                                    width: (parent.width - Theme.spacingMedium) / 2
                                    sensorName: modelData.name
                                    statusText: modelData.status
                                    isOnline: modelData.online
                                }
                            }
                        }
                    }
                }
                
                // 2. 系统信息
                Rectangle {
                    width: parent.width
                    height: childrenRect.height
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingMedium
                        topPadding: Theme.spacingLarge
                        bottomPadding: Theme.spacingLarge
                        leftPadding: Theme.spacingLarge
                        rightPadding: Theme.spacingLarge
                        
                        // 标题
                        Row {
                            width: parent.width - 2 * Theme.spacingLarge
                            spacing: Theme.spacingMedium
                            
                            Rectangle {
                                width: 4
                                height: 24
                                color: Theme.darkAccent
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("systemStatus.systemInfo") }
                                font.pixelSize: Theme.fontSizeLarge
                                color: Theme.textPrimary
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 系统信息网格
                        Grid {
                            width: parent.width - 2 * Theme.spacingLarge
                            columns: 2
                            rowSpacing: Theme.spacingMedium
                            columnSpacing: Theme.spacingMedium
                            
                            Repeater {
                                model: [
                                    {label: "系统版本", value: "v2.5.3", valueColor: Theme.textSecondary},
                                    {label: "设备型号", value: "QY50K-I", valueColor: Theme.textSecondary},
                                    {label: "内存使用", value: "512MB / 2GB", valueColor: "#05df72"},
                                    {label: "存储空间", value: "8.5GB / 32GB", valueColor: "#05df72"},
                                    {label: "数据更新频率", value: "1.5秒/次", valueColor: Theme.textSecondary},
                                    {label: "通信协议", value: "CAN 2.0", valueColor: Theme.textSecondary},
                                    {label: "操作系统", value: "Linux 5.15", valueColor: Theme.textSecondary},
                                    {label: "最后校准时间", value: "2025-11-10", valueColor: Theme.textSecondary}
                                ]
                                
                                Rectangle {
                                    width: (parent.width - Theme.spacingMedium) / 2
                                    height: 84
                                    color: Theme.darkBackground
                                    radius: Theme.radiusMedium
                                    
                                    Column {
                                        anchors.fill: parent
                                        anchors.margins: Theme.spacingMedium
                                        spacing: Theme.spacingTiny
                                        
                                        Text {
                                            text: modelData.label
                                            font.pixelSize: Theme.fontSizeSmall
                                            color: Theme.textTertiary
                                        }
                                        
                                        Text {
                                            text: modelData.value
                                            font.pixelSize: Theme.fontSizeNormal
                                            color: modelData.valueColor
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 3. 网络与通信
                Rectangle {
                    width: parent.width
                    height: childrenRect.height
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingMedium
                        topPadding: Theme.spacingLarge
                        bottomPadding: Theme.spacingLarge
                        leftPadding: Theme.spacingLarge
                        rightPadding: Theme.spacingLarge
                        
                        // 标题
                        Row {
                            width: parent.width - 2 * Theme.spacingLarge
                            spacing: Theme.spacingMedium
                            
                            Rectangle {
                                width: 4
                                height: 24
                                color: Theme.darkAccent
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("systemStatus.network") }
                                font.pixelSize: Theme.fontSizeLarge
                                color: Theme.textPrimary
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 网络状态行
                        Row {
                            width: parent.width - 2 * Theme.spacingLarge
                            spacing: Theme.spacingMedium
                            
                            Repeater {
                                model: [
                                    {title: "WiFi连接", status: "已连接", detail: "信号强度: 良好", icon: "📶"},
                                    {title: "CAN总线", status: "正常", detail: "波特率: 250Kbps", icon: "🔌"},
                                    {title: "数据采集", status: "运行中", detail: "延迟: <50ms", icon: "📊"}
                                ]
                                
                                Rectangle {
                                    width: (parent.width - 2 * Theme.spacingMedium) / 3
                                    height: 113.333
                                    color: Theme.darkBackground
                                    border.color: "#008236"
                                    border.width: Theme.borderThin
                                    radius: Theme.radiusMedium
                                    
                                    Column {
                                        anchors.fill: parent
                                        anchors.margins: Theme.spacingMedium
                                        spacing: Theme.spacingSmall
                                        
                                        Row {
                                            spacing: Theme.spacingSmall
                                            
                                            Text {
                                                text: modelData.icon
                                                font.pixelSize: Theme.fontSizeLarge
                                                anchors.verticalCenter: parent.verticalCenter
                                            }
                                            
                                            Text {
                                                text: modelData.title
                                                font.pixelSize: Theme.fontSizeMedium
                                                color: Theme.textSecondary
                                                anchors.verticalCenter: parent.verticalCenter
                                            }
                                        }
                                        
                                        Text {
                                            text: modelData.status
                                            font.pixelSize: Theme.fontSizeMedium
                                            color: Theme.successColor
                                        }
                                        
                                        Text {
                                            text: modelData.detail
                                            font.pixelSize: Theme.fontSizeSmall
                                            color: Theme.textTertiary
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
