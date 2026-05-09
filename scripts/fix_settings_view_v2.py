#!/usr/bin/env python3
from pathlib import Path

file_path = Path('/mnt/sdb1/qt4rust/.worktrees/i18n/qml/views/SettingsView.qml')
content = file_path.read_text(encoding='utf-8')

# 恢复原始的 model 定义，但使用函数包装
old_model = '''                            model: [
                                {text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("settings.systemStatus") }, icon: "icon-system-status.svg"},
                                {text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("settings.calibration") }, icon: "icon-calibration.svg"},
                                {text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("settings.momentCurve") }, icon: "icon-moment-curve.svg"},
                                {text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("settings.about") }, icon: "icon-about-system.svg"},
                                {text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("settings.language") }, icon: "icon-language.svg"}
                            ]'''

new_model = '''                            model: getTabModel()'''

content = content.replace(old_model, new_model)

# 添加 getTabModel 函数
# 在 getCurrentTabDescription 函数后添加
old_func_end = '''    // 获取当前 Tab 的描述文本
    function getCurrentTabDescription() {
        const _ = TranslationBridge.locale_version
        switch(currentTabIndex) {
            case 0: return TranslationBridge.translate("systemStatus.sensorConnection")
            case 1: return TranslationBridge.translate("calibration.multiplierDesc")
            case 2: return TranslationBridge.translate("momentCurve.title")
            case 3: return TranslationBridge.translate("about.version")
            case 4: return TranslationBridge.translate("settings.language")
            default: return ""
        }
    }
}'''

new_func_end = '''    // 获取当前 Tab 的描述文本
    function getCurrentTabDescription() {
        const _ = TranslationBridge.locale_version
        switch(currentTabIndex) {
            case 0: return TranslationBridge.translate("systemStatus.sensorConnection")
            case 1: return TranslationBridge.translate("calibration.multiplierDesc")
            case 2: return TranslationBridge.translate("momentCurve.title")
            case 3: return TranslationBridge.translate("about.version")
            case 4: return TranslationBridge.translate("settings.language")
            default: return ""
        }
    }
    
    // 获取 Tab 模型数据
    function getTabModel() {
        const _ = TranslationBridge.locale_version
        return [
            {text: TranslationBridge.translate("settings.systemStatus"), icon: "icon-system-status.svg"},
            {text: TranslationBridge.translate("settings.calibration"), icon: "icon-calibration.svg"},
            {text: TranslationBridge.translate("settings.momentCurve"), icon: "icon-moment-curve.svg"},
            {text: TranslationBridge.translate("settings.about"), icon: "icon-about-system.svg"},
            {text: TranslationBridge.translate("settings.language"), icon: "icon-language.svg"}
        ]
    }
}'''

content = content.replace(old_func_end, new_func_end)

file_path.write_text(content, encoding='utf-8')
print("Fixed SettingsView.qml")
