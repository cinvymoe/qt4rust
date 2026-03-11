// RadiusCalibrationContent.qml - 半径传感器校准内容
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../../styles"

Item {
    id: root
    
    Flickable {
        anchors.fill: parent
        anchors.leftMargin: 154
        anchors.rightMargin: 154
        contentHeight: calibrationContent.height
        clip: true
        
        Column {
            id: calibrationContent
            width: parent.width
            spacing: Theme.spacingLarge
            topPadding: Theme.spacingMedium
            
            // 半径范围设置
            Rectangle {
                width: parent.width
                height: 206
                color: Theme.darkSurface
                border.color: Theme.darkBorder
                border.width: Theme.borderThin
                radius: Theme.radiusMedium
                
                Column {
                    anchors.fill: parent
                    anchors.margins: 25
                    spacing: Theme.spacingMedium
                    
                    // 标题
                    Row {
                        width: parent.width
                        height: 28
                        spacing: Theme.spacingSmall
                        
                        Rectangle {
                            width: 4
                            height: 24
                            color: "#00c950"
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Text {
                            text: "半径测量范围"
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Row {
                        width: parent.width
                        spacing: Theme.spacingMedium
                        
                        Column {
                            width: (parent.width - Theme.spacingMedium) / 2
                            spacing: Theme.spacingSmall
                            
                            Text {
                                text: "最小半径（m）"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            TextField {
                                width: parent.width
                                height: 42
                                text: "3.0"
                                font.pixelSize: Theme.fontSizeMedium
                                font.family: Theme.fontFamilyMono
                                color: Theme.textPrimary
                                
                                background: Rectangle {
                                    color: "#314158"
                                    border.color: "#45556c"
                                    border.width: Theme.borderThin
                                    radius: Theme.radiusSmall
                                }
                            }
                        }
                        
                        Column {
                            width: (parent.width - Theme.spacingMedium) / 2
                            spacing: Theme.spacingSmall
                            
                            Text {
                                text: "最大半径（m）"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            TextField {
                                width: parent.width
                                height: 42
                                text: "30.0"
                                font.pixelSize: Theme.fontSizeMedium
                                font.family: Theme.fontFamilyMono
                                color: Theme.textPrimary
                                
                                background: Rectangle {
                                    color: "#314158"
                                    border.color: "#45556c"
                                    border.width: Theme.borderThin
                                    radius: Theme.radiusSmall
                                }
                            }
                        }
                    }
                    
                    Text {
                        text: "设置半径传感器的有效测量范围，超出范围将触发报警"
                        font.pixelSize: Theme.fontSizeTiny
                        color: "#62748e"
                        wrapMode: Text.WordWrap
                        width: parent.width
                    }
                }
            }
            
            // 多点标定设置
            Rectangle {
                width: parent.width
                height: 326
                color: Theme.darkSurface
                border.color: Theme.darkBorder
                border.width: Theme.borderThin
                radius: Theme.radiusMedium
                
                Column {
                    anchors.fill: parent
                    anchors.margins: 25
                    spacing: Theme.spacingMedium
                    
                    // 标题和添加按钮
                    Row {
                        width: parent.width
                        height: 40
                        
                        Row {
                            spacing: Theme.spacingSmall
                            anchors.verticalCenter: parent.verticalCenter
                            
                            Rectangle {
                                width: 4
                                height: 24
                                color: Theme.darkAccent
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "多点标定设置"
                                font.pixelSize: Theme.fontSizeLarge
                                font.weight: Font.Medium
                                color: Theme.textPrimary
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        Item { Layout.fillWidth: true; width: parent.width - 132 - 136 }
                        
                        Button {
                            width: 136
                            height: 40
                            anchors.verticalCenter: parent.verticalCenter
                            
                            background: Rectangle {
                                color: "#155dfc"
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: Row {
                                spacing: Theme.spacingSmall
                                anchors.centerIn: parent
                                
                                Text {
                                    text: "+"
                                    font.pixelSize: Theme.fontSizeMedium
                                    color: Theme.textPrimary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                                
                                Text {
                                    text: "添加标定点"
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.weight: Font.Medium
                                    color: Theme.textPrimary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                            
                            onClicked: console.log("添加半径标定点")
                        }
                    }
                    
                    // 标定点列表
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        // 标定点 1
                        RadiusCalibrationPoint {
                            width: parent.width
                            pointNumber: 1
                            adValue: "5000"
                            radiusValue: "3.0"
                        }
                        
                        // 标定点 2
                        RadiusCalibrationPoint {
                            width: parent.width
                            pointNumber: 2
                            adValue: "25000"
                            radiusValue: "30.0"
                        }
                    }
                }
            }
            
            // 校准说明
            Rectangle {
                width: parent.width
                height: 186
                color: "#0d2d1f"
                border.color: "#00c950"
                border.width: Theme.borderThin
                radius: Theme.radiusMedium
                
                Row {
                    anchors.fill: parent
                    anchors.margins: 17
                    spacing: Theme.spacingSmall
                    
                    Image {
                        width: Theme.iconSizeMedium
                        height: Theme.iconSizeMedium
                        anchors.top: parent.top
                        anchors.topMargin: 4
                        source: "../../../assets/images/icon-radius.png"
                    }
                    
                    Column {
                        width: parent.width - Theme.iconSizeMedium - Theme.spacingSmall
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "半径传感器标定说明"
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.successColor
                        }
                        
                        Column {
                            width: parent.width
                            spacing: 4
                            
                            Text {
                                text: "• 至少需要2个标定点，建议在最小和最大工作半径处标定"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 使用精确测量工具确定实际工作半径"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 标定时确保臂架完全伸展或收缩到位"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 半径传感器通常为拉绳式或角度换算，需定期检查"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 系统使用线性插值算法计算两点之间的半径值"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 半径标定点组件
    component RadiusCalibrationPoint: Rectangle {
        property int pointNumber: 1
        property string adValue: "0"
        property string radiusValue: "0"
        
        height: 104
        color: Theme.darkBackground
        border.color: "#45556c"
        border.width: Theme.borderThin
        radius: Theme.radiusMedium
        
        Row {
            anchors.fill: parent
            anchors.margins: 17
            spacing: Theme.spacingMedium
            
            // 序号
            Rectangle {
                width: 32
                height: 32
                radius: 16
                color: "#00c950"
                anchors.verticalCenter: parent.verticalCenter
                
                Text {
                    anchors.centerIn: parent
                    text: pointNumber
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyMono
                    color: Theme.textPrimary
                }
            }
            
            // 输入字段
            Row {
                width: parent.width - 32 - 36 - Theme.spacingMedium * 3
                spacing: Theme.spacingMedium
                anchors.verticalCenter: parent.verticalCenter
                
                Column {
                    width: (parent.width - Theme.spacingMedium) / 2
                    spacing: Theme.spacingSmall
                    
                    Text {
                        text: "AD值"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                    
                    TextField {
                        width: parent.width
                        height: 42
                        text: adValue
                        font.pixelSize: Theme.fontSizeMedium
                        font.family: Theme.fontFamilyMono
                        color: Theme.textPrimary
                        
                        background: Rectangle {
                            color: "#314158"
                            border.color: "#45556c"
                            border.width: Theme.borderThin
                            radius: Theme.radiusSmall
                        }
                    }
                }
                
                Column {
                    width: (parent.width - Theme.spacingMedium) / 2
                    spacing: Theme.spacingSmall
                    
                    Text {
                        text: "半径（m）"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                    
                    TextField {
                        width: parent.width
                        height: 42
                        text: radiusValue
                        font.pixelSize: Theme.fontSizeMedium
                        font.family: Theme.fontFamilyMono
                        color: Theme.textPrimary
                        
                        background: Rectangle {
                            color: "#314158"
                            border.color: "#45556c"
                            border.width: Theme.borderThin
                            radius: Theme.radiusSmall
                        }
                    }
                }
            }
            
            // 删除按钮
            Button {
                width: 36
                height: 36
                anchors.verticalCenter: parent.verticalCenter
                
                background: Rectangle {
                    color: "transparent"
                    radius: Theme.radiusMedium
                }
                
                contentItem: Text {
                    text: "🗑"
                    font.pixelSize: Theme.fontSizeLarge
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    color: "#62748e"
                }
                
                onClicked: console.log("删除半径标定点", pointNumber)
            }
        }
    }
}
