// SettingsView.qml - 设置页面视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../styles"
import "SettingsView"

Item {
    id: settingsView
    
    property int currentTabIndex: 0
    
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        
        Column {
            anchors.fill: parent
            spacing: 0
            
            // 合并的标题和 Tab 栏区域
            Rectangle {
                width: parent.width
                height: Math.max(70, 92.667)
                color: Theme.darkSurface
                
                Rectangle {
                    width: parent.width
                    height: Theme.borderThin
                    color: Theme.darkBorder
                    anchors.bottom: parent.bottom
                }
                
                Row {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingLarge
                    anchors.rightMargin: Theme.spacingLarge
                    spacing: Theme.spacingMedium
                    
                    // 左侧：标题和描述
                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: Theme.spacingTiny
                        width: Math.min(300, parent.width * 0.3)
                        
                        Text {
                            text: getCurrentTabTitle()
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            font.family: Theme.fontFamilyDefault
                        }
                        
                        Text {
                            text: getCurrentTabDescription()
                            font.pixelSize: Theme.fontSizeTiny
                            color: Theme.textTertiary
                            font.family: Theme.fontFamilyDefault
                            elide: Text.ElideRight
                            width: parent.width
                        }
                    }
                    
                    // 右侧：Tab 栏
                    Row {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - Math.min(300, parent.width * 0.3) - Theme.spacingMedium
                        spacing: 0
                        
                        Repeater {
                            model: [
                                {text: "系统状态", icon: "icon-system-status.svg"},
                                {text: "参数校准", icon: "icon-calibration.svg"},
                                {text: "力矩曲线", icon: "icon-moment-curve.svg"},
                                {text: "关于系统", icon: "icon-about-system.svg"}
                            ]
                            
                            Rectangle {
                                width: parent.width / 4
                                height: Math.max(70, 92.667)
                                color: currentTabIndex === index ? Theme.darkBackground : "transparent"
                                
                                Row {
                                    anchors.centerIn: parent
                                    spacing: Theme.spacingSmall
                                    
                                    Image {
                                        source: "../assets/images/" + modelData.icon
                                        width: Theme.iconSizeSmall
                                        height: Theme.iconSizeSmall
                                        sourceSize.width: Theme.iconSizeSmall
                                        sourceSize.height: Theme.iconSizeSmall
                                        fillMode: Image.PreserveAspectFit
                                        anchors.verticalCenter: parent.verticalCenter
                                    }
                                    
                                    Text {
                                        text: modelData.text
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: currentTabIndex === index ? Theme.textAccent : Theme.textTertiary
                                        font.family: Theme.fontFamilyDefault
                                        anchors.verticalCenter: parent.verticalCenter
                                        elide: Text.ElideRight
                                        
                                        Behavior on color {
                                            ColorAnimation {
                                                duration: Theme.animationDuration
                                            }
                                        }
                                    }
                                }
                                
                                // 底部指示条
                                Rectangle {
                                    visible: currentTabIndex === index
                                    width: parent.width
                                    height: 2
                                    color: Theme.darkAccent
                                    anchors.bottom: parent.bottom
                                }
                                
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: currentTabIndex = index
                                }
                            }
                        }
                    }
                }
            }
            
            // 内容切换区域
            StackLayout {
                width: parent.width
                height: parent.height - Math.max(70, 92.667)
                currentIndex: currentTabIndex
                
                // Tab 0: 系统状态
                SystemStatusView {}
                
                // Tab 1: 参数校准
                CalibrationView {}
                
                // Tab 2: 力矩曲线
                MomentCurveView {}
                
                // Tab 3: 关于系统
                AboutSystemView {}
            }
        }
    }
    
    // 获取当前 Tab 的标题
    function getCurrentTabTitle() {
        switch(currentTabIndex) {
            case 0: return "系统状态"
            case 1: return "参数校准"
            case 2: return "力矩曲线"
            case 3: return "关于系统"
            default: return ""
        }
    }
    
    // 获取当前 Tab 的描述文本
    function getCurrentTabDescription() {
        switch(currentTabIndex) {
            case 0: return "设备运行状态与传感器监控"
            case 1: return "传感器参数校准与配置"
            case 2: return "力矩曲线设置与管理"
            case 3: return "系统版本与设备信息"
            default: return ""
        }
    }
}
