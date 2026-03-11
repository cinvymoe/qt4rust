// LoadCurveChart.qml - 额定载荷曲线图表组件（使用 Canvas 绘制）
import QtQuick
import "../../styles"

Item {
    id: chartContainer
    width: parent.width
    height: 384
    
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        radius: Theme.radiusMedium
        
        Canvas {
            id: canvas
            anchors.fill: parent
            anchors.margins: Theme.spacingLarge
            
            // 曲线数据
            property var mainBoomData: [
                {x: 3, y: 25}, {x: 5, y: 22}, {x: 8, y: 18}, {x: 12, y: 14},
                {x: 15, y: 11}, {x: 18, y: 9}, {x: 22, y: 6}, {x: 25, y: 4}
            ]
            
            property var mainAuxData: [
                {x: 3, y: 20}, {x: 6, y: 17}, {x: 10, y: 14}, {x: 14, y: 11},
                {x: 18, y: 8}, {x: 22, y: 6}, {x: 25, y: 4}
            ]
            
            property var maxLengthData: [
                {x: 5, y: 15}, {x: 8, y: 13}, {x: 12, y: 10}, {x: 16, y: 7},
                {x: 20, y: 5}, {x: 25, y: 3}
            ]
            
            onPaint: {
                var ctx = getContext("2d")
                ctx.clearRect(0, 0, width, height)
                
                var padding = 40
                var chartWidth = width - padding * 2
                var chartHeight = height - padding * 2
                var maxX = 25
                var maxY = 28
                
                // 绘制网格线
                ctx.strokeStyle = "#314158"
                ctx.lineWidth = 1
                for (var i = 0; i <= 7; i++) {
                    var y = padding + (chartHeight / 7) * i
                    ctx.beginPath()
                    ctx.moveTo(padding, y)
                    ctx.lineTo(padding + chartWidth, y)
                    ctx.stroke()
                }
                
                // 绘制曲线函数
                function drawCurve(data, color) {
                    // 绘制折线
                    ctx.strokeStyle = color
                    ctx.lineWidth = 2
                    ctx.beginPath()
                    
                    for (var k = 0; k < data.length; k++) {
                        var px = padding + (data[k].x / maxX) * chartWidth
                        var py = padding + chartHeight - (data[k].y / maxY) * chartHeight
                        
                        if (k === 0) {
                            ctx.moveTo(px, py)
                        } else {
                            ctx.lineTo(px, py)
                        }
                    }
                    ctx.stroke()
                    
                    // 绘制数据点
                    ctx.fillStyle = color
                    for (k = 0; k < data.length; k++) {
                        px = padding + (data[k].x / maxX) * chartWidth
                        py = padding + chartHeight - (data[k].y / maxY) * chartHeight
                        ctx.beginPath()
                        ctx.arc(px, py, 3, 0, 2 * Math.PI)
                        ctx.fill()
                    }
                }
                
                // 绘制三条曲线
                drawCurve(mainBoomData, "#22c55e")
                drawCurve(mainAuxData, "#3b82f6")
                drawCurve(maxLengthData, "#f59e0b")
                
                // Y轴标签
                ctx.fillStyle = "#94a3b8"
                ctx.font = "12px " + Theme.fontFamilyDefault
                ctx.textAlign = "right"
                for (i = 0; i <= 7; i++) {
                    y = padding + (chartHeight / 7) * i
                    var yLabel = (maxY - i * 4).toString()
                    ctx.fillText(yLabel, padding - 10, y + 4)
                }
                
                // X轴标签
                ctx.textAlign = "center"
                ctx.fillStyle = "#94a3b8"
                for (var j = 0; j <= 5; j++) {
                    var x = padding + (chartWidth / 5) * j
                    var xLabel = (j * 5).toString()
                    ctx.fillText(xLabel, x, padding + chartHeight + 20)
                }
            }
            
            Component.onCompleted: {
                requestPaint()
            }
            
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
        }
    }
}

