// LoadCurveChart.qml - 额定载荷曲线图表组件（使用 Canvas 绘制）
import qt.rust.demo
import QtQuick
import "../../styles"

Item {
    id: chartContainer
    width: parent.width
    height: 384

    TranslationBridge { id: tr }

    // Translated strings for canvas (canvas doesn't have direct QML access)
    property string loadingText: tr.translate("common.loading") || "正在加载数据..."
    property string noDataText: tr.translate("common.noData") || "无数据"

    property var viewModel: null
    
    function updateChart() {
        if (viewModel && viewModel.dataLoaded) {
            canvas.requestPaint()
        }
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
                
                console.log("LoadCurveChart onPaint called")
                console.log("viewModel:", viewModel)
                console.log("viewModel.data_loaded:", viewModel ? viewModel.data_loaded : "null")
                
                if (!viewModel || !viewModel.data_loaded) {
                    // 显示加载提示
                    ctx.fillStyle = "#94a3b8"
                    ctx.font = "16px " + Theme.fontFamilyDefault
                    ctx.textAlign = "center"
                    ctx.fillText("正在加载数据...", width / 2, height / 2)
                    return
                }
                
                // 获取当前臂长的曲线数据
                var jsonStr = viewModel.getCurveDataJson(viewModel.current_boom_length)
                console.log("Curve data JSON:", jsonStr)
                var curveData = JSON.parse(jsonStr)
                
                if (curveData.length === 0) {
                    ctx.fillStyle = "#94a3b8"
                    ctx.font = "16px " + Theme.fontFamilyDefault
                    ctx.textAlign = "center"
                    ctx.fillText("无数据", width / 2, height / 2)
                    return
                }
                
                var padding = 50
                var chartWidth = width - padding * 2
                var chartHeight = height - padding * 2
                
                // 使用全局最大值确保所有曲线使用相同的坐标系
                var maxX = viewModel.getGlobalMaxRadius()
                var maxY = viewModel.getGlobalMaxLoad()
                
                // 绘制网格线
                ctx.strokeStyle = "#314158"
                ctx.lineWidth = 1
                
                // 水平网格线（Y轴）
                var ySteps = 7
                for (var i = 0; i <= ySteps; i++) {
                    var y = padding + (chartHeight / ySteps) * i
                    ctx.beginPath()
                    ctx.moveTo(padding, y)
                    ctx.lineTo(padding + chartWidth, y)
                    ctx.stroke()
                }
                
                // 垂直网格线（X轴）
                var xSteps = 5
                for (var j = 0; j <= xSteps; j++) {
                    var x = padding + (chartWidth / xSteps) * j
                    ctx.beginPath()
                    ctx.moveTo(x, padding)
                    ctx.lineTo(x, padding + chartHeight)
                    ctx.stroke()
                }
                
                // 绘制曲线
                ctx.strokeStyle = "#22c55e"
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
                ctx.fillStyle = "#22c55e"
                for (k = 0; k < curveData.length; k++) {
                    px = padding + (curveData[k].x / maxX) * chartWidth
                    py = padding + chartHeight - (curveData[k].y / maxY) * chartHeight
                    ctx.beginPath()
                    ctx.arc(px, py, 4, 0, 2 * Math.PI)
                    ctx.fill()
                }
                
                // Y轴标签（载荷）
                ctx.fillStyle = "#94a3b8"
                ctx.font = "12px " + Theme.fontFamilyDefault
                ctx.textAlign = "right"
                var yStep = maxY / ySteps
                for (i = 0; i <= ySteps; i++) {
                    y = padding + (chartHeight / ySteps) * i
                    var yLabel = (maxY - i * yStep).toFixed(0)
                    ctx.fillText(yLabel + "t", padding - 10, y + 4)
                }
                
                // X轴标签（幅度）
                ctx.textAlign = "center"
                ctx.fillStyle = "#94a3b8"
                var xStep = maxX / xSteps
                for (j = 0; j <= xSteps; j++) {
                    x = padding + (chartWidth / xSteps) * j
                    var xLabel = (j * xStep).toFixed(0)
                    ctx.fillText(xLabel + "m", x, padding + chartHeight + 25)
                }
                
                // 坐标轴标题
                ctx.fillStyle = "#dbeafe"
                ctx.font = "14px " + Theme.fontFamilyDefault
                ctx.textAlign = "center"
                
                // X轴标题
                ctx.fillText("工作幅度 (m)", width / 2, height - 5)
                
                // Y轴标题（旋转90度）
                ctx.save()
                ctx.translate(15, height / 2)
                ctx.rotate(-Math.PI / 2)
                ctx.fillText("额定载荷 (t)", 0, 0)
                ctx.restore()
            }
            
            Component.onCompleted: {
                console.log("LoadCurveChart completed, viewModel:", viewModel)
                if (viewModel && viewModel.data_loaded) {
                    console.log("Data already loaded, requesting paint")
                    requestPaint()
                }
            }
            
            Connections {
                target: viewModel
                function onData_loadedChanged() {
                    console.log("data_loaded changed:", viewModel.data_loaded)
                    if (viewModel.data_loaded) {
                        canvas.requestPaint()
                    }
                }
                function onSelected_boom_indexChanged() {
                    console.log("selected_boom_index changed:", viewModel.selected_boom_index)
                    canvas.requestPaint()
                }
            }
            
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
        }
    }
}
