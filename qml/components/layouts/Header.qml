// Header.qml - 顶部栏组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: header
    height: Theme.headerHeight
    color: Theme.dangerBackground
    border.color: Theme.dangerColor
    border.width: Theme.borderThin
    
    // 状态属性
    property string title: "汽车吊力矩监测系统"
    property string subtitle: "Crane Moment Monitoring System"
    property bool alertActive: false
    property string alertText: "危险报警"
    
    Row {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingSmall
        
        // Logo 图标
        Image {
            id: logoIcon
            source: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-logo.png"
            width: Theme.iconSizeMedium
            height: Theme.iconSizeMedium
            anchors.verticalCenter: parent.verticalCenter
        }
        
        // 标题区域
        Column {
            id: titleColumn
            anchors.verticalCenter: parent.verticalCenter
            spacing: 0
            
            Text {
                text: header.title
                font.pixelSize: Theme.fontSizeNormal
                font.family: Theme.fontFamilyDefault
                color: Theme.textPrimary
            }
            
            Text {
                text: header.subtitle
                font.pixelSize: Theme.fontSizeTiny
                font.family: Theme.fontFamilyDefault
                color: Theme.textTertiary
            }
        }
        
        // 弹性空间
        Item {
            width: parent.width - logoIcon.width - titleColumn.width - alertButton.width - parent.spacing * 3 - parent.anchors.margins * 2
            height: 1
        }
        
        // 报警按钮
        Rectangle {
            id: alertButton
            width: 108
            height: Theme.buttonHeightSmall
            radius: Theme.radiusMedium
            color: Theme.dangerColor
            opacity: header.alertActive ? Theme.opacityMedium : 0
            visible: header.alertActive
            anchors.verticalCenter: parent.verticalCenter
            
            Row {
                anchors.centerIn: parent
                spacing: Theme.spacingSmall
                
                Image {
                    source: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-alert.png"
                    width: Theme.iconSizeSmall
                    height: Theme.iconSizeSmall
                    anchors.verticalCenter: parent.verticalCenter
                }
                
                Text {
                    text: header.alertText
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textPrimary
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }
    }
}
