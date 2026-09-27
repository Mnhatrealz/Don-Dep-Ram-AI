use super::lexer::{Lexer, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    CleanRamAll,
    CleanRamStandby,
    CleanRamWorkingSet,
    CleanDiskTemp,
    CleanGpuShader,
    CpuTimerBoost,
    CpuTimerRestore,
    Log(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Condition {
    RamUsageGreaterThan(u64),
    RamUsageLessThan(u64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    DirectAction(Action),
    ConditionalRule {
        condition: Condition,
        body: Vec<Action>,
    },
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
        }
    }

    fn advance(&mut self) -> Token {
        let old = std::mem::replace(&mut self.current_token, self.lexer.next_token());
        old
    }

    pub fn parse_program(&mut self) -> Vec<Statement> {
        let mut stmts = Vec::new();
        while self.current_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                self.advance();
            }
        }
        stmts
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match &self.current_token {
            Token::Clean => {
                self.advance();
                self.parse_clean_action().map(Statement::DirectAction)
            }
            Token::Cpu => {
                self.advance();
                if self.current_token == Token::Dot {
                    self.advance();
                    match self.current_token {
                        Token::TimerBoost => {
                            self.advance();
                            Some(Statement::DirectAction(Action::CpuTimerBoost))
                        }
                        Token::TimerRestore => {
                            self.advance();
                            Some(Statement::DirectAction(Action::CpuTimerRestore))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Token::Log => {
                self.advance();
                if let Token::StringLit(msg) = self.advance() {
                    Some(Statement::DirectAction(Action::Log(msg)))
                } else {
                    None
                }
            }
            Token::When => {
                self.advance();
                self.parse_when_statement()
            }
            _ => None,
        }
    }

    fn parse_clean_action(&mut self) -> Option<Action> {
        let target = self.advance();
        if self.current_token == Token::Dot {
            self.advance();
            let sub = self.advance();
            match (&target, &sub) {
                (Token::Ram, Token::All) => Some(Action::CleanRamAll),
                (Token::Ram, Token::Standby) => Some(Action::CleanRamStandby),
                (Token::Ram, Token::Workingset) => Some(Action::CleanRamWorkingSet),
                (Token::Disk, Token::Temp) => Some(Action::CleanDiskTemp),
                (Token::Gpu, Token::Shader) => Some(Action::CleanGpuShader),
                _ => None,
            }
        } else {
            match target {
                Token::All => Some(Action::CleanRamAll),
                Token::Standby => Some(Action::CleanRamStandby),
                Token::Temp => Some(Action::CleanDiskTemp),
                _ => None,
            }
        }
    }

    fn parse_when_statement(&mut self) -> Option<Statement> {
        // Cú pháp: when ram.usage > 85% { ... }
        if self.advance() != Token::Ram {
            return None;
        }
        if self.advance() != Token::Dot {
            return None;
        }
        if self.advance() != Token::Usage {
            return None;
        }

        let is_gt = match self.advance() {
            Token::GreaterThan => true,
            Token::LessThan => false,
            _ => return None,
        };

        let threshold = match self.advance() {
            Token::Percent(p) | Token::Number(p) => p,
            _ => return None,
        };

        let condition = if is_gt {
            Condition::RamUsageGreaterThan(threshold)
        } else {
            Condition::RamUsageLessThan(threshold)
        };

        if self.advance() != Token::LBrace {
            return None;
        }

        let mut body = Vec::new();
        while self.current_token != Token::RBrace && self.current_token != Token::Eof {
            if let Some(Statement::DirectAction(act)) = self.parse_statement() {
                body.push(act);
            } else {
                self.advance();
            }
        }

        if self.current_token == Token::RBrace {
            self.advance();
        }

        Some(Statement::ConditionalRule { condition, body })
    }
}
