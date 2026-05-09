// ChartView.qml - 数据曲线分析视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo
import "../styles"
import "../components/controls"

Item {
    id: chartView
    
    
    // 动态数据属性（从 ViewModel 加载）
    property var timeLabels: []
    property var momentData: []
    property var loadData: []
    property var radiusData: []
    property var angleData: []
    
    // HistoryViewModel 实例
    HistoryViewModel {
        id: historyViewModel
        
        Component.onCompleted: {
            historyViewModel.init_with_repository()
        }
    }
    
    // 监听 ViewModel 属性变化
    Connections {
        target: historyViewModel
        function onChart_data_jsonChanged() { parseChartData() }
    }
    
    // 解析图表数据 JSON
    function parseChartData() {
        try {
            var data = JSON.parse(historyViewModel.chart_data_json)
            chartView.timeLabels = data.timeLabels || []
            chartView.momentData = data.momentData || []
            chartView.loadData = data.loadData || []
            chartView.radiusData = data.radiusData || []
            chartView.angleData = data.angleData || []
        } catch (e) {
            console.log("解析图表数据失败:", e)
        }
    }
    
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
                        text: TranslationBridge.translate("chart.title")
                        font.pixelSize: Theme.fontSizeXLarge
                        font.family: Theme.fontFamilyDefault
                        color: Theme.textPrimary
                    }
                    
                    Text {
                        text: TranslationBridge.translate("chart.subtitle")
                        font.pixelSize: Theme.fontSizeSmall
                        font.family: Theme.fontFamilyDefault
                        color: Theme.textTertiary
                    }
                }
                
                Item { Layout.fillWidth: true }
                
                // 右侧刷新按钮和时间范围筛选
                RowLayout {
                    spacing: Theme.spacingMedium
                    
                    // 刷新按钮
                    Button {
                        text: TranslationBridge.translate("chart.refresh")
                        implicitWidth: 60
                        implicitHeight: 32
                        
                        background: Rectangle {
                            color: parent.down ? Theme.darkAccent : Theme.darkSurface
                            radius: Theme.radiusMedium
                            border.color: Theme.darkBorder
                            border.width: Theme.borderNormal
                        }
                        
                        contentItem: Text {
                            text: parent.text
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                        
                        onClicked: {
                            historyViewModel.refresh_chart_data()
                        }
                    }
                    
                    // 时间范围筛选
                    TimeRangeFilter {
                        id: timeRangeFilter
                        selectedRange: "1h"
                        
                        onRangeChanged: function(range, hours) {
                            // 根据时间范围设置筛选
                            var filter = "all"
                            if (hours <= 1) filter = "today"
                            else if (hours <= 24) filter = "today"
                            else if (hours <= 168) filter = "week"
                            else filter = "month"
                            
                            historyViewModel.set_filter(filter)
                            historyViewModel.refresh_chart_data()
                        }
                        
                        onCustomRangeChanged: function(startTime, endTime) {
                            // 解析时间字符串并转换为时间戳
                            // 格式: "YYYY/MM/DD HH:MM"
                            var startDate = new Date(startTime.replace(/\//g, "-"))
                            var endDate = new Date(endTime.replace(/\//g, "-"))
                            var startTimestamp = Math.floor(startDate.getTime() / 1000)
                            var endTimestamp = Math.floor(endDate.getTime() / 1000)
                            
                            historyViewModel.set_custom_time_range(startTimestamp, endTimestamp)
                            historyViewModel.refresh_chart_data()
                        }
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
