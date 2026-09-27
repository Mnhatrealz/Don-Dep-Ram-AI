use std::ffi::c_void;
use std::mem::size_of;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, HMODULE, LUID};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Memory::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows_sys::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};
use windows_sys::Win32::System::SystemInformation::GetModuleHandleA;
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

// Undocumented Native NT constants (tương tự mã nguồn Mem Reduct)
const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 80;
const SYSTEM_FILE_CACHE_INFORMATION_EX: u32 = 21;
const SYSTEM_COMBINE_PHYSICAL_MEMORY_INFORMATION: u32 = 130;

// Các lệnh của SystemMemoryListInformation
const MEMORY_EMPTY_WORKING_SETS: u32 = 2;
const MEMORY_FLUSH_MODIFIED_LIST: u32 = 3;
const MEMORY_PURGE_STANDBY_LIST: u32 = 4;
const MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST: u32 = 5;

type NtSetSystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *const c_void,
    system_information_length: u32,
) -> i32;

#[repr(C)]
struct SystemFileCacheInformation {
    current_size: usize,
    peak_size: usize,
    page_fault_count: u32,
    minimum_working_set: usize,
    maximum_working_set: usize,
    unused: [usize; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct MemorySnapshot {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub usage_percent: u32,
    pub standby_estimate_mb: u64,
}

/// Nâng quyền tiến trình để can thiệp hệ thống (SeProfileSingleProcessPrivilege, SeIncreaseQuotaPrivilege)
pub fn enable_privileges() -> bool {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return false;
        }

        let privileges = [
            windows_sys::core::w!("SeProfileSingleProcessPrivilege"),
            windows_sys::core::w!("SeIncreaseQuotaPrivilege"),
        ];

        for priv_name in privileges {
            let mut luid = LUID {
                LowPart: 0,
                HighPart: 0,
            };
            if LookupPrivilegeValueW(std::ptr::null(), priv_name, &mut luid) != 0 {
                let mut tp = TOKEN_PRIVILEGES {
                    PrivilegeCount: 1,
                    Privileges: [LUID_AND_ATTRIBUTES {
                        Luid: luid,
                        Attributes: 0x00000002, // SE_PRIVILEGE_ENABLED
                    }],
                };
                AdjustTokenPrivileges(
                    token,
                    0,
                    &mut tp,
                    size_of::<TOKEN_PRIVILEGES>() as u32,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
            }
        }

        CloseHandle(token);
        true
    }
}

fn get_nt_set_system_information() -> Option<NtSetSystemInformationFn> {
    unsafe {
        let ntdll: HMODULE = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if ntdll.is_null() {
            return None;
        }
        let proc = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
            ntdll,
            b"NtSetSystemInformation\0".as_ptr(),
        );
        proc.map(|p| std::mem::transmute::<_, NtSetSystemInformationFn>(p))
    }
}

/// Lấy thông tin thống kê trạng thái RAM thời gian thực
pub fn get_memory_info() -> MemorySnapshot {
    unsafe {
        let mut status: MEMORYSTATUSEX = std::mem::zeroed();
        status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;

        let mut perf_info: PERFORMANCE_INFORMATION = std::mem::zeroed();
        perf_info.cb = size_of::<PERFORMANCE_INFORMATION>() as u32;

        let total_mb = if GlobalMemoryStatusEx(&mut status) != 0 {
            status.ullTotalPhys / (1024 * 1024)
        } else {
            0
        };

        let free_mb = if status.ullAvailPhys > 0 {
            status.ullAvailPhys / (1024 * 1024)
        } else {
            0
        };

        let used_mb = total_mb.saturating_sub(free_mb);
        let usage_percent = status.dwMemoryLoad;

        let mut standby_mb = 0;
        if GetPerformanceInfo(&mut perf_info, perf_info.cb) != 0 {
            let page_size_kb = (perf_info.PageSize / 1024) as u64;
            let system_cache_mb = (perf_info.SystemCache as u64 * page_size_kb) / 1024;
            standby_mb = system_cache_mb;
        }

        MemorySnapshot {
            total_mb,
            used_mb,
            free_mb,
            usage_percent,
            standby_estimate_mb: standby_mb,
        }
    }
}

/// Xả Standby Page List (Bộ nhớ đệm chờ - chống giật lag game)
pub fn purge_standby_list() -> Result<(), String> {
    let func = get_nt_set_system_information().ok_or("Không thể nạp NtSetSystemInformation từ ntdll")?;
    let command = MEMORY_PURGE_STANDBY_LIST;
    unsafe {
        let status = func(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &command as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        );
        if status >= 0 {
            Ok(())
        } else {
            Err(format!("Mã lỗi NTSTATUS: 0x{:X}", status))
        }
    }
}

/// Xả Standby Priority 0 List
pub fn purge_low_priority_standby_list() -> Result<(), String> {
    let func = get_nt_set_system_information().ok_or("Không thể nạp NtSetSystemInformation từ ntdll")?;
    let command = MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST;
    unsafe {
        let status = func(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &command as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        );
        if status >= 0 {
            Ok(())
        } else {
            Err(format!("Mã lỗi NTSTATUS: 0x{:X}", status))
        }
    }
}

/// Thu hồi Working Set của tất cả tiến trình
pub fn empty_working_sets() -> Result<(), String> {
    let func = get_nt_set_system_information().ok_or("Không thể nạp NtSetSystemInformation từ ntdll")?;
    let command = MEMORY_EMPTY_WORKING_SETS;
    unsafe {
        let status = func(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &command as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        );
        if status >= 0 {
            Ok(())
        } else {
            Err(format!("Mã lỗi NTSTATUS: 0x{:X}", status))
        }
    }
}

/// Xả Modified Page List
pub fn flush_modified_list() -> Result<(), String> {
    let func = get_nt_set_system_information().ok_or("Không thể nạp NtSetSystemInformation từ ntdll")?;
    let command = MEMORY_FLUSH_MODIFIED_LIST;
    unsafe {
        let status = func(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &command as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        );
        if status >= 0 {
            Ok(())
        } else {
            Err(format!("Mã lỗi NTSTATUS: 0x{:X}", status))
        }
    }
}

/// Thu gọn System File Cache
pub fn clean_system_file_cache() -> Result<(), String> {
    let func = get_nt_set_system_information().ok_or("Không thể nạp NtSetSystemInformation từ ntdll")?;
    let sfci = SystemFileCacheInformation {
        current_size: 0,
        peak_size: 0,
        page_fault_count: 0,
        minimum_working_set: usize::MAX,
        maximum_working_set: usize::MAX,
        unused: [0; 4],
    };
    unsafe {
        let status = func(
            SYSTEM_FILE_CACHE_INFORMATION_EX,
            &sfci as *const SystemFileCacheInformation as *const c_void,
            size_of::<SystemFileCacheInformation>() as u32,
        );
        if status >= 0 {
            Ok(())
        } else {
            Err(format!("Mã lỗi NTSTATUS: 0x{:X}", status))
        }
    }
}

/// Dọn dẹp toàn diện tất cả các vùng nhớ (Full Purge)
pub fn clean_all_ram() -> (u64, u64, u64) {
    let before = get_memory_info();
    let _ = empty_working_sets();
    let _ = clean_system_file_cache();
    let _ = purge_standby_list();
    let _ = purge_low_priority_standby_list();
    let _ = flush_modified_list();
    let after = get_memory_info();

    let freed_mb = before.used_mb.saturating_sub(after.used_mb);
    (before.used_mb, after.used_mb, freed_mb)
}
