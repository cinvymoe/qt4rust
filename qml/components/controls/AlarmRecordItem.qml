// AlarmRecordItem.qml - 报警记录项组件
import qt.rust.demo
import QtQuick
import QtQuick.Layouts
import "../../styles"

Rectangle {
    id: root
    
    TranslationBridge { id: tr }
    
    // 公开属性
    property string alarmType: "warning"  // "warning" 或 "danger"
    property string title: "预警提示"
    property string message: "预警：力矩达到 75.9%，注意安全！"
    property string date: "2026/3/6"
    property string time: "15:47:36"
    property string momentValue: "75.9%"
    
    // 根据类型设置样式
    readonly property color bgColor: alarmType === "danger" 
        ? Qt.rgba(70/255, 8/255, 9/255, 0.3)
        : Qt.rgba(67/255, 32/255, 4/255, 0.3)
    
    readonly property color borderColor: alarmType === "danger"
        ? Theme.dangerLight
        : Theme.warningColor
    
    readonly property color titleColor: alarmType === "danger"
        ? "#ff6467"
        : "#fdc700"
    
    readonly property string iconSource: alarmType === "danger"
        ? "../../assets/images/icon-danger.png"
        : "../../assets/images/icon-alert.png"
    
    // 组件样式
    // 在 Layout 中使用时的属性
    Layout.fillWidth: true
    Layout.preferredHeight: 116
    
    // 在 ListView 中使用时的属性
    height: 116
    
    color: bgColor
    border.color: borderColor
    border.width: 4
    radius: Theme.radiusMedium
    
    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 20
        anchors.rightMargin: 16
        anchors.topMargin: 16
        spacing: Theme.spacingMedium - 4
        
        // 图标
        Image {
            Layout.preferredWidth: Theme.iconSizeMedium
            Layout.preferredHeight: Theme.iconSizeMedium
            Layout.alignment: Qt.AlignTop
            source: root.iconSource
            fillMode: Image.PreserveAspectFit
        }
        
        // 内容区域
        ColumnLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSmall
            
            // 标题
            Text {
                text: root.title
                font.pixelSize: Theme.fontSizeNormal
                font.family: Theme.fontFamilyDefault
                color: root.titleColor
            }
            
            // 消息内容
            Text {
                text: root.message
                font.pixelSize: Theme.fontSizeMedium
                font.family: Theme.fontFamilyDefault
                color: "#e2e8f0"
            }
            
            // 时间和数据信息
            RowLayout {
                spacing: Theme.spacingMedium
                
                Text {
                    text: root.date
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textTertiary
                }
                
                Text {
                    text: root.time
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textTertiary
                }
                
                Text {
                    text: tr.translate("alarm.momentValue") + ": " + root.momentValue
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textTertiary
                }
            }
        }
    }
}