// Header.qml - 顶部栏组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: header
    height: Theme.headerHeight
    
    // 状态属性
    property string title: "汽车吊力矩监测系统"
    property string subtitle: "Crane Moment Monitoring System"
    property bool alertActive: false
    property bool isWarning: false  // 预警状态
    property bool isDanger: false   // 报警状态
    
    // 根据状态动态计算颜色
    // 正常状态：使用表面色背景
    // 预警状态：使用黄色背景和边框
    // 报警状态：使用红色背景和边框
    color: isDanger ? Theme.dangerBackground : (isWarning ? "#2a1f00" : Theme.darkSurface)
    border.color: isDanger ? Theme.dangerColor : (isWarning ? Theme.warningColor : Theme.darkBorder)
    border.width: isDanger || isWarning ? Theme.borderNormal : Theme.borderThin
    
    // 报警文本根据状态动态变化
    readonly property string alertText: isDanger ? "危险报警" : (isWarning ? "力矩预警" : "")
    
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
        
        // 报警按钮 - 根据预警/报警状态显示不同颜色
        Rectangle {
            id: alertButton
            width: 108
            height: Theme.buttonHeightSmall
            radius: Theme.radiusMedium
            // 预警状态使用黄色，报警状态使用红色
            color: header.isDanger ? Theme.dangerColor : Theme.warningColor
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
