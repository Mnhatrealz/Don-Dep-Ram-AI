use super::parser::{Action, Condition, Statement};
use crate::{cpu, gpu, junk, memory};
use colored::Colorize;

pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_statements(&self, statements: &[Statement]) {
        for stmt in statements {
            match stmt {
                Statement::DirectAction(action) => {
                    self.execute_action(action);
                }
                Statement::ConditionalRule { condition, body } => {
                    if self.evaluate_condition(condition) {
                        for act in body {
                            self.execute_action(act);
                        }
                    }
                }
            }
        }
    }

    fn evaluate_condition(&self, cond: &Condition) -> bool {
        let mem = memory::get_memory_info();
        match cond {
            Condition::RamUsageGreaterThan(th) => (mem.usage_percent as u64) > *th,
            Condition::RamUsageLessThan(th) => (mem.usage_percent as u64) < *th,
        }
    }

    pub fn execute_action(&self, action: &Action) {
        match action {
            Action::CleanRamAll => {
                let (b, a, freed) = memory::clean_all_ram();
                println!(
                    "{} Đã xả toàn bộ RAM (Từ {}MB -> {}MB, giải phóng: {}MB)",
                    "[MemScript]".green().bold(),
                    b,
                    a,
                    freed.to_string().cyan().bold()
                );
            }
            Action::CleanRamStandby => {
                match memory::purge_standby_list() {
                    Ok(_) => println!(
                        "{} Đã xả sạch bộ nhớ đệm Standby Cache",
                        "[MemScript]".green().bold()
                    ),
                    Err(e) => eprintln!("{} Lỗi khi xả Standby: {}", "[MemScript]".red(), e),
                }
            }
            Action::CleanRamWorkingSet => {
                match memory::empty_working_sets() {
                    Ok(_) => println!(
                        "{} Đã thu hồi Working Set của các tiến trình",
                        "[MemScript]".green().bold()
                    ),
                    Err(e) => eprintln!("{} Lỗi khi thu hồi WorkingSet: {}", "[MemScript]".red(), e),
                }
            }
            Action::CleanDiskTemp => {
                let res = junk::clean_all_junk();
                let mb = res.bytes_freed / (1024 * 1024);
                println!(
                    "{} Đã dọn {} file rác/temp, thu hồi: {}MB",
                    "[MemScript]".green().bold(),
                    res.files_deleted,
                    mb.to_string().cyan().bold()
                );
            }
            Action::CleanGpuShader => {
                let res = gpu::clean_gpu_shader_cache();
                let mb = res.bytes_freed / (1024 * 1024);
                println!(
                    "{} Đã dọn {} file GPU Shader Cache, thu hồi: {}MB",
                    "[MemScript]".green().bold(),
                    res.files_deleted,
                    mb.to_string().cyan().bold()
                );
            }
            Action::CpuTimerBoost => match cpu::set_timer_resolution_05ms() {
                Ok(res) => println!(
                    "{} Đã kích hoạt Timer Resolution xung nhịp cao ({}ms - Giảm tối đa Input Lag)",
                    "[MemScript]".green().bold(),
                    res
                ),
                Err(e) => eprintln!("{} Không thể tăng Timer Resolution: {}", "[MemScript]".red(), e),
            },
            Action::CpuTimerRestore => {
                cpu::restore_timer_resolution();
                println!(
                    "{} Đã khôi phục Timer Resolution về mặc định",
                    "[MemScript]".green().bold()
                );
            }
            Action::Log(msg) => {
                println!("{} {}", "[MemScript Log]".yellow().bold(), msg);
            }
        }
    }
}
