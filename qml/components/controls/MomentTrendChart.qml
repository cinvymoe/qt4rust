// MomentTrendChart.qml - 力矩百分比趋势图组件
import qt.rust.demo
import QtQuick
import "../../styles"

Rectangle {
    id: root
    
    Tr { id: tr }
    
    // 公开属性
    property var timeLabels: []
    property var momentData: []
    
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
                text: tr.t("chart.momentTrend")
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
                    if (momentData.length === 0) return
                    
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
                        var yVal = padding + chartHeight * (1 - value)
                        if (j === 0) {
                            ctx.lineTo(x, yVal)
                        } else {
                            ctx.lineTo(x, yVal)
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
                        yVal = padding + chartHeight * (1 - value)
                        if (j === 0) {
                            ctx.moveTo(x, yVal)
                        } else {
                            ctx.lineTo(x, yVal)
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
                
                Connections {
                    target: root
                    function onMomentDataChanged() { momentCanvas.requestPaint() }
                    function onTimeLabelsChanged() { momentCanvas.requestPaint() }
                }
            }
            
            // 预警线标签
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                y: parent.height * 0.25
                text: tr.t("moment.warning") + " 75%"
                font.pixelSize: Theme.fontSizeTiny
                font.family: Theme.fontFamilyDefault
                color: Theme.warningColor
            }
            
            // 危险线标签
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                y: parent.height * 0.1
                text: tr.t("moment.danger") + " 90%"
                font.pixelSize: Theme.fontSizeTiny
                font.family: Theme.fontFamilyDefault
                color: Theme.dangerColor
            }
        }
    }
}