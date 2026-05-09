// TranslationManager.qml - 全局翻译管理器单例
pragma Singleton
import qt.rust.demo
import QtQuick

QtObject {
    id: translationManager
    
    // 翻译刷新计数器
    property int refreshCount: 0
    
    // 监听语言切换信号
    Connections {
        target: TranslationBridge
        function onLocale_changed() {
            translationManager.refreshCount++
        }
    }
    
    // 翻译函数 - 创建响应式绑定
    function tr(key) {
        // 通过访问 refreshCount 强制依赖
        var _ = translationManager.refreshCount
        return TranslationBridge.translate(key)
    }
}
