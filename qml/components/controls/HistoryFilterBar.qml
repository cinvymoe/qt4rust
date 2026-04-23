// HistoryFilterBar.qml - 历史记录筛选栏组件
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"
import "../dialogs"

RowLayout {
    id: historyFilterBar
    spacing: Theme.spacingSmall
    
    // 对外暴露的属性
    property string selectedFilter: "all"  // all, today, week, month, custom
    
    // 信号
    signal filterChanged(string filter)
    signal customTimeRangeChanged(int startTimestamp, int endTimestamp)
    
    Text {
        text: "历史记录:"
        font.pixelSize: Theme.fontSizeMedium
        font.family: Theme.fontFamilyDefault
        color: Theme.textSecondary
    }
    
    Button {
        id: allButton
        text: "全部"
        background: Rectangle {
            color: historyFilterBar.selectedFilter === "all" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: historyFilterBar.selectedFilter === "all" ? Theme.textPrimary : Theme.textSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        implicitWidth: 56
        implicitHeight: 32
        onClicked: {
            historyFilterBar.selectedFilter = "all"
            historyFilterBar.filterChanged("all")
        }
    }
    
    Button {
        id: todayButton
        text: "今天"
        background: Rectangle {
            color: historyFilterBar.selectedFilter === "today" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: historyFilterBar.selectedFilter === "today" ? Theme.textPrimary : Theme.textSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        implicitWidth: 56
        implicitHeight: 32
        onClicked: {
            historyFilterBar.selectedFilter = "today"
            historyFilterBar.filterChanged("today")
        }
    }
    
    Button {
        id: weekButton
        text: "最近7天"
        background: Rectangle {
            color: historyFilterBar.selectedFilter === "week" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: historyFilterBar.selectedFilter === "week" ? Theme.textPrimary : Theme.textSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        implicitWidth: 80
        implicitHeight: 32
        onClicked: {
            historyFilterBar.selectedFilter = "week"
            historyFilterBar.filterChanged("week")
        }
    }
    
    Button {
        id: monthButton
        text: "最近30天"
        background: Rectangle {
            color: historyFilterBar.selectedFilter === "month" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: historyFilterBar.selectedFilter === "month" ? Theme.textPrimary : Theme.textSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        implicitWidth: 88
        implicitHeight: 32
        onClicked: {
            historyFilterBar.selectedFilter = "month"
            historyFilterBar.filterChanged("month")
        }
    }
    
    Button {
        id: customButton
        text: "自定义"
        background: Rectangle {
            color: historyFilterBar.selectedFilter === "custom" ? Theme.darkAccent : Theme.darkBorder
            radius: Theme.radiusMedium
        }
        contentItem: Text {
            text: parent.text
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: historyFilterBar.selectedFilter === "custom" ? Theme.textPrimary : Theme.textSecondary
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
        
        onTimeRangeApplied: function(startTime, endTime) {
            // 解析时间字符串并转换为时间戳
            // 格式: "YYYY/MM/DD HH:MM"
            var startDate = new Date(startTime.replace(/\//g, "-"))
            var endDate = new Date(endTime.replace(/\//g, "-"))
            var startTimestamp = Math.floor(startDate.getTime() / 1000)
            var endTimestamp = Math.floor(endDate.getTime() / 1000)
            
            historyFilterBar.selectedFilter = "custom"
            historyFilterBar.customTimeRangeChanged(startTimestamp, endTimestamp)
        }
        
        onTimeRangeReset: {
            historyFilterBar.selectedFilter = "all"
            historyFilterBar.filterChanged("all")
        }
    }
}
