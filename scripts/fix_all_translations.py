#!/usr/bin/env python3
"""
批量修复所有 QML 文件的翻译调用，添加 _localeVersion 依赖
"""
import re
from pathlib import Path
from typing import List, Tuple

def find_component_root(lines: List[str], line_index: int) -> Tuple[str, int]:
    """
    找到当前行所在的组件根节点（Rectangle, RowLayout, Column, Item 等）
    返回 (组件id, 起始行号)
    """
    # QML 组件根节点关键字
    root_keywords = [
        'Rectangle', 'RowLayout', 'ColumnLayout', 'Column', 'Row', 'Item',
        'Page', 'Pane', 'Frame', 'ScrollView', 'ListView', 'GridView',
        'Dialog', 'Popup', 'Drawer', 'ApplicationWindow', 'Window',
        'StackView', 'SwipeView', 'TabBar', 'ToolBar', 'GroupBox'
    ]
    
    brace_count = 0
    for i in range(line_index, -1, -1):
        line = lines[i]
        brace_count += line.count('{') - line.count('}')
        
        # 当 brace_count 回到 0 或负数时，找到了组件根节点
        if brace_count <= 0:
            # 检查是否是组件根节点
            for keyword in root_keywords:
                pattern = rf'^\s*{keyword}\s*\{{'
                if re.match(pattern, line):
                    # 提取 id
                    id_match = re.search(r'id:\s*(\w+)', '\n'.join(lines[i:min(i+10, len(lines))]))
                    component_id = id_match.group(1) if id_match else 'root'
                    return component_id, i
            
            # 如果不是已知的组件根节点，继续向上查找
            if brace_count < 0:
                brace_count = 0
    
    return 'root', 0

def has_locale_version(lines: List[str], start_line: int) -> bool:
    """检查组件是否已经有 _localeVersion 属性"""
    brace_count = 0
    for i in range(start_line, min(start_line + 50, len(lines))):
        line = lines[i]
        if '_localeVersion' in line:
            return True
        brace_count += line.count('{') - line.count('}')
        if brace_count <= 0 and i > start_line:
            break
    return False

def add_locale_version(lines: List[str], start_line: int, component_id: str) -> List[str]:
    """在组件根节点添加 _localeVersion 属性"""
    if has_locale_version(lines, start_line):
        return lines
    
    # 找到第一个 { 后的位置
    insert_pos = start_line + 1
    
    # 确定缩进
    indent_match = re.match(r'^(\s*)', lines[start_line])
    indent = indent_match.group(1) if indent_match else ''
    
    # 插入 _localeVersion 属性
    new_line = f'{indent}    property int _localeVersion: TranslationBridge.locale_version\n'
    lines.insert(insert_pos, new_line)
    
    return lines

def fix_file(file_path: Path) -> bool:
    """修复单个文件"""
    print(f"Processing: {file_path}")
    content = file_path.read_text(encoding='utf-8')
    lines = content.split('\n')
    
    modified = False
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # 检查是否包含 TranslationBridge.translate 且没有 _localeVersion
        if 'TranslationBridge.translate' in line and '_localeVersion' not in line:
            # 检查是否是简单的 text: TranslationBridge.translate("key") 模式
            simple_match = re.match(r'^(\s*)(text|title|subtitle|label|placeholderText):\s*TranslationBridge\.translate\("([^"]+)"\)', line)
            
            if simple_match:
                indent = simple_match.group(1)
                prop_name = simple_match.group(2)
                key = simple_match.group(3)
                
                # 找到组件根节点
                component_id, root_line = find_component_root(lines, i)
                
                # 添加 _localeVersion 到组件根节点
                if not has_locale_version(lines, root_line):
                    lines = add_locale_version(lines, root_line, component_id)
                    i += 1  # 因为插入了一行，需要调整索引
                    modified = True
                
                # 修改当前行
                lines[i] = f'{indent}{prop_name}: {{ const _ = {component_id}._localeVersion; return TranslationBridge.translate("{key}") }}'
                modified = True
        
        i += 1
    
    if modified:
        file_path.write_text('\n'.join(lines), encoding='utf-8')
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
    for qml_file in qml_files:
        if fix_file(qml_file):
            modified_count += 1
    
    print(f"\nDone! Modified {modified_count} files.")

if __name__ == '__main__':
    main()
