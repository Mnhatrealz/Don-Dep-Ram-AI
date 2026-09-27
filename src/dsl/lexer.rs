#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Clean,
    Ram,
    Cpu,
    Gpu,
    Disk,
    Dot,
    All,
    Standby,
    Workingset,
    Temp,
    Shader,
    TimerBoost,
    TimerRestore,
    When,
    Usage,
    GreaterThan,
    LessThan,
    Percent(u64),
    Number(u64),
    LBrace,
    RBrace,
    StringLit(String),
    Log,
    Eof,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.input[self.pos..].starts_with("//") {
                // Bỏ qua dòng comment //
                while let Some(c) = self.advance() {
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let ch = match self.advance() {
            Some(c) => c,
            None => return Token::Eof,
        };

        match ch {
            '.' => Token::Dot,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '>' => Token::GreaterThan,
            '<' => Token::LessThan,
            '"' => {
                let start = self.pos;
                while let Some(c) = self.advance() {
                    if c == '"' {
                        let text = &self.input[start..self.pos - 1];
                        return Token::StringLit(text.to_string());
                    }
                }
                Token::StringLit(self.input[start..].to_string())
            }
            c if c.is_ascii_digit() => {
                let start = self.pos - c.len_utf8();
                while let Some(p) = self.peek() {
                    if p.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }
                let num: u64 = self.input[start..self.pos].parse().unwrap_or(0);
                if self.peek() == Some('%') {
                    self.advance();
                    Token::Percent(num)
                } else {
                    Token::Number(num)
                }
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = self.pos - c.len_utf8();
                while let Some(p) = self.peek() {
                    if p.is_alphanumeric() || p == '_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                let ident = &self.input[start..self.pos];
                match ident.to_lowercase().as_str() {
                    "clean" => Token::Clean,
                    "ram" => Token::Ram,
                    "cpu" => Token::Cpu,
                    "gpu" => Token::Gpu,
                    "disk" => Token::Disk,
                    "all" => Token::All,
                    "standby" => Token::Standby,
                    "workingset" => Token::Workingset,
                    "temp" => Token::Temp,
                    "shader" => Token::Shader,
                    "timer_boost" => Token::TimerBoost,
                    "timer_restore" => Token::TimerRestore,
                    "when" => Token::When,
                    "usage" => Token::Usage,
                    "log" => Token::Log,
                    _ => Token::StringLit(ident.to_string()),
                }
            }
            _ => self.next_token(),
        }
    }
}
