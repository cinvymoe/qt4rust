// DangerCard.qml - 危险状态卡片组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: root
    
    width: parent.width
    height: 96
    color: "transparent"
    border.color: Theme.dangerColor
    border.width: Theme.borderThick
    radius: Theme.radiusMedium
    
    // 可配置属性
    property string title: "危险状态"
    property string message: "力矩超限！立即减载或降低幅度"
    property string iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-danger.png"
    
    // 半透明背景
    Rectangle {
        anchors.fill: parent
        color: Theme.dangerBackground
        opacity: 0.3
        radius: Theme.radiusMedium
    }
    
    Row {
        anchors.centerIn: parent
        spacing: Theme.spacingMedium
        
        Image {
            source: root.iconSource
            width: Theme.iconSizeLarge
            height: Theme.iconSizeLarge
            anchors.verticalCenter: parent.verticalCenter
        }
        
        Column {
            spacing: Theme.spacingTiny
            anchors.verticalCenter: parent.verticalCenter
            
            Text {
                text: root.title
                font.pixelSize: Theme.fontSizeXLarge
                font.family: Theme.fontFamilyDefault
                font.weight: Font.Medium
                color: Theme.textPrimary
            }
            
            Text {
                text: root.message
                font.pixelSize: Theme.fontSizeMedium
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
            }
        }
    }
}
