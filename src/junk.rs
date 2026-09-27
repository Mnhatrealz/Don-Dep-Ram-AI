use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use windows_sys::Win32::UI::Shell::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND};

pub struct JunkCleanResult {
    pub files_deleted: usize,
    pub bytes_freed: u64,
}

/// Dọn dẹp các thư mục rác hệ thống và file tạm
pub fn clean_all_junk() -> JunkCleanResult {
    let mut total_files = 0;
    let mut total_bytes = 0;

    let mut folders_to_clean: Vec<PathBuf> = Vec::new();

    // 1. Thư mục Temp người dùng (%TEMP%)
    if let Ok(user_temp) = env::var("TEMP") {
        folders_to_clean.push(PathBuf::from(user_temp));
    }

    // 2. Thư mục Temp của hệ thống
    folders_to_clean.push(PathBuf::from("C:\\Windows\\Temp"));

    // 3. Crash Dumps
    if let Some(local_appdata) = dirs::data_local_dir() {
        folders_to_clean.push(local_appdata.join("CrashDumps"));
    }

    // 4. Windows Update Download Cache
    folders_to_clean.push(PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download"));

    // 5. Prefetch (Chỉ xóa file cũ)
    folders_to_clean.push(PathBuf::from("C:\\Windows\\Prefetch"));

    for folder in folders_to_clean {
        if folder.exists() && folder.is_dir() {
            let (f, b) = clean_dir_safe(&folder);
            total_files += f;
            total_bytes += b;
        }
    }

    // 6. Dọn Thùng Rác (Recycle Bin)
    empty_recycle_bin();

    JunkCleanResult {
        files_deleted: total_files,
        bytes_freed: total_bytes,
    }
}

/// Xóa các file trong thư mục nhưng bỏ qua file đang bị ứng dụng khóa (In use)
fn clean_dir_safe(dir: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut bytes = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = entry.metadata() {
                    let len = meta.len();
                    // Thử xóa file, nếu file đang dùng bởi tiến trình khác thì bỏ qua không crash
                    if fs::remove_file(&path).is_ok() {
                        count += 1;
                        bytes += len;
                    }
                }
            } else if path.is_dir() {
                let (sub_c, sub_b) = clean_dir_safe(&path);
                count += sub_c;
                bytes += sub_b;
                let _ = fs::remove_dir(&path);
            }
        }
    }

    (count, bytes)
}

/// Dọn sạch thùng rác Windows không hiển thị hộp thoại
pub fn empty_recycle_bin() {
    unsafe {
        let flags = SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND;
        let _ = SHEmptyRecycleBinW(std::ptr::null_mut(), std::ptr::null(), flags);
    }
}
