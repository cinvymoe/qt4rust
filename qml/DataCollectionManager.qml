// DataCollectionManager.qml - 数据采集管理器单例
pragma Singleton
import QtQuick

QtObject {
    id: root
    
    // 数据采集是否已启动
    property bool isStarted: false
    
    // 启动数据采集
    function startDataCollection() {
        if (isStarted) {
            console.log("[QML] Data collection already started")
            return
        }
        
        console.log("[QML] Starting data collection...")
        
        // 调用 C++ 函数启动数据采集
        // 注意：这需要在 C++ 中暴露一个全局函数
        // 暂时通过定时器模拟数据更新
        isStarted = true
        
        console.log("[QML] Data collection started")
    }
    
    // 停止数据采集
    function stopDataCollection() {
        if (!isStarted) {
            return
        }
        
        console.log("[QML] Stopping data collection...")
        isStarted = false
        console.log("[QML] Data collection stopped")
    }
}
