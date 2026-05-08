// DangerCard.qml - 危险/预警状态卡片组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: root

    property bool isWarning: false
    property bool isAngleAlarm: false
    property string title: "危险状态"
    property string message: "力矩超限！立即减载或降低幅度"
    property string iconSource: isWarning ?
        "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-alert.png" :
        "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-danger.png"

    readonly property color backgroundColor: isWarning ? "#f0b100" : Theme.dangerBackground
    
    color: "transparent"
    border.color: isWarning ? Theme.warningColor : Theme.dangerColor
    border.width: Theme.borderThick
    radius: Theme.radiusMedium

    Rectangle {
        anchors.fill: parent
        color: root.backgroundColor
        opacity: isWarning ? 0.2 : 0.3
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
            fillMode: Image.PreserveAspectFit
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
