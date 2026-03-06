// ChartView.qml - 数据曲线分析视图（使用 Canvas 绘制）
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../styles"

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
            height: 92
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            
            Column {
                anchors.centerIn: parent
                spacing: Theme.spacingSmall
                
                Text {
                    text: "数据曲线分析"
                    font.pixelSize: Theme.fontSizeXLarge
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textPrimary
                    anchors.horizontalCenter: parent.horizontalCenter
                }
                
                Text {
                    text: "实时监测数据变化趋势"
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textTertiary
                    anchors.horizontalCenter: parent.horizontalCenter
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
                Rectangle {
                    width: parent.width
                    height: 397
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMedium
                        spacing: Theme.spacingMedium
                        
                        // 标题
                        Row {
                            spacing: Theme.spacingMedium
                            
                            Rectangle {
                                width: 4
                                height: Theme.fontSizeLarge
                                color: Theme.darkAccent
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "力矩百分比趋势"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyDefault
                                font.weight: Font.Medium
                                color: Theme.textPrimary
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 图表容器
                        Rectangle {
                            width: parent.width
                            height: parent.height - Theme.fontSizeLarge - Theme.spacingMedium * 2
                            color: Theme.darkBackground
                            radius: Theme.radiusMedium
                            
                            Canvas {
                                id: momentCanvas
                                anchors.fill: parent
                                anchors.margins: Theme.spacingLarge
                                
                                onPaint: {
                                    var ctx = getContext("2d")
                                    ctx.clearRect(0, 0, width, height)
                                    
                                    var padding = 40
                                    var chartWidth = width - padding * 2
                                    var chartHeight = height - padding * 2
                                    var dataCount = momentData.length
                                    
                                    // 绘制网格线
                                    ctx.strokeStyle = Theme.darkBorder
                                    ctx.lineWidth = 1
                                    for (var i = 0; i <= 4; i++) {
                                        var y = padding + (chartHeight / 4) * i
                                        ctx.beginPath()
                                        ctx.moveTo(padding, y)
                                        ctx.lineTo(padding + chartWidth, y)
                                        ctx.stroke()
                                    }
                                    
                                    // 绘制预警线 75%
                                    ctx.strokeStyle = Theme.warningColor
                                    ctx.lineWidth = 2
                                    ctx.setLineDash([5, 5])
                                    var warningY = padding + chartHeight * (1 - 0.75)
                                    ctx.beginPath()
                                    ctx.moveTo(padding, warningY)
                                    ctx.lineTo(padding + chartWidth, warningY)
                                    ctx.stroke()
                                    
                                    // 绘制危险线 90%
                                    ctx.strokeStyle = Theme.dangerColor
                                    var dangerY = padding + chartHeight * (1 - 0.90)
                                    ctx.beginPath()
                                    ctx.moveTo(padding, dangerY)
                                    ctx.lineTo(padding + chartWidth, dangerY)
                                    ctx.stroke()
                                    ctx.setLineDash([])
                                    
                                    // 绘制面积图
                                    ctx.fillStyle = Theme.darkAccent
                                    ctx.globalAlpha = 0.3
                                    ctx.beginPath()
                                    ctx.moveTo(padding, padding + chartHeight)
                                    
                                    for (var j = 0; j < dataCount; j++) {
                                        var x = padding + (chartWidth / (dataCount - 1)) * j
                                        var value = momentData[j] / 100
                                        var y = padding + chartHeight * (1 - value)
                                        if (j === 0) {
                                            ctx.lineTo(x, y)
                                        } else {
                                            ctx.lineTo(x, y)
                                        }
                                    }
                                    
                                    ctx.lineTo(padding + chartWidth, padding + chartHeight)
                                    ctx.closePath()
                                    ctx.fill()
                                    
                                    // 绘制折线
                                    ctx.globalAlpha = 1.0
                                    ctx.strokeStyle = Theme.darkAccent
                                    ctx.lineWidth = 2
                                    ctx.beginPath()
                                    
                                    for (j = 0; j < dataCount; j++) {
                                        x = padding + (chartWidth / (dataCount - 1)) * j
                                        value = momentData[j] / 100
                                        y = padding + chartHeight * (1 - value)
                                        if (j === 0) {
                                            ctx.moveTo(x, y)
                                        } else {
                                            ctx.lineTo(x, y)
                                        }
                                    }
                                    ctx.stroke()
                                    
                                    // Y轴标签
                                    ctx.fillStyle = Theme.textTertiary
                                    ctx.font = "12px " + Theme.fontFamilyDefault
                                    ctx.textAlign = "right"
                                    for (i = 0; i <= 4; i++) {
                                        y = padding + (chartHeight / 4) * i
                                        var label = (100 - i * 25).toString()
                                        ctx.fillText(label, padding - 10, y + 4)
                                    }
                                    
                                    // X轴时间标签
                                    ctx.textAlign = "center"
                                    ctx.fillStyle = Theme.textTertiary
                                    var labelStep = Math.floor(dataCount / 5)
                                    for (i = 0; i < dataCount; i += labelStep) {
                                        x = padding + (chartWidth / (dataCount - 1)) * i
                                        ctx.fillText(timeLabels[i], x, padding + chartHeight + 20)
                                    }
                                }
                                
                                Component.onCompleted: requestPaint()
                            }
                            
                            // 标签
                            Text {
                                anchors.horizontalCenter: parent.horizontalCenter
                                y: parent.height * 0.25
                                text: "预警线 75%"
                                font.pixelSize: Theme.fontSizeTiny
                                font.family: Theme.fontFamilyDefault
                                color: Theme.warningColor
                            }
                            
                            Text {
                                anchors.horizontalCenter: parent.horizontalCenter
                                y: parent.height * 0.1
                                text: "危险线 90%"
                                font.pixelSize: Theme.fontSizeTiny
                                font.family: Theme.fontFamilyDefault
                                color: Theme.dangerColor
                            }
                        }
                    }
                }
                
                // 2. 载荷变化曲线
                Rectangle {
                    width: parent.width
                    height: 333
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMedium
                        spacing: Theme.spacingMedium
                        
                        // 标题
                        Row {
                            spacing: Theme.spacingMedium
                            
                            Rectangle {
                                width: 4
                                height: Theme.fontSizeLarge
                                color: Theme.successColor
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "载荷变化曲线"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyDefault
                                font.weight: Font.Medium
                                color: Theme.textPrimary
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 图表容器
                        Rectangle {
                            width: parent.width
                            height: parent.height - Theme.fontSizeLarge - Theme.spacingMedium * 2
                            color: Theme.darkBackground
                            radius: Theme.radiusMedium
                            
                            Canvas {
                                id: loadCanvas
                                anchors.fill: parent
                                anchors.margins: Theme.spacingLarge
                                
                                onPaint: {
                                    var ctx = getContext("2d")
                                    ctx.clearRect(0, 0, width, height)
                                    
                                    var padding = 40
                                    var chartWidth = width - padding * 2
                                    var chartHeight = height - padding * 2
                                    var dataCount = loadData.length
                                    var maxValue = 28
                                    
                                    // 绘制网格线
                                    ctx.strokeStyle = Theme.darkBorder
                                    ctx.lineWidth = 1
                                    for (var i = 0; i <= 4; i++) {
                                        var y = padding + (chartHeight / 4) * i
                                        ctx.beginPath()
                                        ctx.moveTo(padding, y)
                                        ctx.lineTo(padding + chartWidth, y)
                                        ctx.stroke()
                                    }
                                    
                                    // 绘制折线
                                    ctx.strokeStyle = Theme.successColor
                                    ctx.lineWidth = 2
                                    ctx.beginPath()
                                    
                                    for (var j = 0; j < dataCount; j++) {
                                        var x = padding + (chartWidth / (dataCount - 1)) * j
                                        var value = loadData[j] / maxValue
                                        y = padding + chartHeight * (1 - value)
                                        if (j === 0) {
                                            ctx.moveTo(x, y)
                                        } else {
                                            ctx.lineTo(x, y)
                                        }
                                    }
                                    ctx.stroke()
                                    
                                    // 绘制数据点
                                    ctx.fillStyle = Theme.successColor
                                    for (j = 0; j < dataCount; j++) {
                                        x = padding + (chartWidth / (dataCount - 1)) * j
                                        value = loadData[j] / maxValue
                                        y = padding + chartHeight * (1 - value)
                                        ctx.beginPath()
                                        ctx.arc(x, y, 3, 0, 2 * Math.PI)
                                        ctx.fill()
                                    }
                                    
                                    // Y轴标签
                                    ctx.fillStyle = Theme.textTertiary
                                    ctx.font = "12px " + Theme.fontFamilyDefault
                                    ctx.textAlign = "right"
                                    for (i = 0; i <= 4; i++) {
                                        y = padding + (chartHeight / 4) * i
                                        var label = (maxValue - i * 7).toString()
                                        ctx.fillText(label, padding - 10, y + 4)
                                    }
                                    
                                    // X轴时间标签
                                    ctx.textAlign = "center"
                                    ctx.fillStyle = Theme.textTertiary
                                    var labelStep = Math.floor(dataCount / 5)
                                    for (i = 0; i < dataCount; i += labelStep) {
                                        x = padding + (chartWidth / (dataCount - 1)) * i
                                        ctx.fillText(timeLabels[i], x, padding + chartHeight + 20)
                                    }
                                }
                                
                                Component.onCompleted: requestPaint()
                            }
                        }
                    }
                }
                
                // 3. 多参数对比分析
                Rectangle {
                    width: parent.width
                    height: 433
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMedium
                        spacing: Theme.spacingMedium
                        
                        // 标题
                        Row {
                            spacing: Theme.spacingMedium
                            
                            Rectangle {
                                width: 4
                                height: Theme.fontSizeLarge
                                color: "#ad46ff"
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "多参数对比分析"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyDefault
                                font.weight: Font.Medium
                                color: Theme.textPrimary
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 图表容器
                        Rectangle {
                            width: parent.width
                            height: 320
                            color: Theme.darkBackground
                            radius: Theme.radiusMedium
                            
                            Canvas {
                                id: multiCanvas
                                anchors.fill: parent
                                anchors.margins: Theme.spacingLarge
                                
                                onPaint: {
                                    var ctx = getContext("2d")
                                    ctx.clearRect(0, 0, width, height)
                                    
                                    var padding = 40
                                    var chartWidth = width - padding * 2
                                    var chartHeight = height - padding * 2
                                    var dataCount = radiusData.length
                                    var maxValue = 80
                                    
                                    // 绘制网格线
                                    ctx.strokeStyle = Theme.darkBorder
                                    ctx.lineWidth = 1
                                    for (var i = 0; i <= 4; i++) {
                                        var y = padding + (chartHeight / 4) * i
                                        ctx.beginPath()
                                        ctx.moveTo(padding, y)
                                        ctx.lineTo(padding + chartWidth, y)
                                        ctx.stroke()
                                    }
                                    
                                    // 绘制工作半径曲线
                                    ctx.strokeStyle = "#00b8db"
                                    ctx.lineWidth = 2
                                    ctx.beginPath()
                                    
                                    for (var j = 0; j < dataCount; j++) {
                                        var x = padding + (chartWidth / (dataCount - 1)) * j
                                        var value = radiusData[j] / maxValue
                                        y = padding + chartHeight * (1 - value)
                                        if (j === 0) {
                                            ctx.moveTo(x, y)
                                        } else {
                                            ctx.lineTo(x, y)
                                        }
                                    }
                                    ctx.stroke()
                                    
                                    // 绘制吊臂角度曲线
                                    ctx.strokeStyle = "#ff6900"
                                    ctx.beginPath()
                                    
                                    for (j = 0; j < dataCount; j++) {
                                        x = padding + (chartWidth / (dataCount - 1)) * j
                                        value = angleData[j] / maxValue
                                        y = padding + chartHeight * (1 - value)
                                        if (j === 0) {
                                            ctx.moveTo(x, y)
                                        } else {
                                            ctx.lineTo(x, y)
                                        }
                                    }
                                    ctx.stroke()
                                    
                                    // Y轴标签
                                    ctx.fillStyle = Theme.textTertiary
                                    ctx.font = "12px " + Theme.fontFamilyDefault
                                    ctx.textAlign = "right"
                                    for (i = 0; i <= 4; i++) {
                                        y = padding + (chartHeight / 4) * i
                                        var label = (maxValue - i * 20).toString()
                                        ctx.fillText(label, padding - 10, y + 4)
                                    }
                                    
                                    // X轴时间标签
                                    ctx.textAlign = "center"
                                    ctx.fillStyle = Theme.textTertiary
                                    var labelStep = Math.floor(dataCount / 5)
                                    for (i = 0; i < dataCount; i += labelStep) {
                                        x = padding + (chartWidth / (dataCount - 1)) * i
                                        ctx.fillText(timeLabels[i], x, padding + chartHeight + 20)
                                    }
                                }
                                
                                Component.onCompleted: requestPaint()
                            }
                        }
                        
                        // 图例
                        Row {
                            anchors.horizontalCenter: parent.horizontalCenter
                            spacing: Theme.spacingLarge
                            
                            // 工作半径图例
                            Row {
                                spacing: Theme.spacingSmall
                                
                                Rectangle {
                                    width: 24
                                    height: 2
                                    color: "#00b8db"
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                                
                                Text {
                                    text: "工作半径 (m)"
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.family: Theme.fontFamilyDefault
                                    color: Theme.textSecondary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                            
                            // 吊臂角度图例
                            Row {
                                spacing: Theme.spacingSmall
                                
                                Rectangle {
                                    width: 24
                                    height: 2
                                    color: "#ff6900"
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                                
                                Text {
                                    text: "吊臂角度 (°)"
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.family: Theme.fontFamilyDefault
                                    color: Theme.textSecondary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                        }
                    }
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
