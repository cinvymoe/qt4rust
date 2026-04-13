#!/bin/bash
set -e

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

SHOW_ACKNOWLEDGED=false
LIMIT=50
DB_PATH="db/crane_data.db"

while [[ $# -gt 0 ]]; do
	case $1 in
	--acknowledged)
		SHOW_ACKNOWLEDGED=true
		shift
		;;
	--limit)
		LIMIT="$2"
		shift 2
		;;
	--help | -h)
		echo "用法: $0 [--acknowledged] [--limit N] [--help]"
		echo ""
		echo "选项:"
		echo "  --acknowledged    显示已确认的报警"
		echo "  --limit N         限制显示最近 N 条记录"
		echo "  --help, -h        显示帮助信息"
		echo ""
		echo "示例:"
		echo "  $0                           # 检查最近 50 条未确认报警"
		echo "  $0 --acknowledged            # 检查所有报警"
		echo "  $0 --limit 10                # 只显示最近 10 条未确认报警"
		exit 0
		;;
	*)
		echo "未知选项: $1"
		echo "使用 --help 查看帮助"
		exit 1
		;;
	esac
done

echo "=========================================="
echo "起重机报警记录检查工具"
echo "=========================================="
echo ""

if ! command -v sqlite3 &>/dev/null; then
	echo "错误: 未安装 sqlite3 命令行工具"
	echo "请安装 sqlite3: sudo apt-get install sqlite3"
	exit 1
fi

echo "[1/3] 正在从设备拉取数据库..."
if ! make pull-db; then
	echo ""
	echo "错误: 拉取数据库失败"
	echo "请检查设备是否已连接 (adb devices)"
	exit 1
fi

echo ""
echo "[2/3] 检查数据库文件..."

if [ ! -f "$DB_PATH" ]; then
	echo "错误: 数据库文件不存在: $DB_PATH"
	exit 1
fi

echo "       数据库文件: $DB_PATH"
echo ""

echo "[3/3] 查询报警记录..."
echo ""

TABLE_EXISTS=$(sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='table' AND name='alarm_records';" 2>/dev/null || true)

if [ -z "$TABLE_EXISTS" ]; then
	echo "警告: 数据库中不存在 alarm_records 表"
	exit 0
fi

if [ "$SHOW_ACKNOWLEDGED" = true ]; then
	WHERE_CLAUSE=""
	echo "查询条件: 所有报警记录"
else
	WHERE_CLAUSE="WHERE acknowledged = 0"
	echo "查询条件: 仅未确认报警"
fi
echo "显示数量: 最近 $LIMIT 条"
echo ""

TOTAL_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records;" 2>/dev/null || echo "0")
UNACK_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE acknowledged = 0;" 2>/dev/null || echo "0")
WARNING_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE alarm_type = 'warning';" 2>/dev/null || echo "0")
DANGER_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE alarm_type = 'danger';" 2>/dev/null || echo "0")

echo "=========================================="
echo "报警统计"
echo "=========================================="
printf "总报警数:     %b%s%b\n" "$GREEN" "$TOTAL_ALARMS" "$NC"
printf "未确认报警:   %b%s%b\n" "$RED" "$UNACK_ALARMS" "$NC"
printf "预警(warning): %b%s%b\n" "$YELLOW" "$WARNING_ALARMS" "$NC"
printf "危险(danger): %b%s%b\n" "$RED" "$DANGER_ALARMS" "$NC"
echo ""

QUERY="SELECT 
    id,
    datetime(timestamp, 'unixepoch', 'localtime') as alarm_time,
    alarm_type,
    printf('%.1f', current_load) as load_ton,
    printf('%.1f', working_radius) as radius_m,
    printf('%.1f', boom_angle) as angle_deg,
    printf('%.1f', moment_percentage) as moment_pct,
    CASE WHEN acknowledged = 1 THEN '已确认' ELSE '未确认' END as status,
    description
FROM alarm_records 
$WHERE_CLAUSE
ORDER BY timestamp DESC 
LIMIT $LIMIT;"

RESULT=$(sqlite3 -header -column "$DB_PATH" "$QUERY" 2>/dev/null || true)

if [ -z "$RESULT" ]; then
	if [ "$SHOW_ACKNOWLEDGED" = false ]; then
		printf "%b✓ 没有未确认的报警记录%b\n" "$GREEN" "$NC"
		echo ""
		echo "提示: 使用 --acknowledged 参数查看历史报警记录"
	else
		echo "数据库中没有报警记录"
	fi
else
	echo "=========================================="
	echo "详细报警记录"
	echo "=========================================="
	echo ""
	echo "$RESULT"
	echo ""

	if [ "$SHOW_ACKNOWLEDGED" = false ]; then
		DANGER_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE alarm_type = 'danger' AND acknowledged = 0;" 2>/dev/null || echo "0")
		if [ "$DANGER_COUNT" -gt 0 ]; then
			printf "%b⚠ 警告: 存在 %s 条未确认的危险报警！%b\n" "$RED" "$DANGER_COUNT" "$NC"
		fi
	fi
fi

echo ""
echo "=========================================="
echo "检查完成"
echo "=========================================="
echo "数据库位置: $DB_PATH"
echo "当前时间: $(date '+%Y-%m-%d %H:%M:%S')"
