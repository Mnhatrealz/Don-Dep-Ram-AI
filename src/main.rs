mod ai;
mod config;
mod cpu;
mod dsl;
mod gpu;
mod junk;
mod memory;

use ai::AiAgent;
use colored::Colorize;
use config::AppConfig;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // 1. Nâng quyền Administrator Token
    memory::enable_privileges();

    // 2. Nạp cấu hình
    let config = AppConfig::load_or_default();
    let ai_agent = AiAgent::new(config.clone());

    // 3. Hiển thị Banner Dashboard
    print_banner(&config);

    // 4. Khởi động luồng chạy ngầm tự động dọn RAM (nếu bật)
    let auto_clean_active = Arc::new(AtomicBool::new(config.automation.auto_clean_enabled));
    let auto_active_clone = Arc::clone(&auto_clean_active);
    let auto_cfg = config.clone();

    thread::spawn(move || {
        let interval = Duration::from_secs(auto_cfg.automation.check_interval_seconds);
        loop {
            thread::sleep(interval);
            if auto_active_clone.load(Ordering::Relaxed) {
                let mem = memory::get_memory_info();
                if (mem.usage_percent as u64) >= auto_cfg.automation.ram_threshold_percent {
                    println!(
                        "\n{} RAM chạm ngưỡng {}% (Hiện tại: {}%), đang kích hoạt tự động dọn...",
                        "[Auto-Clean]".yellow().bold(),
                        auto_cfg.automation.ram_threshold_percent,
                        mem.usage_percent
                    );
                    let (b, a, freed) = memory::clean_all_ram();
                    println!(
                        "{} Đã xả RAM tự động: {}MB -> {}MB (Thu hồi: {}MB)\n> ",
                        "[Auto-Clean]".green().bold(),
                        b,
                        a,
                        freed.to_string().cyan().bold()
                    );
                    io::stdout().flush().unwrap();
                }
            }
        }
    });

    // 5. Tự động kích hoạt Timer Boost nếu cấu hình cho phép
    if config.automation.timer_resolution_boost {
        if let Ok(ms) = cpu::set_timer_resolution_05ms() {
            println!(
                "{} Timer Resolution đã mở khóa ở mức {}ms (Giảm Input Lag)",
                "⚡ [CPU]".magenta().bold(),
                ms
            );
        }
    }

    // 6. Vòng lặp tương tác dòng lệnh (Interactive CLI)
    let stdin = io::stdin();
    loop {
        print!("{}", "\nMemScript-AI> ".bold().cyan());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            break;
        }

        let cmd = input.trim();
        if cmd.is_empty() {
            continue;
        }

        match cmd.to_lowercase().as_str() {
            "exit" | "quit" | "q" => {
                println!("{}", "Đang khôi phục cài đặt và thoát ứng dụng...".yellow());
                cpu::restore_timer_resolution();
                break;
            }
            "1" | "clean ram" | "ram" => {
                println!("{}", "Đang dọn dẹp toàn diện bộ nhớ RAM...".cyan());
                let (b, a, freed) = memory::clean_all_ram();
                println!(
                    "{} Thành công! RAM: {}MB -> {}MB (Đã giải phóng: {}MB)",
                    "✔ [RAM]".green().bold(),
                    b,
                    a,
                    freed.to_string().cyan().bold()
                );
            }
            "2" | "clean standby" | "standby" => {
                println!("{}", "Đang xả Standby Page Cache...".cyan());
                match memory::purge_standby_list() {
                    Ok(_) => println!("{} Đã xả sạch Standby Cache!", "✔ [RAM]".green().bold()),
                    Err(e) => eprintln!("{} Lỗi: {}", "✖ [RAM]".red().bold(), e),
                }
            }
            "3" | "clean disk" | "temp" => {
                println!("{}", "Đang quét và dọn rác hệ thống (%TEMP%, CrashDumps, Update Cache)...".cyan());
                let res = junk::clean_all_junk();
                let mb = res.bytes_freed / (1024 * 1024);
                println!(
                    "{} Đã xóa {} file rác, thu hồi: {}MB ổ đĩa!",
                    "✔ [DISK]".green().bold(),
                    res.files_deleted,
                    mb.to_string().cyan().bold()
                );
            }
            "4" | "clean gpu" | "shader" => {
                println!("{}", "Đang quét và dọn GPU Shader Cache (DirectX, NVIDIA, AMD)...".cyan());
                let res = gpu::clean_gpu_shader_cache();
                let mb = res.bytes_freed / (1024 * 1024);
                println!(
                    "{} Đã xóa {} file Shader Cache, thu hồi: {}MB!",
                    "✔ [GPU]".green().bold(),
                    res.files_deleted,
                    mb.to_string().cyan().bold()
                );
            }
            "5" | "timer" | "boost" => match cpu::set_timer_resolution_05ms() {
                Ok(ms) => println!(
                    "{} Đã kích hoạt Timer Resolution {}ms!",
                    "✔ [CPU]".green().bold(),
                    ms
                ),
                Err(e) => eprintln!("{} Lỗi: {}", "✖ [CPU]".red().bold(), e),
            },
            "status" => {
                print_status();
            }
            "auto" => {
                let current = auto_clean_active.load(Ordering::Relaxed);
                auto_clean_active.store(!current, Ordering::Relaxed);
                if !current {
                    println!("{}", "✔ Đã BẬT chế độ tự động dọn RAM nền.".green());
                } else {
                    println!("{}", "✖ Đã TẮT chế độ tự động dọn RAM nền.".yellow());
                }
            }
            c if c.starts_with("run ") => {
                let file_path = &cmd[4..].trim();
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    println!("{} Đang thực thi kịch bản '{}'...", "▶ [RUN]".cyan().bold(), file_path);
                    dsl::execute_script(&content);
                } else {
                    eprintln!("{} Không thể đọc file kịch bản '{}'", "✖".red(), file_path);
                }
            }
            c if c.starts_with("ai ") => {
                let prompt = &cmd[3..].trim();
                println!("{} Đang gửi yêu cầu tới AI ({})...", "🤖 [AI]".cyan().bold(), config.ai.model);
                match ai_agent.ask(prompt) {
                    Ok(reply) => {
                        println!("\n{} {}\n", "🤖 [AI Trả Lời]:".green().bold(), reply);
                    }
                    Err(e) => {
                        eprintln!("\n{} {}\n", "✖ [AI Lỗi]:".red().bold(), e);
                    }
                }
            }
            _ => {
                // Mặc định: nếu không khớp lệnh nội bộ, tự động chuyển thành câu hỏi cho AI!
                println!("{} Đang chuyển lệnh tới AI ({}) để phân tích & tự động thực thi...", "🤖 [AI]".cyan().bold(), config.ai.model);
                match ai_agent.ask(cmd) {
                    Ok(reply) => {
                        println!("\n{} {}\n", "🤖 [AI Trả Lời]:".green().bold(), reply);
                    }
                    Err(e) => {
                        eprintln!("\n{} {}\n", "✖ [AI Lỗi]:".red().bold(), e);
                    }
                }
            }
        }
    }
}

fn print_banner(config: &AppConfig) {
    println!("{}", "=================================================================".bright_blue());
    println!("{}", "       MemScript-AI v1.0 - Windows Hardware & Memory Optimizer   ".bold().green());
    println!("{}", "   Powered by Rust Native Engine + xKiro AI (Qwen3.5-Omni-Flash) ".cyan());
    println!("{}", "=================================================================".bright_blue());
    print_status();
    println!("{}", "\nDanh mục lệnh điều khiển:".bold().yellow());
    println!("  {}     : Dọn dẹp toàn diện RAM (WorkingSet + Standby + Cache)", "[1] hoặc clean ram".cyan());
    println!("  {} : Xả sạch Standby Cache (Chống giật lag game)", "[2] hoặc standby".cyan());
    println!("  {}    : Dọn rác hệ thống (%TEMP%, CrashDumps, Update Cache)", "[3] hoặc temp".cyan());
    println!("  {}  : Dọn GPU Shader Cache (NVIDIA / AMD / DirectX)", "[4] hoặc shader".cyan());
    println!("  {}   : Mở khóa Timer Resolution 0.5ms (Giảm Input Lag)", "[5] hoặc boost".cyan());
    println!("  {}  : Chạy file kịch bản MemScript", "run <file.ms>".cyan());
    println!("  {}   : Nói chuyện tự nhiên để AI tự động tối ưu hóa", "ai <câu lệnh>".magenta().bold());
    println!("  {}        : Bật/Tắt dọn RAM tự động khi vượt ngưỡng", "auto".cyan());
    println!("  {}        : Thoát chương trình", "exit".cyan());
    println!("{}", "-----------------------------------------------------------------".bright_black());
    println!("💡 Gợi ý: Bạn có thể gõ trực tiếp: 'Tối ưu máy cho tao chơi game' để AI tự xử lý!");
}

fn print_status() {
    let mem = memory::get_memory_info();
    let ram_color = if mem.usage_percent > 85 {
        "ĐỎ (Nguy hiểm)".red().bold()
    } else if mem.usage_percent > 70 {
        "CAM (Cảnh báo)".yellow().bold()
    } else {
        "XANH (Tốt)".green().bold()
    };

    println!("\n{}", "📊 [Trạng thái Phần cứng Thời Gian Thực]:".bold().white());
    println!("   • RAM Sử Dụng   : {} MB / {} MB ({}%) -> {}", mem.used_mb, mem.total_mb, mem.usage_percent, ram_color);
    println!("   • RAM Còn Trống : {} MB", mem.free_mb.to_string().green());
    println!("   • Standby Cache : {} MB", mem.standby_estimate_mb.to_string().cyan());
}
