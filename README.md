# MemScript-AI ⚡
> **Hệ thống tối ưu hóa toàn diện Phần cứng Windows (RAM, CPU, GPU, Disk) bằng Ngôn ngữ chuyên biệt (DSL) kết hợp Trí tuệ nhân tạo (AI)**

[![Build and Release](https://github.com/your-username/memscript-ai/actions/workflows/build.yml/badge.svg)](https://github.com/your-username/memscript-ai/actions/workflows/build.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

---

## 🌟 TÍNH NĂNG NỔI BẬT

* 🧠 **Bộ não AI Tự Động Hóa (xKiro API):**
  * Tích hợp model **`qwen/qwen3.5-omni-flash:free`** (hoặc `cohere/command-r7b-12-2024`).
  * Tự động phân tích câu lệnh tự nhiên của người dùng (ví dụ: *"Tối ưu máy cho tao chơi game"*) $\rightarrow$ Tự sinh mã kịch bản `MemScript` và thực thi ngay lập tức.
* 💾 **Quản lý RAM Native cấp thấp (Kế thừa từ Mem Reduct):**
  * Sử dụng Windows NT Native API (`NtSetSystemInformation` qua `ntdll.dll`).
  * Xả sạch **Standby Page List** (nguyên nhân gây khựng/drop FPS game).
  * Thu hồi **Working Sets** của tất cả tiến trình.
  * Thu gọn **System File Cache**.
* ⚡ **Tối ưu hóa CPU & Giảm độ trễ:**
  * Mở khóa xung nhịp **Timer Resolution xuống 0.5ms** (giảm thiểu tối đa Input Lag chuột/phím).
  * Hỗ trợ nâng độ ưu tiên tiến trình và ghim luồng CPU.
* 🎮 **Tối ưu hóa GPU (DirectX / NVIDIA / AMD):**
  * Tự động quét và dọn sạch **Shader Cache** cũ bị phân mảnh (`D3DSCache`, `DXCache`, `GLCache`).
* 🧹 **Dọn rác & File tạm ổ cứng:**
  * Làm sạch `%TEMP%`, `C:\Windows\Temp`, CrashDumps, Windows Update Cache và thùng rác an toàn.
* 📜 **Ngôn ngữ kịch bản riêng (`MemScript`):**
  * Cho phép tự viết kịch bản tối ưu hóa cực kỳ trực quan và an toàn (Sandbox).

---

## 🚀 HƯỚNG DẪN TỰ ĐỘNG BUILD BẰNG GITHUB ACTIONS (0% TẢI GÌ VỀ MÁY)

Bạn **không cần cài đặt Rust hay C++ trên máy tính**. Hãy để GitHub tự động build ra file `.exe`:

### Bước 1: Tạo Repository trên GitHub
1. Đăng nhập vào [GitHub](https://github.com) và tạo một repository mới (ví dụ: `memscript-ai`).
2. Tải toàn bộ thư mục mã nguồn này lên GitHub (thông qua Git hoặc kéo thả trực tiếp trên giao diện web của GitHub).

### Bước 2: Nhận file `.exe` từ GitHub Actions
1. Ngay khi code được đẩy lên, chuyển sang tab **Actions** trên GitHub.
2. Bạn sẽ thấy tiến trình **"Build and Release MemScript-AI"** đang tự động chạy trên máy ảo Windows của GitHub.
3. Sau khoảng 1-2 phút khi build xong:
   * Bấm vào workflow vừa chạy $\rightarrow$ Tải file nén **`MemScript-AI-Windows-x64.zip`** trong mục **Artifacts**.
   * Hoặc nếu bạn tạo thẻ Tag (ví dụ `v1.0.0`), GitHub sẽ tự động tạo một bản Release chính thức kèm file `.zip` tải về!

---

## 💻 CÁCH SỬ DỤNG

Sau khi tải file `memscript-ai.exe` về:

1. **Nhấp chuột phải vào `memscript-ai.exe` $\rightarrow$ Chọn `Run as administrator`** (bắt buộc quyền Admin để can thiệp Native API dọn RAM).
2. **Cấu hình API AI (`config.json`):**
   ```json
   {
     "ai": {
       "api_key": "sk-xt-xxxxxxxxxxxxxxxx",
       "base_url": "https://api.xkiro.com/v1",
       "model": "qwen/qwen3.5-omni-flash:free",
       "temperature": 0.3,
       "max_tokens": 1024
     },
     "automation": {
       "auto_clean_enabled": true,
       "ram_threshold_percent": 85,
       "check_interval_seconds": 15,
       "timer_resolution_boost": true
     }
   }
   ```
3. **Các câu lệnh trong bảng điều khiển:**
   * Gõ `1` hoặc `clean ram`: Dọn dẹp toàn diện RAM.
   * Gõ `2` hoặc `standby`: Xả Standby Cache.
   * Gõ `3` hoặc `temp`: Dọn rác ổ đĩa.
   * Gõ `4` hoặc `shader`: Dọn GPU Shader Cache.
   * Gõ `5` hoặc `boost`: Kích hoạt Timer Resolution 0.5ms.
   * Gõ `run examples/gaming_boost.ms`: Thực thi file kịch bản.
   * Gõ `ai <câu lệnh>` hoặc gõ thẳng câu lệnh tự nhiên:
     > `MemScript-AI> Tối ưu hóa máy tính để chuẩn bị vào game bắn súng Valorant`
     > $\rightarrow$ AI sẽ tự động phân tích và kích hoạt kịch bản dọn RAM, Timer 0.5ms và dọn Shader Cache!

---

## 📜 CÚ PHÁP NGÔN NGỮ MEMSCRIPT

Ví dụ file `gaming_boost.ms`:
```memscript
log "Kích hoạt chế độ Gaming..."
cpu.timer_boost
clean ram.all
clean gpu.shader
clean disk.temp
log "Hoàn tất tối ưu!"
```

---

## 🛡️ BẢN QUYỀN
Phát triển dựa trên các nghiên cứu kỹ thuật hệ thống Windows NT Native API (GPL-3.0).
