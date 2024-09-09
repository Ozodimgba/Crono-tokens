use anchor_lang::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Token {
    Number(f64),
    Variable(String),
    Operator(char),
    Function(String),
    LeftParen,
    RightParen,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    variables: HashMap<String, f64>,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self> {
        let tokens = Self::tokenize(input)?;
        Ok(Self {
            tokens,
            variables: HashMap::new(),
        })
    }

    fn tokenize(input: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '0'..='9' | '.' => {
                    let mut num = String::new();
                    num.push(ch);
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_digit(10) || next_ch == '.' {
                            num.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num.parse().map_err(|_| error!(ErrorCode::InvalidNumber))?));
                },
                '+' | '-' | '*' | '/' | '^' => tokens.push(Token::Operator(ch)),
                '(' => tokens.push(Token::LeftParen),
                ')' => tokens.push(Token::RightParen),
                'a'..='z' | 'A'..='Z' => {
                    let mut name = String::new();
                    name.push(ch);
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() {
                            name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if chars.peek() == Some(&'(') {
                        tokens.push(Token::Function(name));
                    } else {
                        tokens.push(Token::Variable(name));
                    }
                },
                ' ' => {},
                _ => return Err(error!(ErrorCode::InvalidCharacter)),
            }
        }
        Ok(tokens)
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn evaluate(&self) -> Result<f64> {
        let mut output_queue = Vec::new();
        let mut operator_stack = Vec::new();

        for token in &self.tokens {
            match token {
                Token::Number(_) | Token::Variable(_) => output_queue.push(token.clone()),
                Token::Function(_) => operator_stack.push(token.clone()),
                Token::Operator(op) => {
                    while let Some(Token::Operator(top)) = operator_stack.last() {
                        if Self::precedence(*top) >= Self::precedence(*op) {
                            output_queue.push(operator_stack.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                    operator_stack.push(token.clone());
                },
                Token::LeftParen => operator_stack.push(token.clone()),
                Token::RightParen => {
                    while let Some(top) = operator_stack.pop() {
                        if let Token::LeftParen = top {
                            break;
                        }
                        output_queue.push(top);
                    }
                    if let Some(Token::Function(_)) = operator_stack.last() {
                        output_queue.push(operator_stack.pop().unwrap());
                    }
                },
            }
        }

        while let Some(op) = operator_stack.pop() {
            output_queue.push(op);
        }

        let mut stack = Vec::new();
        for token in output_queue {
            match token {
                Token::Number(num) => stack.push(num),
                Token::Variable(name) => {
                    let value = self.variables.get(&name).ok_or(error!(ErrorCode::UndefinedVariable))?;
                    stack.push(*value);
                },
                Token::Operator(op) => {
                    let b = stack.pop().ok_or(error!(ErrorCode::InvalidExpression))?;
                    let a = stack.pop().ok_or(error!(ErrorCode::InvalidExpression))?;
                    let result = match op {
                        '+' => a + b,
                        '-' => a - b,
                        '*' => a * b,
                        '/' => a / b,
                        '^' => a.powf(b),
                        _ => return Err(error!(ErrorCode::UnsupportedOperator)),
                    };
                    stack.push(result);
                },
                Token::Function(name) => {
                    let arg = stack.pop().ok_or(error!(ErrorCode::InvalidExpression))?;
                    let result = match name.as_str() {
                        "exp" => arg.exp(),
                        "ln" => arg.ln(),
                        "sin" => arg.sin(),
                        "cos" => arg.cos(),
                        _ => return Err(error!(ErrorCode::UnsupportedFunction)),
                    };
                    stack.push(result);
                },
                _ => return Err(error!(ErrorCode::InvalidExpression)),
            }
        }

        stack.pop().ok_or(error!(ErrorCode::InvalidExpression))
    }

    fn precedence(op: char) -> u8 {
        match op {
            '+' | '-' => 1,
            '*' | '/' => 2,
            '^' => 3,
            _ => 0,
        }
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid number in expression")]
    InvalidNumber,
    #[msg("Invalid character in expression")]
    InvalidCharacter,
    #[msg("Undefined variable")]
    UndefinedVariable,
    #[msg("Invalid expression")]
    InvalidExpression,
    #[msg("Unsupported operator")]
    UnsupportedOperator,
    #[msg("Unsupported function")]
    UnsupportedFunction,
}