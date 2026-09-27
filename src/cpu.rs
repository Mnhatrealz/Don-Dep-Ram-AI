use std::ffi::c_void;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HMODULE};
use windows_sys::Win32::Media::{timeBeginPeriod, timeEndPeriod};
use windows_sys::Win32::System::SystemInformation::GetModuleHandleA;
use windows_sys::Win32::System::Threading::{
    OpenProcess, SetPriorityClass, SetProcessAffinityMask,
    HIGH_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, PROCESS_SET_INFORMATION,
};

type NtSetTimerResolutionFn = unsafe extern "system" fn(
    desired_resolution: u32,
    set_resolution: u8,
    current_resolution: *mut u32,
) -> i32;

type NtQueryTimerResolutionFn = unsafe extern "system" fn(
    maximum_resolution: *mut u32,
    minimum_resolution: *mut u32,
    current_resolution: *mut u32,
) -> i32;

static mut TIMER_BOOSTED: bool = false;

/// Mở khóa Timer Resolution xuống 0.5ms (hoặc 1ms) để giảm tối đa Input Lag và giật khung hình
pub fn set_timer_resolution_05ms() -> Result<f32, String> {
    unsafe {
        let ntdll: HMODULE = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if !ntdll.is_null() {
            let proc = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                ntdll,
                b"NtSetTimerResolution\0".as_ptr(),
            );
            if let Some(func_ptr) = proc {
                let func = std::mem::transmute::<_, NtSetTimerResolutionFn>(func_ptr);
                let desired_100ns = 5000; // 5000 * 100ns = 0.5ms
                let mut current_res = 0u32;
                let status = func(desired_100ns, 1, &mut current_res);
                if status >= 0 {
                    TIMER_BOOSTED = true;
                    return Ok(0.5);
                }
            }
        }

        // Dự phòng: dùng timeBeginPeriod (1ms)
        if timeBeginPeriod(1) == 0 {
            TIMER_BOOSTED = true;
            Ok(1.0)
        } else {
            Err("Không thể mở khóa Timer Resolution".to_string())
        }
    }
}

/// Khôi phục Timer Resolution về mặc định của Windows
pub fn restore_timer_resolution() {
    unsafe {
        if TIMER_BOOSTED {
            let ntdll: HMODULE = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
            if !ntdll.is_null() {
                let proc = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                    ntdll,
                    b"NtSetTimerResolution\0".as_ptr(),
                );
                if let Some(func_ptr) = proc {
                    let func = std::mem::transmute::<_, NtSetTimerResolutionFn>(func_ptr);
                    let mut current_res = 0u32;
                    let _ = func(156250, 0, &mut current_res); // 15.625ms
                }
            }
            timeEndPeriod(1);
            TIMER_BOOSTED = false;
        }
    }
}

/// Đặt độ ưu tiên CPU cho một tiến trình (High Priority)
pub fn set_process_high_priority(pid: u32) -> Result<(), String> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_SET_INFORMATION, 0, pid);
        if handle.is_null() {
            return Err(format!("Không thể mở tiến trình PID {}", pid));
        }

        let success = SetPriorityClass(handle, HIGH_PRIORITY_CLASS);
        CloseHandle(handle);

        if success != 0 {
            Ok(())
        } else {
            Err(format!("Không thể gán HIGH priority cho PID {}", pid))
        }
    }
}

/// Ghim tiến trình vào các nhân CPU mong muốn (CPU Affinity Mask)
pub fn set_process_affinity(pid: u32, affinity_mask: usize) -> Result<(), String> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_SET_INFORMATION, 0, pid);
        if handle.is_null() {
            return Err(format!("Không thể mở tiến trình PID {}", pid));
        }

        let success = SetProcessAffinityMask(handle, affinity_mask);
        CloseHandle(handle);

        if success != 0 {
            Ok(())
        } else {
            Err(format!("Không thể gán Affinity Mask cho PID {}", pid))
        }
    }
}
