// LoadCurveChart.qml - 额定载荷曲线图表组件（使用 Canvas 绘制）
// 支持动态数据显示，横坐标为幅度，纵坐标为载荷重量
import QtQuick
import "../../styles"

Item {
    id: chartContainer
    width: parent.width
    height: 384
    
    // 属性：当前要显示的数据
    property var curveData: []
    property string curveColor: "#22c55e"
    property string curveName: ""
    property real boomLength: 0.0
    
    // 坐标轴范围
    property real maxX: 25  // 最大幅度（米）
    property real maxY: 60  // 最大载荷（吨）
    
    // 强制重绘
    function requestRepaint() {
        canvas.requestPaint()
    }
    
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        radius: Theme.radiusMedium
        
        Canvas {
            id: canvas
            anchors.fill: parent
            anchors.margins: Theme.spacingLarge
            
            onPaint: {
                var ctx = getContext("2d")
                ctx.clearRect(0, 0, width, height)
                
                var padding = 50
                var chartWidth = width - padding * 2
                var chartHeight = height - padding * 2
                
                // 如果没有数据，显示提示
                if (!curveData || curveData.length === 0) {
                    ctx.fillStyle = "#94a3b8"
                    ctx.font = "14px " + Theme.fontFamilyDefault
                    ctx.textAlign = "center"
                    ctx.fillText("暂无数据", width / 2, height / 2)
                    return
                }
                
                // 绘制网格线 - 水平线（载荷）
                ctx.strokeStyle = "#314158"
                ctx.lineWidth = 1
                var ySteps = 6
                for (var i = 0; i <= ySteps; i++) {
                    var y = padding + (chartHeight / ySteps) * i
                    ctx.beginPath()
                    ctx.moveTo(padding, y)
                    ctx.lineTo(padding + chartWidth, y)
                    ctx.stroke()
                }
                
                // 绘制网格线 - 垂直线（幅度）
                var xSteps = 5
                for (var j = 0; j <= xSteps; j++) {
                    var x = padding + (chartWidth / xSteps) * j
                    ctx.beginPath()
                    ctx.moveTo(x, padding)
                    ctx.lineTo(x, padding + chartHeight)
                    ctx.stroke()
                }
                
                // 绘制坐标轴
                ctx.strokeStyle = "#64748b"
                ctx.lineWidth = 2
                // Y轴
                ctx.beginPath()
                ctx.moveTo(padding, padding)
                ctx.lineTo(padding, padding + chartHeight)
                ctx.stroke()
                // X轴
                ctx.beginPath()
                ctx.moveTo(padding, padding + chartHeight)
                ctx.lineTo(padding + chartWidth, padding + chartHeight)
                ctx.stroke()
                
                // 绘制曲线
                ctx.strokeStyle = curveColor
                ctx.lineWidth = 3
                ctx.beginPath()
                
                for (var k = 0; k < curveData.length; k++) {
                    var px = padding + (curveData[k].x / maxX) * chartWidth
                    var py = padding + chartHeight - (curveData[k].y / maxY) * chartHeight
                    
                    if (k === 0) {
                        ctx.moveTo(px, py)
                    } else {
                        ctx.lineTo(px, py)
                    }
                }
                ctx.stroke()
                
                // 绘制数据点
                ctx.fillStyle = curveColor
                for (k = 0; k < curveData.length; k++) {
                    px = padding + (curveData[k].x / maxX) * chartWidth
                    py = padding + chartHeight - (curveData[k].y / maxY) * chartHeight
                    ctx.beginPath()
                    ctx.arc(px, py, 5, 0, 2 * Math.PI)
                    ctx.fill()
                    
                    // 数据点外圈
                    ctx.strokeStyle = "#ffffff"
                    ctx.lineWidth = 2
                    ctx.stroke()
                }
                
                // Y轴标签（载荷）
                ctx.fillStyle = "#94a3b8"
                ctx.font = "12px " + Theme.fontFamilyDefault
                ctx.textAlign = "right"
                ctx.textBaseline = "middle"
                for (i = 0; i <= ySteps; i++) {
                    y = padding + (chartHeight / ySteps) * i
                    var yLabel = (maxY - i * (maxY / ySteps)).toFixed(0)
                    ctx.fillText(yLabel + "t", padding - 10, y)
                }
                
                // X轴标签（幅度）
                ctx.textAlign = "center"
                ctx.textBaseline = "top"
                ctx.fillStyle = "#94a3b8"
                for (j = 0; j <= xSteps; j++) {
                    x = padding + (chartWidth / xSteps) * j
                    var xLabel = (j * (maxX / xSteps)).toFixed(0)
                    ctx.fillText(xLabel + "m", x, padding + chartHeight + 10)
                }
                
                // 轴标题
                ctx.save()
                ctx.translate(15, height / 2)
                ctx.rotate(-Math.PI / 2)
                ctx.textAlign = "center"
                ctx.fillStyle = "#cbd5e1"
                ctx.font = "bold 12px " + Theme.fontFamilyDefault
                ctx.fillText("载荷 (吨)", 0, 0)
                ctx.restore()
                
                ctx.textAlign = "center"
                ctx.fillStyle = "#cbd5e1"
                ctx.font = "bold 12px " + Theme.fontFamilyDefault
                ctx.fillText("幅度 (米)", width / 2, height - 15)
            }
            
            Component.onCompleted: {
                requestPaint()
            }
            
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
        }
    }
}
