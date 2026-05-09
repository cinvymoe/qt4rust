#!/usr/bin/env python3
"""
处理复杂的翻译调用模式
"""
import re
from pathlib import Path

def fix_file(file_path: Path) -> bool:
    """修复单个文件"""
    print(f"Processing: {file_path}")
    content = file_path.read_text(encoding='utf-8')
    original = content
    
    lines = content.split('\n')
    result = []
    
    for i, line in enumerate(lines):
        modified_line = line
        
        # 模式1: 字符串拼接 text: xxx + TranslationBridge.translate("key")
        # 替换为: text: { const _ = TranslationBridge.locale_version; return xxx + TranslationBridge.translate("key") }
        if 'TranslationBridge.translate(' in line and 'text:' in line and '+' in line:
            match = re.match(r'^(\s*)text:\s*(.+)$', line)
            if match and '{ const _' not in line:
                indent = match.group(1)
                expr = match.group(2)
                modified_line = f'{indent}text: {{ const _ = TranslationBridge.locale_version; return {expr} }}'
                print(f"  Fixed line {i+1}: string concatenation")
        
        # 模式2: label:, unit:, description: 等属性
        elif re.match(r'^\s*(label|unit|description|placeholderText|sensorName|statusText):\s*TranslationBridge\.translate\(', line):
            match = re.match(r'^(\s*)(\w+):\s*TranslationBridge\.translate\("([^"]+)"\)(\s*\|\|\s*"[^"]*")?\s*$', line)
            if match and '{ const _' not in line:
                indent = match.group(1)
                prop_name = match.group(2)
                key = match.group(3)
                default_value = match.group(4) or ''
                modified_line = f'{indent}{prop_name}: {{ const _ = TranslationBridge.locale_version; return TranslationBridge.translate("{key}"){default_value} }}'
                print(f"  Fixed line {i+1}: {prop_name} property")
        
        # 模式3: return TranslationBridge.translate(...) 在函数中
        elif 'return TranslationBridge.translate(' in line:
            match = re.match(r'^(\s*)return TranslationBridge\.translate\("([^"]+)"\)(\s*\|\|\s*"[^"]*")?\s*$', line)
            if match and 'locale_version' not in line:
                indent = match.group(1)
                key = match.group(2)
                default_value = match.group(3) or ''
                # 在 return 前添加 const _ = ...
                modified_line = f'{indent}const _ = TranslationBridge.locale_version; return TranslationBridge.translate("{key}"){default_value}'
                print(f"  Fixed line {i+1}: return statement")
        
        result.append(modified_line)
    
    new_content = '\n'.join(result)
    
    if new_content != original:
        file_path.write_text(new_content, encoding='utf-8')
        print(f"  ✓ Modified")
        return True
    else:
        print(f"  - No changes")
        return False

def main():
    qml_dir = Path('/mnt/sdb1/qt4rust/.worktrees/i18n/qml')
    
    qml_files = list(qml_dir.rglob('*.qml'))
    qml_files = [f for f in qml_files if not f.name.endswith('.bak')]
    
    print(f"Found {len(qml_files)} QML files\n")
    
    modified_count = 0
    for qml_file in sorted(qml_files):
        if fix_file(qml_file):
            modified_count += 1
    
    print(f"\nDone! Modified {modified_count} files.")

if __name__ == '__main__':
    main()
