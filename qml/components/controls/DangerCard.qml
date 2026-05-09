// DangerCard.qml - 危险/预警状态卡片组件
import qt.rust.demo
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: root


    width: parent.width
    height: 96
    color: "transparent"
    border.color: isWarning ? Theme.warningColor : Theme.dangerColor
    border.width: Theme.borderThick
    radius: Theme.radiusMedium

    // 可配置属性
    property bool isWarning: false  // true=预警(黄色), false=危险(红色)
    property bool isAngleAlarm: false
    property string title: TranslationBridge.translate("danger.title.danger")
    property string message: TranslationBridge.translate("danger.message.danger")
    // 根据状态自动选择图标：预警使用黄色感叹号，危险使用红色危险图标
    property string iconSource: isWarning ?
        "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-alert.png" :
        "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-danger.png"

    // 计算背景色：预警使用黄色半透明，危险使用红色半透明
    readonly property color backgroundColor: isWarning ? "#f0b100" : Theme.dangerBackground

    // 半透明背景
    Rectangle {
        anchors.fill: parent
        color: root.backgroundColor
        opacity: isWarning ? 0.2 : 0.3  // 预警稍透明一些
        radius: Theme.radiusMedium
    }

    Row {
        anchors.centerIn: parent
        spacing: Theme.spacingMedium

        // 图标
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
