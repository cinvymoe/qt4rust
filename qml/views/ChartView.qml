// ChartView.qml - 数据曲线分析视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../styles"
import "../components/controls"

Item {
    id: chartView
    
    // 模拟时间序列数据
    property var timeLabels: ["14:33:49", "14:33:52", "14:33:55", "14:33:59", "14:34:02", 
                               "14:34:05", "14:34:08", "14:34:11", "14:34:14", "14:34:17",
                               "14:34:20", "14:34:23", "14:34:25", "14:34:28", "14:34:32"]
    
    // 力矩百分比数据 (0-100)
    property var momentData: [85, 88, 92, 95, 94, 93, 91, 89, 87, 90, 92, 94, 96, 95, 93]
    
    // 载荷数据 (吨)
    property var loadData: [22, 28, 27, 18, 20, 21, 21, 25, 24, 28, 26, 19, 20, 21, 22]
    
    // 工作半径数据 (米)
    property var radiusData: [8, 10, 12, 15, 18, 20, 22, 25, 28, 30, 32, 35, 38, 40, 42]
    
    // 吊臂角度数据 (度)
    property var angleData: [75, 72, 70, 68, 65, 63, 60, 58, 55, 52, 50, 48, 45, 43, 40]
    
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        
        // 顶部标题区域
        Rectangle {
            id: headerSection
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 93
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderNormal
            
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.spacingLarge
                anchors.rightMargin: Theme.spacingLarge
                spacing: Theme.spacingLarge
                
                // 左侧标题区域
                ColumnLayout {
                    spacing: Theme.spacingSmall
                    
                    Text {
                        text: "数据曲线分析"
                        font.pixelSize: Theme.fontSizeXLarge
                        font.family: Theme.fontFamilyDefault
                        color: Theme.textPrimary
                    }
                    
                    Text {
                        text: "实时监测数据变化趋势"
                        font.pixelSize: Theme.fontSizeSmall
                        font.family: Theme.fontFamilyDefault
                        color: Theme.textTertiary
                    }
                }
                
                Item { Layout.fillWidth: true }
                
                // 右侧时间范围筛选区域
                TimeRangeFilter {
                    id: timeRangeFilter
                    selectedRange: "1h"
                    
                    onRangeChanged: function(range, hours) {
                        console.log("时间范围已更改:", range, "小时数:", hours)
                        // TODO: 根据选择的时间范围更新图表数据
                        // 可以在这里调用 Rust 后端获取对应时间范围的数据
                    }
                }
            }
        }
        
        // 可滚动内容区域
        Flickable {
            id: scrollArea
            anchors.top: headerSection.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.spacingMedium
            anchors.rightMargin: Theme.spacingMedium + 10  // 为滚动条留出空间
            contentHeight: contentColumn.height
            clip: true
            
            Column {
                id: contentColumn
                width: parent.width
                spacing: Theme.spacingMedium
                
                // 1. 力矩百分比趋势图
                MomentTrendChart {
                    width: parent.width
                    height: 397
                    timeLabels: chartView.timeLabels
                    momentData: chartView.momentData
                }
                
                // 2. 载荷变化曲线
                LoadTrendChart {
                    width: parent.width
                    height: 333
                    timeLabels: chartView.timeLabels
                    loadData: chartView.loadData
                    maxValue: 28
                }
                
                // 3. 多参数对比分析
                MultiParamChart {
                    width: parent.width
                    height: 433
                    timeLabels: chartView.timeLabels
                    radiusData: chartView.radiusData
                    angleData: chartView.angleData
                    maxValue: 80
                }
            }
        }
        
        // 外部滚动条指示器
        ScrollBar {
            id: scrollBar
            anchors.right: parent.right
            anchors.top: scrollArea.top
            anchors.bottom: scrollArea.bottom
            anchors.rightMargin: Theme.spacingSmall
            
            orientation: Qt.Vertical
            size: scrollArea.height / scrollArea.contentHeight
            position: scrollArea.contentY / scrollArea.contentHeight
            active: scrollArea.moving || hovered || pressed
            policy: ScrollBar.AsNeeded
            
            onPositionChanged: {
                if (pressed) {
                    scrollArea.contentY = position * scrollArea.contentHeight
                }
            }
            
            contentItem: Rectangle {
                implicitWidth: 6
                radius: 3
                color: scrollBar.pressed ? Theme.textSecondary : Theme.textTertiary
                opacity: scrollBar.active ? 0.75 : 0.5
                
                Behavior on opacity {
                    NumberAnimation { duration: Theme.animationDuration }
                }
            }
        }
    }
}
