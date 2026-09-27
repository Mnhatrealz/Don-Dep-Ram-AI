use crate::config::AppConfig;
use crate::dsl;
use crate::memory;
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

pub struct AiAgent {
    config: AppConfig,
    client: reqwest::blocking::Client,
}

impl AiAgent {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Gửi câu lệnh ngôn ngữ tự nhiên của người dùng tới AI kèm ngữ cảnh phần cứng thời gian thực
    pub fn ask(&self, user_prompt: &str) -> Result<String, String> {
        let mem = memory::get_memory_info();

        let system_prompt = format!(
            r#"Bạn là AI Quản Lý Phần Cứng & Tối Ưu Hệ Thống Windows của ứng dụng MemScript-AI.
Thông số phần cứng hiện tại của máy tính:
- RAM Tổng: {} MB
- RAM Đang Dùng: {} MB ({}%)
- RAM Còn Trống: {} MB
- Standby Cache ước tính: {} MB

Bạn có thể tự động điều khiển phần cứng bằng cách trả về một khối mã kịch bản MemScript trong dấu ```memscript:
Các lệnh MemScript hợp lệ:
- clean ram.all            (Dọn toàn bộ RAM: Standby + WorkingSets)
- clean ram.standby        (Chỉ xả Standby Cache để chống drop FPS game)
- clean ram.workingset     (Thu hồi WorkingSet các app)
- clean disk.temp          (Dọn dẹp file tạm, %TEMP%, Windows Temp, CrashDumps)
- clean gpu.shader         (Xóa Shader Cache của NVIDIA, AMD, DirectX để chống giật)
- cpu.timer_boost          (Mở khóa Timer Resolution xuống 0.5ms để giảm Input Lag)
- cpu.timer_restore        (Khôi phục Timer Resolution)
- log "Tin nhắn thông báo"

Yêu cầu:
1. Trả lời người dùng ngắn gọn, chuyên nghiệp bằng tiếng Việt.
2. Nếu câu nói của người dùng yêu cầu hành động (ví dụ dọn RAM, tối ưu game, dọn rác), BẮT BUỘC sinh khối code ```memscript ... ``` để ứng dụng tự động thực thi."#,
            mem.total_mb, mem.used_mb, mem.usage_percent, mem.free_mb, mem.standby_estimate_mb
        );

        let url = format!("{}/chat/completions", self.config.ai.base_url.trim_end_matches('/'));

        let payload = ChatCompletionRequest {
            model: self.config.ai.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string>,
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string>,
                    content: user_prompt.to_string(),
                },
            ],
            temperature: self.config.ai.temperature,
            max_tokens: self.config.ai.max_tokens,
        };

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.ai.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .map_err(|e| format!("Lỗi kết nối API AI: {}", e))?;

        if !res.status().is_success() {
            let err_text = res.text().unwrap_or_default();
            return Err(format!("API trả về mã lỗi: {}", err_text));
        }

        let resp_json: ChatCompletionResponse = res
            .json()
            .map_err(|e| format!("Lỗi giải mã JSON phản hồi: {}", e))?;

        if let Some(first_choice) = resp_json.choices.into_iter().next() {
            let reply = first_choice.message.content;
            self.try_execute_generated_script(&reply);
            Ok(reply)
        } else {
            Err("Không nhận được phản hồi từ AI".to_string())
        }
    }

    /// Trích xuất và tự động thực thi kịch bản MemScript mà AI tạo ra
    fn try_execute_generated_script(&self, reply: &str) {
        if let Some(start) = reply.find("```memscript") {
            let code_part = &reply[start + 12..];
            if let Some(end) = code_part.find("```") {
                let script = &code_part[..end].trim();
                if !script.is_empty() {
                    println!("\n{}", "⚙️ [AI Tự Động Kích Hoạt Kịch Bản MemScript]:".cyan().bold());
                    for line in script.lines() {
                        println!("   {}", line.italic().bright_black());
                    }
                    println!();
                    dsl::execute_script(script);
                }
            }
        }
    }
}
