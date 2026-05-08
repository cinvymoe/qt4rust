// TimeCard.qml - 时间卡片组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: root
    
    property string currentTime: Qt.formatDateTime(new Date(), "hh:mm:ss")
    property string currentDate: Qt.formatDateTime(new Date(), "yyyy年MM月dd日")
    property string weekDay: Qt.formatDateTime(new Date(), "dddd")
    
    color: Qt.rgba(Theme.darkSurface.r, Theme.darkSurface.g, Theme.darkSurface.b, 0.75)
    border.color: Theme.darkBorder
    border.width: Theme.borderThin
    radius: Theme.radiusMedium
    
    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            var now = new Date()
            root.currentTime = Qt.formatDateTime(now, "hh:mm:ss")
            root.currentDate = Qt.formatDateTime(now, "yyyy年MM月dd日")
            root.weekDay = Qt.formatDateTime(now, "dddd")
        }
    }
    
    Row {
        anchors.centerIn: parent
        spacing: Theme.spacingLarge
        
        Rectangle {
            width: Theme.iconSizeLarge
            height: Theme.iconSizeLarge
            radius: Theme.iconSizeLarge / 2
            color: Theme.darkAccent
            opacity: 0.2
            anchors.verticalCenter: parent.verticalCenter
            
            Text {
                anchors.centerIn: parent
                text: "🕐"
                font.pixelSize: Theme.fontSizeLarge
            }
        }
        
        Column {
            spacing: Theme.spacingTiny
            anchors.verticalCenter: parent.verticalCenter
            
            Text {
                text: root.currentTime
                font.pixelSize: Theme.fontSizeXLarge
                font.family: Theme.fontFamilyMono
                font.weight: Font.Medium
                color: Theme.textPrimary
            }
            
            Text {
                text: root.currentDate + " " + root.weekDay
                font.pixelSize: Theme.fontSizeMedium
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
            }
        }
    }
}
