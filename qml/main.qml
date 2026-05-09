// Qt Rust Demo - 主界面 QML 文件
// 起重机力矩监测系统

import qt.rust.demo
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
    height: 600
    visible: true
    title: "Crane Monitoring System"
    color: Theme.darkBackground
    
    // i18n 翻译对象
    
    // 导航栏显示状态
    property bool navigationVisible: true
    // 顶部栏显示状态（虚拟键盘显示时隐藏）
    property bool headerVisible: true
    // 启动画面显示状态
    property bool showSplash: true
    
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
    
    // 主内容
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        
        // 全局鼠标区域，用于检测点击
        MouseArea {
            anchors.fill: parent
            onClicked: {
                root.resetHideTimer()
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
                            pageIndex: 0
                            currentIndex: stackLayout.currentIndex
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
                        root.resetHideTimer()
                    }
                }
            }
        }
    }
    
    // 启动画面覆盖层
    Rectangle {
        id: splashScreen
        anchors.fill: parent
        color: Theme.darkBackground
        visible: showSplash
        z: 1000
        
        Column {
            anchors.centerIn: parent
            spacing: Theme.spacingXLarge
            
            // Logo 图片
            Image {
                id: logoImage
                source: "assets/images/icon-logo.png"
                width: 200
                height: 200
                anchors.horizontalCenter: parent.horizontalCenter
                fillMode: Image.PreserveAspectFit
                
                // 淡入动画
                opacity: 0
                NumberAnimation on opacity {
                    from: 0
                    to: 1
                    duration: 800
                    easing.type: Easing.InOutQuad
                }
            }
            
            // 应用标题
            Text {
                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("main.title") }
                font.pixelSize: Theme.fontSizeXXLarge
                font.family: Theme.fontFamilyDefault
                color: Theme.textPrimary
                anchors.horizontalCenter: parent.horizontalCenter
                
                // 淡入动画
                opacity: 0
                NumberAnimation on opacity {
                    from: 0
                    to: 1
                    duration: 800
                    easing.type: Easing.InOutQuad
                }
            }
            
            // 副标题
            Text {
                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("main.subtitle") }
                font.pixelSize: Theme.fontSizeMedium
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
                anchors.horizontalCenter: parent.horizontalCenter
                
                // 淡入动画
                opacity: 0
                NumberAnimation on opacity {
                    from: 0
                    to: 1
                    duration: 800
                    easing.type: Easing.InOutQuad
                }
            }
            
            // 加载指示器
            Item {
                width: 200
                height: 4
                anchors.horizontalCenter: parent.horizontalCenter
                
                Rectangle {
                    anchors.fill: parent
                    color: Theme.darkSurface
                    radius: 2
                    
                    Rectangle {
                        id: progressBar
                        height: parent.height
                        width: 0
                        color: Theme.darkAccent
                        radius: 2
                        
                        // 进度条动画
                        NumberAnimation on width {
                            from: 0
                            to: 200
                            duration: 2000
                            easing.type: Easing.InOutQuad
                        }
                    }
                }
            }
        }
        
        // 版本信息
        Text {
            text: "v1.0.0"
            font.pixelSize: Theme.fontSizeSmall
            color: Theme.textTertiary
            anchors.bottom: parent.bottom
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottomMargin: Theme.spacingLarge
        }
        
        // 淡出动画
        opacity: 1
        Behavior on opacity {
            NumberAnimation {
                duration: 500
                easing.type: Easing.InOutQuad
            }
        }
    }
    
    // 启动画面定时器
    Timer {
        id: splashTimer
        interval: 2500  // 2.5秒后隐藏启动画面
        running: true
        repeat: false
        onTriggered: {
            splashScreen.opacity = 0
            // 延迟隐藏启动画面
            hideSplashTimer.start()
        }
    }
    
    // 隐藏启动画面定时器
    Timer {
        id: hideSplashTimer
        interval: 500  // 等待淡出动画完成
        running: false
        repeat: false
        onTriggered: {
            showSplash = false
            hideNavigationTimer.start()
        }
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
                root.resetHideTimer()
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
