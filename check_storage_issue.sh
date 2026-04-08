#!/bin/sh
# ADB 设备存储问题诊断脚本

echo "=== 存储管道问题诊断 (ADB 设备) ==="
echo ""

# 1. 检查数据库文件
echo "1. 检查数据库文件："
if [ -f "crane_data.db" ]; then
    echo "   数据库文件存在"
    ls -l crane_data.db
else
    echo "   数据库文件不存在"
fi
echo ""

# 2. 检查磁盘空间
echo "2. 检查磁盘空间："
df -h .
echo ""

# 3. 检查文件权限
echo "3. 检查文件权限："
if [ -f "crane_data.db" ]; then
    ls -l crane_data.db
else
    echo "   数据库文件不存在，检查当前目录权限："
    ls -ld .
fi
echo ""

# 4. 检查进程
echo "4. 检查相关进程："
ps | grep -E "qt-rust-demo|crane" || echo "   未找到相关进程"
echo ""

# 5. 检查 WAL 和 journal 文件
echo "5. 检查数据库相关文件："
ls -l crane_data.db* 2>/dev/null || echo "   只有主数据库文件"
echo ""

# 6. 尝试简单的数据库操作
echo "6. 测试数据库访问："
if [ -f "crane_data.db" ]; then
    sqlite3 crane_data.db "SELECT COUNT(*) FROM runtime_data;" 2>&1 || echo "   数据库访问失败"
else
    echo "   数据库文件不存在"
fi
echo ""

echo "=== 诊断完成 ==="
echo ""
echo "建议操作："
echo "1. 如果磁盘空间不足，清理旧数据"
echo "2. 如果权限问题，执行: chmod 666 crane_data.db"
echo "3. 如果数据库损坏，备份后删除重建"
echo "4. 如果进程卡死，重启应用"
