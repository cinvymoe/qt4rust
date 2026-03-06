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
    // 顶部栏显示状态（虚拟键盘显示时隐藏）
    property bool headerVisible: true
    
    // 自动隐藏定时器
    Timer {
        id: hideNavigationTimer
        interval: 3000  // 3秒
        running: false
        repeat: false
        onTriggered: {
            // 同时隐藏底部导航栏和顶部导航栏
            navigationVisible = false
            headerVisible = false
        }
    }
    
    // 重置定时器的函数
    function resetHideTimer() {
        navigationVisible = true
        headerVisible = true
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
                
                // 主内容区域
                Item {
                    id: contentArea
                    width: parent.width
                    height: parent.height - (navigationVisible ? navigation.height : 0)
                    
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
                            headerVisible: root.headerVisible
                        }
                        
                        // 数据曲线页面
                        ChartView {
                            id: chartView
                        }
                        
                        // 报警记录页面
                        AlarmRecordView {
                            id: alarmRecordView
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
        
        // 监听虚拟键盘状态变化
        onActiveChanged: {
            if (active) {
                // 键盘显示时，隐藏顶部栏和底部导航
                headerVisible = false
                navigationVisible = false
                hideNavigationTimer.stop()
            } else {
                // 键盘隐藏时，恢复顶部栏并重启自动隐藏定时器
                headerVisible = true
                resetHideTimer()
            }
        }
        
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
