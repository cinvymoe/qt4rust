#!/usr/bin/env python3
import re
from pathlib import Path

file_path = Path('/mnt/sdb1/qt4rust/.worktrees/i18n/qml/views/SettingsView.qml')
content = file_path.read_text(encoding='utf-8')

# 1. 修改标题栏的 Text 组件
content = re.sub(
    r'text: getCurrentTabTitle\(\)',
    r'text: { const _ = settingsView._localeVersion; return getCurrentTabTitle() }',
    content
)

content = re.sub(
    r'text: getCurrentTabDescription\(\)',
    r'text: { const _ = settingsView._localeVersion; return getCurrentTabDescription() }',
    content
)

# 2. 修改 Repeater model 中的翻译
content = re.sub(
    r'\{text: TranslationBridge\.translate\("([^"]+)"\), icon: "([^"]+)"\}',
    r'{text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("\1") }, icon: "\2"}',
    content
)

# 3. 在函数中添加 locale_version 依赖
content = re.sub(
    r'function getCurrentTabTitle\(\) \{\s*switch\(currentTabIndex\) \{',
    r'function getCurrentTabTitle() {\n        const _ = TranslationBridge.locale_version\n        switch(currentTabIndex) {',
    content
)

content = re.sub(
    r'function getCurrentTabDescription\(\) \{\s*switch\(currentTabIndex\) \{',
    r'function getCurrentTabDescription() {\n        const _ = TranslationBridge.locale_version\n        switch(currentTabIndex) {',
    content
)

file_path.write_text(content, encoding='utf-8')
print("Fixed SettingsView.qml")
