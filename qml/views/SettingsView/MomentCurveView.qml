// MomentCurveView.qml - 力矩曲线子页面
import QtQuick
import QtQuick.Controls
import "../../styles"
import "../../components/controls"

Flickable {
    id: momentCurveView
    width: parent.width
    height: parent.height
    contentHeight: contentColumn.height
    clip: true
    
    Column {
        id: contentColumn
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
                
                // 1. 说明卡片
                Rectangle {
                    width: parent.width
                    height: 153.333
                    color: "#162456"
                    border.color: "#1447e6"
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        anchors.fill: parent
                        anchors.topMargin: 16.667
                        anchors.leftMargin: 16.667
                        anchors.rightMargin: 16.667
                        spacing: 0
                        
                        Row {
                            width: parent.width
                            spacing: 12
                            
                            Text {
                                text: "ℹ"
                                font.pixelSize: Theme.fontSizeLarge
                                color: "#dbeafe"
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "力矩曲线图说明："
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: "#dbeafe"
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        Column {
                            width: parent.width
                            spacing: 4
                            topPadding: Theme.spacingSmall
                            opacity: 0.8
                            
                            Repeater {
                                model: [
                                    "• 曲线显示不同工作半径下的额定载荷能力",
                                    "• 实际作业时，载荷必须低于对应半径的额定值",
                                    "• 工作半径越大，额定载荷越小",
                                    "• 不同臂长配置对应不同的性能曲线"
                                ]
                                
                                Text {
                                    text: modelData
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.family: Theme.fontFamilyDefault
                                    color: "#bedbff"
                                    width: parent.width
                                }
                            }
                        }
                    }
                }
                
                // 2. 额定载荷曲线
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
                        topPadding: 16.667
                        bottomPadding: Theme.borderThin
                        leftPadding: 16.667
                        rightPadding: 16.667
                        
                        // 标题
                        Item {
                            width: parent.width - 2 * 16.667
                            height: 28
                            
                            Rectangle {
                                width: 4
                                height: 24
                                color: Theme.successColor
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "额定载荷曲线"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                anchors.left: parent.left
                                anchors.leftMargin: 12
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 图表
                        LoadCurveChart {
                            width: parent.width - 2 * 16.667
                            height: 384
                        }
                        
                        // 图例
                        Row {
                            width: parent.width - 2 * 16.667
                            spacing: Theme.spacingLarge
                            
                            Repeater {
                                model: [
                                    {label: "主臂配置", color: "#22c55e"},
                                    {label: "主臂+副臂", color: "#3b82f6"},
                                    {label: "最大臂长", color: "#f59e0b"}
                                ]
                                
                                Rectangle {
                                    width: (parent.width - 2 * Theme.spacingLarge) / 3
                                    height: 64
                                    color: Theme.darkBackground
                                    radius: Theme.radiusSmall
                                    
                                    Column {
                                        anchors.fill: parent
                                        anchors.margins: 12
                                        spacing: 4
                                        
                                        Row {
                                            spacing: Theme.spacingSmall
                                            
                                            Rectangle {
                                                width: 16
                                                height: 4
                                                color: modelData.color
                                                radius: Theme.radiusSmall
                                                anchors.verticalCenter: parent.verticalCenter
                                            }
                                            
                                            Text {
                                                text: modelData.label
                                                font.pixelSize: Theme.fontSizeSmall
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textSecondary
                                            }
                                        }
                                        
                                        Text {
                                            text: index === 0 ? "臂长约30m，最大载荷25吨" : 
                                                  index === 1 ? "臂长约40m，最大载荷20吨" : 
                                                  "臂长约50m，最大载荷15吨"
                                            font.pixelSize: Theme.fontSizeTiny
                                            font.family: Theme.fontFamilyDefault
                                            color: Theme.textTertiary
                                            wrapMode: Text.WordWrap
                                            width: parent.width
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 3. 使用示例
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
                        topPadding: 16.667
                        bottomPadding: Theme.borderThin
                        leftPadding: 16.667
                        rightPadding: 16.667
                        
                        // 标题
                        Item {
                            width: parent.width - 2 * 16.667
                            height: 28
                            
                            Rectangle {
                                width: 4
                                height: 24
                                color: "#ff6900"
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "使用示例"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                anchors.left: parent.left
                                anchors.leftMargin: 12
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 示例卡片容器
                        Column {
                            width: parent.width - 2 * 16.667
                            spacing: 12
                            
                            // 安全作业示例
                            Rectangle {
                                width: parent.width
                                height: 148
                                color: Theme.darkBackground
                                border.color: Theme.successColor
                                border.width: 4
                                radius: Theme.radiusMedium
                                
                                Column {
                                    anchors.fill: parent
                                    anchors.topMargin: Theme.spacingMedium
                                    anchors.leftMargin: 20
                                    anchors.rightMargin: Theme.spacingMedium
                                    spacing: Theme.spacingSmall
                                    
                                    Text {
                                        text: "✓ 安全作业示例"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: "#05df72"
                                        width: parent.width
                                    }
                                    
                                    Text {
                                        text: "主臂配置，工作半径10m，吊运载荷12吨"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textSecondary
                                        width: parent.width
                                    }
                                    
                                    Column {
                                        width: parent.width
                                        spacing: 4
                                        
                                        Repeater {
                                            model: [
                                                "• 根据曲线，10m半径时额定载荷约17吨",
                                                "• 实际载荷12吨，低于额定载荷17吨",
                                                "• 载荷率为70.6%，处于安全范围"
                                            ]
                                            
                                            Text {
                                                text: modelData
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                width: parent.width
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // 预警作业示例
                            Rectangle {
                                width: parent.width
                                height: 148
                                color: Theme.darkBackground
                                border.color: Theme.warningColor
                                border.width: 4
                                radius: Theme.radiusMedium
                                
                                Column {
                                    anchors.fill: parent
                                    anchors.topMargin: Theme.spacingMedium
                                    anchors.leftMargin: 20
                                    anchors.rightMargin: Theme.spacingMedium
                                    spacing: Theme.spacingSmall
                                    
                                    Text {
                                        text: "⚠ 预警作业示例"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: "#fdc700"
                                        width: parent.width
                                    }
                                    
                                    Text {
                                        text: "主臂配置，工作半径15m，吊运载荷10吨"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textSecondary
                                        width: parent.width
                                    }
                                    
                                    Column {
                                        width: parent.width
                                        spacing: 4
                                        
                                        Repeater {
                                            model: [
                                                "• 根据曲线，15m半径时额定载荷约12吨",
                                                "• 实际载荷10吨，载荷率为83.3%",
                                                "• 超过75%预警线，建议减载或减小半径"
                                            ]
                                            
                                            Text {
                                                text: modelData
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                width: parent.width
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // 危险作业示例
                            Rectangle {
                                width: parent.width
                                height: 148
                                color: Theme.darkBackground
                                border.color: Theme.dangerLight
                                border.width: 4
                                radius: Theme.radiusMedium
                                
                                Column {
                                    anchors.fill: parent
                                    anchors.topMargin: Theme.spacingMedium
                                    anchors.leftMargin: 20
                                    anchors.rightMargin: Theme.spacingMedium
                                    spacing: Theme.spacingSmall
                                    
                                    Text {
                                        text: "✗ 危险作业示例"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: "#ff6467"
                                        width: parent.width
                                    }
                                    
                                    Text {
                                        text: "最大臂长配置，工作半径20m，吊运载荷3吨"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textSecondary
                                        width: parent.width
                                    }
                                    
                                    Column {
                                        width: parent.width
                                        spacing: 4
                                        
                                        Repeater {
                                            model: [
                                                "• 根据曲线，20m半径时额定载荷约3.2吨",
                                                "• 实际载荷3吨，载荷率为93.8%",
                                                "• 超过90%危险线！必须立即减载"
                                            ]
                                            
                                            Text {
                                                text: modelData
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                width: parent.width
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
