#!/bin/bash
# 检查数据库记录数的脚本
#
# 使用方法:
# chmod +x examples/check_db_records.sh
# ./examples/check_db_records.sh [数据库文件路径]

DB_FILE="${1:-crane_data.db}"

if [ ! -f "$DB_FILE" ]; then
    echo "错误: 数据库文件不存在: $DB_FILE"
    exit 1
fi

echo "========================================"
echo "  数据库记录统计"
echo "========================================"
echo "数据库文件: $DB_FILE"
echo ""

# 查询总记录数
echo "【运行数据表】"
RUNTIME_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM runtime_data;")
echo "总记录数: $RUNTIME_COUNT"

# 查询序列号范围
MIN_SEQ=$(sqlite3 "$DB_FILE" "SELECT MIN(sequence_number) FROM runtime_data;")
MAX_SEQ=$(sqlite3 "$DB_FILE" "SELECT MAX(sequence_number) FROM runtime_data;")
echo "序列号范围: $MIN_SEQ - $MAX_SEQ"

# 查询最早的 5 条记录
echo ""
echo "最早的 5 条记录:"
sqlite3 -header -column "$DB_FILE" "SELECT id, sequence_number, current_load, working_radius, timestamp FROM runtime_data ORDER BY id ASC LIMIT 5;"

# 查询最晚的 5 条记录
echo ""
echo "最晚的 5 条记录:"
sqlite3 -header -column "$DB_FILE" "SELECT id, sequence_number, current_load, working_radius, timestamp FROM runtime_data ORDER BY id DESC LIMIT 5;"

# 查询报警记录数
echo ""
echo "【报警记录表】"
ALARM_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM alarm_records;")
echo "总记录数: $ALARM_COUNT"

if [ "$ALARM_COUNT" -gt 0 ]; then
    echo ""
    echo "最近的 5 条报警:"
    sqlite3 -header -column "$DB_FILE" "SELECT id, alarm_type, moment_percentage, timestamp FROM alarm_records ORDER BY id DESC LIMIT 5;"
fi

echo ""
echo "========================================"
