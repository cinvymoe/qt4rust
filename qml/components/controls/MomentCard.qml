// MomentCard.qml - 力矩百分比卡片组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: momentCard
    
    property real percentage: 94.8
    property real warningThreshold: 90.0
    property real dangerThreshold: 100.0
    
    color: Theme.darkSurface
    border.color: Theme.darkBorder
    border.width: Theme.borderThick
    radius: Theme.radiusMedium
    
    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingMedium
        
        // 标题行
        Item {
            width: parent.width
            height: Theme.buttonHeightSmall
            
            Row {
                spacing: Theme.spacingSmall
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                
                Image {
                    source: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-moment.png"
                    width: Theme.iconSizeMedium
                    height: Theme.iconSizeMedium
                    anchors.verticalCenter: parent.verticalCenter
                }
                
                Text {
                    text: "力矩百分比"
                    font.pixelSize: Theme.fontSizeNormal
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textSecondary
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            
            Rectangle {
                width: 80
                height: Theme.buttonHeightSmall
                radius: Theme.radiusSmall
                color: momentCard.percentage >= momentCard.dangerThreshold ? Theme.dangerColor : 
                       momentCard.percentage >= momentCard.warningThreshold ? Theme.warningColor : 
                       Theme.successColor
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                
                Text {
                    text: momentCard.percentage >= momentCard.dangerThreshold ? "超限危险" : 
                          momentCard.percentage >= momentCard.warningThreshold ? "预警状态" : 
                          "正常"
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textPrimary
                    anchors.centerIn: parent
                }
            }
        }
        
        // 数值显示
        Row {
            spacing: 4
            
            Text {
                text: momentCard.percentage.toFixed(1)
                font.pixelSize: Theme.fontSizeDisplay
                font.family: Theme.fontFamilyMono
                color: Theme.textPrimary
            }
            
            Text {
                text: "%"
                font.pixelSize: Theme.fontSizeXXLarge
                font.family: Theme.fontFamilyDefault
                color: Theme.textTertiary
                anchors.verticalCenter: parent.verticalCenter
                anchors.verticalCenterOffset: 8
            }
        }
        
        // 进度条和标记
        Column {
            width: parent.width
            spacing: Theme.spacingSmall
            
            // 进度条
            Rectangle {
                width: parent.width
                height: 16
                radius: Theme.radiusLarge
                color: Theme.darkBorder
                
                Rectangle {
                    width: parent.width * (momentCard.percentage / 100)
                    height: parent.height
                    radius: Theme.radiusLarge
                    color: momentCard.percentage >= momentCard.dangerThreshold ? Theme.dangerLight : 
                           momentCard.percentage >= momentCard.warningThreshold ? Theme.warningColor : 
                           Theme.successColor
                }
            }
            
            // 标记行
            Item {
                width: parent.width
                height: Theme.fontSizeSmall + 4
                
                Row {
                    anchors.fill: parent
                    spacing: 0
                    
                    Text {
                        text: "0%"
                        font.pixelSize: Theme.fontSizeSmall
                        font.family: Theme.fontFamilyDefault
                        color: Theme.textTertiary
                    }
                    
                    Item { 
                        width: parent.width * 0.25
                        height: 1 
                    }
                    
                    Text {
                        text: "预警 " + momentCard.warningThreshold.toFixed(0) + "%"
                        font.pixelSize: Theme.fontSizeSmall
                        font.family: Theme.fontFamilyDefault
                        color: Theme.warningColor
                    }
                    
                    Item { 
                        width: parent.width * 0.15
                        height: 1 
                    }
                    
                    Text {
                        text: "危险 " + momentCard.dangerThreshold.toFixed(0) + "%"
                        font.pixelSize: Theme.fontSizeSmall
                        font.family: Theme.fontFamilyDefault
                        color: Theme.dangerLight
                    }
                    
                    Item { 
                        width: parent.width * 0.1
                        height: 1 
                    }
                    
                    Text {
                        text: "100%"
                        font.pixelSize: Theme.fontSizeSmall
                        font.family: Theme.fontFamilyDefault
                        color: Theme.textTertiary
                    }
                }
            }
        }
    }
}
