// Counter 业务对象 - 使用 cxx-qt 桥接到 QML

#[cxx_qt::bridge]
pub mod counter_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, count)]
        #[qproperty(QString, platform_info)]
        type Counter = super::CounterRust;

        #[qinvokable]
        unsafe fn increment(self: Pin<&mut Counter>);

        #[qinvokable]
        unsafe fn reset(self: Pin<&mut Counter>);
    }
}

// Re-export Counter for easier access (used by QML)
#[allow(unused_imports)]
pub use counter_bridge::Counter;

use core::pin::Pin;
use cxx_qt_lib::QString;

/// Counter 结构体 - 存储计数器状态和平台信息
pub struct CounterRust {
    count: i32,
    platform_info: QString,
}

impl Default for CounterRust {
    fn default() -> Self {
        let platform_info = Self::get_platform_info();
        Self {
            count: 0,
            platform_info,
        }
    }
}

impl CounterRust {
    /// 获取平台信息字符串
    /// 根据编译目标判断平台类型
    fn get_platform_info() -> QString {
        #[cfg(target_arch = "arm")]
        {
            QString::from("Linux ARM32")
        }
        #[cfg(target_arch = "aarch64")]
        {
            QString::from("Linux ARM64")
        }
        #[cfg(target_arch = "x86_64")]
        {
            QString::from("Linux x86_64")
        }
        #[cfg(not(any(target_arch = "arm", target_arch = "aarch64", target_arch = "x86_64")))]
        {
            QString::from("Unknown Platform")
        }
    }
}

// 实现 Counter 的方法
impl counter_bridge::Counter {
    /// 增加计数值
    /// 包含溢出保护，限制在 i32::MAX
    pub fn increment(mut self: Pin<&mut Self>) {
        let count = *self.as_ref().count();
        if count < i32::MAX {
            self.as_mut().set_count(count + 1);
        } else {
            eprintln!("[WARN] Counter reached maximum value ({})", i32::MAX);
        }
    }

    /// 重置计数值为 0
    pub fn reset(mut self: Pin<&mut Self>) {
        self.as_mut().set_count(0);
    }
}
