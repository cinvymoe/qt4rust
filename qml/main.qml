// Qt Rust Demo - 主界面 QML 文件
// 起重机力矩监测系统

import QtQuick
import QtQuick.Controls
import QtQuick.Window
import QtQuick.Layouts
import QtQuick.VirtualKeyboard
import "views"
import "components/layouts"
import "styles"

Window {
    id: root
    width: 1024
    height: 800
    visible: true
    title: "Crane Monitoring System"
    
    // 导航栏显示状态
    property bool navigationVisible: true
    
    // 自动隐藏定时器
    Timer {
        id: hideNavigationTimer
        interval: 3000  // 3秒
        running: false
        repeat: false
        onTriggered: {
            navigationVisible = false
        }
    }
    
    // 重置定时器的函数
    function resetHideTimer() {
        navigationVisible = true
        hideNavigationTimer.restart()
    }
    
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        
        // 全局鼠标区域，用于检测点击
        MouseArea {
            anchors.fill: parent
            onClicked: {
                resetHideTimer()
            }
            propagateComposedEvents: true
            
            Column {
                anchors.fill: parent
                spacing: 0
                
                // 顶部栏
                Header {
                    id: header
                    width: parent.width
                    alertActive: true
                }
                
                // 主内容区域
                Item {
                    id: contentArea
                    width: parent.width
                    height: parent.height - header.height - (navigationVisible ? navigation.height : 0)
                    
                    Behavior on height {
                        NumberAnimation {
                            duration: Theme.animationDuration
                            easing.type: Easing.InOutQuad
                        }
                    }
                    
                    // 页面切换器
                    StackLayout {
                        id: stackLayout
                        anchors.fill: parent
                        currentIndex: navigation.currentIndex
                        
                        // 监控主界面
                        MonitoringView {
                            id: monitoringView
                        }
                        
                        // 数据曲线（占位）
                        Item {
                            Text {
                                anchors.centerIn: parent
                                text: "数据曲线"
                                font.pixelSize: Theme.fontSizeXXLarge
                                color: Theme.textSecondary
                            }
                        }
                        
                        // 报警记录（占位）
                        Item {
                            Text {
                                anchors.centerIn: parent
                                text: "报警记录"
                                font.pixelSize: Theme.fontSizeXXLarge
                                color: Theme.textSecondary
                            }
                        }
                        
                        // 设置页面
                        SettingsView {
                            id: settingsView
                        }
                    }
                }
                
                // 底部导航
                Navigation {
                    id: navigation
                    width: parent.width
                    height: navigationVisible ? Theme.navigationHeight : 0
                    visible: height > 0
                    clip: true
                    
                    Behavior on height {
                        NumberAnimation {
                            duration: Theme.animationDuration
                            easing.type: Easing.InOutQuad
                        }
                    }
                    
                    // 监听导航栏的点击事件
                    onCurrentIndexChanged: {
                        resetHideTimer()
                    }
                }
            }
        }
    }
    
    // 启动时开始定时器
    Component.onCompleted: {
        hideNavigationTimer.start()
    }
    
    // 虚拟键盘
    InputPanel {
        id: inputPanel
        z: 99
        x: 0
        y: root.height
        width: root.width
        
        states: State {
            name: "visible"
            when: inputPanel.active
            PropertyChanges {
                target: inputPanel
                y: root.height - inputPanel.height
            }
        }
        transitions: Transition {
            from: ""
            to: "visible"
            reversible: true
            NumberAnimation {
                properties: "y"
                duration: Theme.animationDuration
                easing.type: Easing.InOutQuad
            }
        }
    }
}
