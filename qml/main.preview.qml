// Qt Rust Demo - 预览版主界面
// 用于 qmlscene 预览，不依赖 Rust 后端

import QtQuick
import QtQuick.Controls
import QtQuick.Window
import QtQuick.Layouts

Window {
    id: root
    width: 1024
    height: 800
    visible: true
    title: "Crane Monitoring System - Preview"
    
    // 深色主题颜色
    readonly property color darkBackground: "#0f172b"
    readonly property color darkSurface: "#1d293d"
    readonly property color darkBorder: "#314158"
    readonly property color darkAccent: "#2b7fff"
    readonly property color dangerColor: "#e7000b"
    readonly property color dangerBackground: "#460809"
    readonly property color dangerLight: "#fb2c36"
    readonly property color warningColor: "#f0b100"
    readonly property color successColor: "#00c950"
    readonly property color textPrimary: "#ffffff"
    readonly property color textSecondary: "#cad5e2"
    readonly property color textTertiary: "#90a1b9"
    readonly property color textAccent: "#51a2ff"
    
    Rectangle {
        anchors.fill: parent
        color: darkBackground
        
        Column {
            anchors.fill: parent
            spacing: 0
            
            // 顶部栏
            Rectangle {
                id: header
                width: parent.width
                height: 68
                color: dangerBackground
                border.color: dangerColor
                border.width: 0.667
                
                Row {
                    anchors.fill: parent
                    anchors.margins: 16
                    spacing: 8
                    
                    Image {
                        source: "assets/images/icon-logo.png"
                        width: 24
                        height: 24
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    
                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        
                        Text {
                            text: "汽车吊力矩监测系统"
                            font.pixelSize: 18
                            color: textPrimary
                        }
                        
                        Text {
                            text: "Crane Moment Monitoring System"
                            font.pixelSize: 12
                            color: textTertiary
                        }
                    }
                    
                    Item { Layout.fillWidth: true }
                    
                    Rectangle {
                        width: 108
                        height: 32
                        radius: 10
                        color: dangerColor
                        opacity: 0.62
                        anchors.verticalCenter: parent.verticalCenter
                        
                        Row {
                            anchors.centerIn: parent
                            spacing: 8
                            
                            Image {
                                source: "assets/images/icon-alert.png"
                                width: 20
                                height: 20
                            }
                            
                            Text {
                                text: "危险报警"
                                font.pixelSize: 14
                                color: textPrimary
                            }
                        }
                    }
                }
            }
            
            // 主内容
            Item {
                width: parent.width
                height: parent.height - header.height - navigation.height
                
                Text {
                    anchors.centerIn: parent
                    text: "监控界面预览\n请使用 cargo run 运行完整版本"
                    font.pixelSize: 24
                    color: textSecondary
                    horizontalAlignment: Text.AlignHCenter
                }
            }
            
            // 底部导航
            Rectangle {
                id: navigation
                width: parent.width
                height: 68
                color: darkSurface
                border.color: "#000000"
                border.width: 0.667
                
                Row {
                    anchors.fill: parent
                    
                    Repeater {
                        model: ["主界面", "数据曲线", "报警记录", "设置"]
                        
                        Rectangle {
                            width: parent.width / 4
                            height: parent.height
                            color: index === 0 ? darkBorder : "transparent"
                            
                            Column {
                                anchors.centerIn: parent
                                spacing: 4
                                
                                Rectangle {
                                    width: 24
                                    height: 24
                                    color: index === 0 ? textAccent : textTertiary
                                    anchors.horizontalCenter: parent.horizontalCenter
                                }
                                
                                Text {
                                    text: modelData
                                    font.pixelSize: 12
                                    color: index === 0 ? textAccent : textTertiary
                                    anchors.horizontalCenter: parent.horizontalCenter
                                }
                            }
                            
                            Rectangle {
                                visible: index === 0
                                width: parent.width
                                height: 4
                                color: darkAccent
                                anchors.top: parent.top
                            }
                        }
                    }
                }
            }
        }
    }
}
