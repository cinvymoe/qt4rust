#!/bin/bash
#
# setup-device-time.sh - Configure device time and RTC persistence
# Usage: ./setup-device-time.sh
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

check_adb() {
    local devices
    devices=$(adb devices | grep -v "List of devices" | grep "device$" | wc -l)
    if [ "$devices" -eq 0 ]; then
        log_error "No ADB device connected. Please connect your device first."
        exit 1
    fi
    log_info "ADB device connected: $(adb devices | grep 'device$' | awk '{print $1}')"
}

sync_time() {
    log_info "Syncing time from host to device..."
    local host_time
    host_time=$(date +%s)
    adb shell "date -s @${host_time}"
    log_info "System time synced: $(adb shell date | tr -d '\r')"
}

save_to_rtc() {
    log_info "Writing system time to hardware RTC..."
    if ! adb shell "which hwclock" &>/dev/null; then
        log_error "hwclock not found on device"
        exit 1
    fi
    adb shell "hwclock --systohc"
    log_info "RTC time saved: $(adb shell 'hwclock --show' | tr -d '\r')"
}

create_init_script() {
    log_info "Creating hwclock init script..."
    if adb shell "[ -f /etc/init.d/S02hwclock ]" &>/dev/null; then
        log_warn "S02hwclock already exists, overwriting..."
    fi
    adb shell "cat > /etc/init.d/S02hwclock << 'EOFSCRIPT'
#!/bin/sh

case \"\$1\" in
    start)
        echo 'Setting system time from RTC...'
        hwclock --hctosys 2>/dev/null
        ;;
    stop)
        echo 'Saving system time to RTC...'
        hwclock --systohc 2>/dev/null
        ;;
esac
EOFSCRIPT"
    adb shell "chmod +x /etc/init.d/S02hwclock"
    log_info "Init script created: /etc/init.d/S02hwclock"
}

verify() {
    log_info "Verifying time configuration..."
    echo ""
    echo "=== Host Time ==="
    date
    echo ""
    echo "=== Device System Time ==="
    adb shell date
    echo ""
    echo "=== Device RTC Time ==="
    adb shell "hwclock --show"
    echo ""
    echo "=== Init Script ==="
    adb shell "ls -la /etc/init.d/S02hwclock"
    echo ""
}

main() {
    echo "======================================"
    echo "  Device Time Configuration Script"
    echo "======================================"
    echo ""

    check_adb
    sync_time
    save_to_rtc
    create_init_script
    verify

    log_info "All done! Device time is now configured and will persist on reboot."
}

main "$@"
