// StatusCard.qml - 状态卡片组件
import QtQuick
import "../../styles"

Rectangle {
    id: root
    
    property string iconSource: ""
    property string title: ""
    property string value: ""
    property string unit: ""
    property string subtitle: ""
    property real progress: 0.0  // 0.0 到 1.0
    property bool showProgress: false
    property color progressColor: Theme.successColor
    
    color: Theme.darkSurface
    border.color: Theme.darkBorder
    border.width: Theme.borderThin
    radius: Theme.radiusMedium
    
    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingSmall
        
        // 标题行
        Row {
            width: parent.width
            spacing: Theme.spacingSmall
            
            Image {
                source: root.iconSource
                width: Theme.iconSizeSmall
                height: Theme.iconSizeSmall
                visible: root.iconSource !== ""
            }
            
            Text {
                text: root.title
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textSecondary
            }
        }
        
        // 数值行
        Row {
            width: parent.width
            spacing: 4
            
            Text {
                text: root.value
                font.pixelSize: Theme.fontSizeHuge
                font.family: Theme.fontFamilyMono
                color: Theme.textPrimary
            }
            
            Text {
                text: root.unit
                font.pixelSize: Theme.fontSizeLarge
                color: Theme.textTertiary
                anchors.verticalCenter: parent.verticalCenter
                anchors.verticalCenterOffset: 4
            }
        }
        
        // 副标题
        Text {
            width: parent.width
            text: root.subtitle
            font.pixelSize: Theme.fontSizeTiny
            color: Theme.textTertiary
            visible: root.subtitle !== ""
        }
        
        // 进度条
        Rectangle {
            width: parent.width
            height: 8
            color: Theme.darkBorder
            radius: Theme.radiusLarge
            visible: root.showProgress
            
            Rectangle {
                width: parent.width * root.progress
                height: parent.height
                color: root.progressColor
                radius: parent.radius
            }
        }
    }
}
