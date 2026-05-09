#!/usr/bin/env python3
"""
处理 title: TranslationBridge.translate("key") 模式
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
        # 检查是否是 title: TranslationBridge.translate("key") 模式（不是 property string title:）
        if re.match(r'^\s*title:\s*TranslationBridge\.translate\("', line) and 'property' not in line:
            match = re.match(r'^(\s*)title:\s*TranslationBridge\.translate\("([^"]+)"\)(\s*\|\|\s*"[^"]*")?\s*$', line)
            
            if match:
                indent = match.group(1)
                key = match.group(2)
                default_value = match.group(3) or ''
                
                # 修改为带 _localeVersion 的版本
                new_line = f'{indent}title: {{ const _ = TranslationBridge.locale_version; return TranslationBridge.translate("{key}"){default_value} }}'
                result.append(new_line)
                print(f"  Fixed line {i+1}: title")
            else:
                result.append(line)
        else:
            result.append(line)
    
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
