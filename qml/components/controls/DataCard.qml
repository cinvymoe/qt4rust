// DataCard.qml - 数据卡片组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: dataCard
    
    property string iconSource: ""
    property string label: ""
    property string value: "0.0"
    property string unit: ""
    property string description: ""
    property bool showProgress: false
    property real progress: 0.0
    
    color: Theme.darkSurface
    border.color: Theme.darkBorder
    border.width: Theme.borderThin
    radius: Theme.radiusMedium
    
    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingSmall
        
        // 标签行
        Row {
            spacing: Theme.spacingSmall
            width: parent.width
            
            Image {
                source: dataCard.iconSource
                width: Theme.iconSizeSmall
                height: Theme.iconSizeSmall
                anchors.verticalCenter: parent.verticalCenter
            }
            
            Text {
                text: dataCard.label
                font.pixelSize: Theme.fontSizeSmall
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
                anchors.verticalCenter: parent.verticalCenter
            }
        }
        
        // 数值行
        Row {
            spacing: 8
            
            Text {
                text: dataCard.value
                font.pixelSize: Theme.fontSizeHuge
                font.family: Theme.fontFamilyMono
                color: Theme.textPrimary
            }
            
            Text {
                text: dataCard.unit
                font.pixelSize: Theme.fontSizeLarge
                font.family: Theme.fontFamilyDefault
                color: Theme.textTertiary
                anchors.verticalCenter: parent.verticalCenter
                anchors.verticalCenterOffset: 4
            }
        }
        
        // 描述文本
        Text {
            text: dataCard.description
            font.pixelSize: Theme.fontSizeTiny
            font.family: Theme.fontFamilyDefault
            color: Theme.textTertiary
            width: parent.width
        }
        
        // 进度条
        Rectangle {
            visible: dataCard.showProgress
            width: parent.width
            height: 8
            radius: Theme.radiusLarge
            color: Theme.darkBorder
            
            Rectangle {
                width: parent.width * dataCard.progress
                height: parent.height
                radius: Theme.radiusLarge
                color: Theme.successColor
            }
        }
    }
}
