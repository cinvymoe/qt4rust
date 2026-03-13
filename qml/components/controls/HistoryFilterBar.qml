// HistoryFilterBar.qml - 历史记录筛选栏组件
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"

RowLayout {
    id: historyFilterBar
    spacing: Theme.spacingSmall
    
    // 对外暴露的属性
    property string selectedFilter: "all"  // all, today, week, month
    
    // 信号
    signal filterChanged(string filter)
    
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
}
