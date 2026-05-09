// TrText.qml - 支持响应式翻译的文本组件
import QtQuick
import qt.rust.demo

Text {
    id: root
    
    // 翻译 key（必需）
    property string trKey: ""
    
    // 翻译参数（可选，JSON 格式）
    property var trArgs: null
    
    // 内部属性：依赖 locale_version 触发更新
    property int _localeVersion: TranslationBridge.locale_version
    
    // 计算翻译文本
    text: {
        // 强制依赖 _localeVersion，确保语言切换时重新计算
        const _ = _localeVersion
        
        if (trKey === "") return ""
        
        if (trArgs) {
            return TranslationBridge.translate_with_args(trKey, JSON.stringify(trArgs))
        }
        
        return TranslationBridge.translate(trKey)
    }
}
