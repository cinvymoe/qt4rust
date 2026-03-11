// LoadCalibrationContent.qml - 载荷传感器校准内容
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
            
            // 标定倍率设置
            Rectangle {
                width: parent.width
                height: 246
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
                            color: "#ad46ff"
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Text {
                            text: "标定倍率设置"
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "选择标定倍率"
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                        }
                        
                        // 倍率选择按钮
                        Grid {
                            width: parent.width
                            columns: 4
                            spacing: Theme.spacingSmall
                            
                            Repeater {
                                model: [
                                    {value: "1x", label: "标准倍率"},
                                    {value: "2x", label: "2倍放大"},
                                    {value: "4x", label: "4倍放大"},
                                    {value: "customx", label: "自定义"}
                                ]
                                
                                Rectangle {
                                    width: (parent.width - 3 * Theme.spacingSmall) / 4
                                    height: 92
                                    color: index === 0 ? "#9810fa" : "#314158"
                                    border.color: index === 0 ? "#ad46ff" : "#45556c"
                                    border.width: 2
                                    radius: Theme.radiusMedium
                                    
                                    Column {
                                        anchors.centerIn: parent
                                        spacing: 4
                                        
                                        Text {
                                            text: modelData.value
                                            font.pixelSize: Theme.fontSizeXXLarge
                                            font.family: Theme.fontFamilyMono
                                            color: index === 0 ? Theme.textPrimary : Theme.textSecondary
                                            anchors.horizontalCenter: parent.horizontalCenter
                                        }
                                        
                                        Text {
                                            text: modelData.label
                                            font.pixelSize: Theme.fontSizeTiny
                                            color: index === 0 ? Theme.textPrimary : Theme.textSecondary
                                            anchors.horizontalCenter: parent.horizontalCenter
                                        }
                                    }
                                    
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: console.log("选择倍率:", modelData.value)
                                    }
                                }
                            }
                        }
                        
                        Text {
                            text: "标定倍率用于调整传感器灵敏度，适用于不同测量范围的应用场景"
                            font.pixelSize: Theme.fontSizeTiny
                            color: "#62748e"
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }
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
                            
                            onClicked: console.log("添加标定点")
                        }
                    }
                    
                    // 标定点列表
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        // 标定点 1
                        CalibrationPoint {
                            width: parent.width
                            pointNumber: 1
                            adValue: "0"
                            weightValue: "0"
                        }
                        
                        // 标定点 2
                        CalibrationPoint {
                            width: parent.width
                            pointNumber: 2
                            adValue: "10000"
                            weightValue: "5"
                        }
                    }
                }
            }
            
            // 校准说明
            Rectangle {
                width: parent.width
                height: 206
                color: "#162456"
                border.color: "#155dfc"
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
                        source: "../../../assets/images/icon-chart.svg"
                    }
                    
                    Column {
                        width: parent.width - Theme.iconSizeMedium - Theme.spacingSmall
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "载荷传感器标定说明"
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.textAccent
                        }
                        
                        Column {
                            width: parent.width
                            spacing: 4
                            
                            Text {
                                text: "• 至少需要2个标定点，建议使用3-5个标定点"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 标定点应均匀覆盖整个测量范围"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• AD值是传感器的原始输出值（模拟数字转换值）"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 使用标准砝码进行标定，确保重量准确"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 标定时应确保设备稳定，避免振动干扰"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 系统使用线性插值算法计算两点之间的重量值"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 标定点组件
    component CalibrationPoint: Rectangle {
        property int pointNumber: 1
        property string adValue: "0"
        property string weightValue: "0"
        
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
                color: "#155dfc"
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
                        text: "重量（吨）"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                    
                    TextField {
                        width: parent.width
                        height: 42
                        text: weightValue
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
                
                onClicked: console.log("删除标定点", pointNumber)
            }
        }
    }
}
