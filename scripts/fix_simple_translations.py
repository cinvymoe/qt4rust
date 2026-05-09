#!/usr/bin/env python3
"""
只修复简单的 text: TranslationBridge.translate("key") 模式
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
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # 检查是否是简单的 text: TranslationBridge.translate("key") 模式
        match = re.match(r'^(\s*)(text|title|subtitle|label|placeholderText):\s*TranslationBridge\.translate\("([^"]+)"\)\s*$', line)
        
        if match:
            indent = match.group(1)
            prop_name = match.group(2)
            key = match.group(3)
            
            # 修改为带 _localeVersion 的版本
            new_line = f'{indent}{prop_name}: {{ const _ = TranslationBridge.locale_version; return TranslationBridge.translate("{key}") }}'
            result.append(new_line)
            print(f"  Fixed line {i+1}: {prop_name}")
        else:
            result.append(line)
        
        i += 1
    
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
    
    # 找到所有 QML 文件
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
