// TimeRangeFilter.qml - 时间范围筛选组件
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"
import "../dialogs"

RowLayout {
    id: timeRangeFilter
    spacing: Theme.spacingSmall
    
    // 对外暴露的属性
    property string selectedRange: "1h"  // 1h, 2h, 5h, custom
    property int customHours: 1  // 自定义小时数
    property string customStartTime: ""  // 自定义开始时间
    property string customEndTime: ""    // 自定义结束时间
    
    // 信号
    signal rangeChanged(string range, int hours)
    signal customRangeChanged(string startTime, string endTime)
    
    Text {
        text: "时间范围:"
        font.pixelSize: Theme.fontSizeMedium
        font.family: Theme.fontFamilyDefault
        color: Theme.textSecondary
    }
    
    Button {
        id: oneHourButton
        text: "1小时"
        background: Rectangle {
            color: timeRangeFilter.selectedRange === "1h" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: timeRangeFilter.selectedRange === "1h" ? Theme.textPrimary : Theme.textSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        implicitWidth: 68
        implicitHeight: 32
        onClicked: {
            timeRangeFilter.selectedRange = "1h"
            timeRangeFilter.rangeChanged("1h", 1)
        }
    }
    
    Button {
        id: twoHoursButton
        text: "2小时"
        background: Rectangle {
            color: timeRangeFilter.selectedRange === "2h" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: timeRangeFilter.selectedRange === "2h" ? Theme.textPrimary : Theme.textSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        implicitWidth: 68
        implicitHeight: 32
        onClicked: {
            timeRangeFilter.selectedRange = "2h"
            timeRangeFilter.rangeChanged("2h", 2)
        }
    }
    
    Button {
        id: fiveHoursButton
        text: "5小时"
        background: Rectangle {
            color: timeRangeFilter.selectedRange === "5h" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: timeRangeFilter.selectedRange === "5h" ? Theme.textPrimary : Theme.textSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        implicitWidth: 68
        implicitHeight: 32
        onClicked: {
            timeRangeFilter.selectedRange = "5h"
            timeRangeFilter.rangeChanged("5h", 5)
        }
    }
    
    Button {
        id: customButton
        text: "自定义"
        background: Rectangle {
            color: timeRangeFilter.selectedRange === "custom" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: timeRangeFilter.selectedRange === "custom" ? Theme.textPrimary : Theme.textSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        implicitWidth: 68
        implicitHeight: 32
        onClicked: {
            customTimeDialog.open()
        }
    }
    
    // 自定义时间范围对话框
    CustomTimeRangeDialog {
        id: customTimeDialog
        parent: Overlay.overlay
        
        startTime: timeRangeFilter.customStartTime
        endTime: timeRangeFilter.customEndTime
        
        onTimeRangeApplied: function(startTime, endTime) {
            timeRangeFilter.selectedRange = "custom"
            timeRangeFilter.customStartTime = startTime
            timeRangeFilter.customEndTime = endTime
            timeRangeFilter.customRangeChanged(startTime, endTime)
        }
        
        onTimeRangeReset: {
            timeRangeFilter.selectedRange = "1h"
            timeRangeFilter.customStartTime = ""
            timeRangeFilter.customEndTime = ""
            timeRangeFilter.rangeChanged("1h", 1)
        }
    }
}
