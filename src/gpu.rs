use std::fs;
use std::path::{Path, PathBuf};

pub struct GpuCleanResult {
    pub files_deleted: usize,
    pub bytes_freed: u64,
}

/// Quét và làm sạch an toàn các bộ nhớ đệm Shader của GPU (DirectX, NVIDIA, AMD)
pub fn clean_gpu_shader_cache() -> GpuCleanResult {
    let mut files_deleted = 0;
    let mut bytes_freed = 0;

    let local_appdata = dirs::data_local_dir();
    if let Some(base) = local_appdata {
        let targets = [
            base.join("D3DSCache"),
            base.join("NVIDIA").join("DXCache"),
            base.join("NVIDIA").join("GLCache"),
            base.join("AMD").join("DxCache"),
            base.join("AMD").join("GLCache"),
            base.join("Intel").join("ShaderCache"),
        ];

        for folder in &targets {
            if folder.exists() && folder.is_dir() {
                let (cnt, bytes) = clean_folder_contents(folder);
                files_deleted += cnt;
                bytes_freed += bytes;
            }
        }
    }

    GpuCleanResult {
        files_deleted,
        bytes_freed,
    }
}

fn clean_folder_contents(dir: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut bytes = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = entry.metadata() {
                    let len = meta.len();
                    if fs::remove_file(&path).is_ok() {
                        count += 1;
                        bytes += len;
                    }
                }
            } else if path.is_dir() {
                let (sub_cnt, sub_bytes) = clean_folder_contents(&path);
                count += sub_cnt;
                bytes += sub_bytes;
                let _ = fs::remove_dir(&path);
            }
        }
    }

    (count, bytes)
}
