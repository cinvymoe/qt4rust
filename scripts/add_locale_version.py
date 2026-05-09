#!/usr/bin/env python3
import re
from pathlib import Path

def add_locale_version(content):
    lines = content.split('\n')
    result = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # 检查是否是 Text { 行
        if re.match(r'^\s*Text\s*\{', line):
            text_start = i
            brace_count = 0
            text_lines = []
            
            # 收集整个 Text 组件
            j = i
            while j < len(lines):
                text_lines.append(lines[j])
                brace_count += lines[j].count('{') - lines[j].count('}')
                if brace_count == 0 and j > i:
                    break
                j += 1
            
            text_block = '\n'.join(text_lines)
            
            # 检查是否包含 TranslationBridge.translate
            if 'TranslationBridge.translate' in text_block:
                # 检查是否已经有 _localeVersion
                if '_localeVersion' not in text_block:
                    # 在 Text { 后添加 _localeVersion
                    modified_lines = text_lines.copy()
                    indent_match = re.match(r'^(\s*)', text_lines[0])
                    indent = indent_match.group(1) if indent_match else ''
                    modified_lines.insert(1, f'{indent}    property int _localeVersion: TranslationBridge.locale_version')
                    result.extend(modified_lines)
                    i = j + 1
                    continue
        
        result.append(line)
        i += 1
    
    return '\n'.join(result)

def process_file(file_path):
    print(f"Processing: {file_path}")
    content = file_path.read_text(encoding='utf-8')
    modified = add_locale_version(content)
    
    if modified != content:
        file_path.write_text(modified, encoding='utf-8')
        print(f"  ✓ Modified")
    else:
        print(f"  - No changes")

def main():
    qml_dir = Path('/mnt/sdb1/qt4rust/.worktrees/i18n/qml')
    
    # 处理特定文件
    target_files = [
        qml_dir / 'components/dialogs/CustomTimeRangeDialog.qml',
    ]
    
    for qml_file in target_files:
        if qml_file.exists():
            process_file(qml_file)
    
    print("\nDone!")

if __name__ == '__main__':
    main()
