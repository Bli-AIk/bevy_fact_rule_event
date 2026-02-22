//! # expr.rs
//!
//! Simple expression evaluation for FRE modifications.
//! Supports arithmetic operations on fact values.
//!
//! FRE 修改器的简单表达式求值。
//! 支持对 fact 值进行算术运算。

use crate::database::FactValue;
use crate::layered::LayeredFactDatabase;

/// Evaluate a simple arithmetic expression.
///
/// 评估简单的算术表达式。
///
/// Supported syntax:
/// - `$key` - Reference to a fact value
/// - Numbers (integers and floats)
/// - Operators: `+`, `-`, `*`, `/`, `%`
/// - Parentheses for grouping
///
/// 支持的语法：
/// - `$key` - 引用 fact 值
/// - 数字（整数和浮点数）
/// - 运算符：`+`、`-`、`*`、`/`、`%`
/// - 括号用于分组
///
/// Returns the result as f64, or None if evaluation fails.
pub fn evaluate_expr(expr: &str, db: &LayeredFactDatabase) -> Option<f64> {
    let expr = expr.trim();
    if expr.is_empty() {
        return None;
    }

    // Tokenize and parse the expression
    let tokens = tokenize(expr, db)?;
    parse_expr(&tokens, 0).map(|(result, _)| result)
}

/// Evaluate an expression and return as FactValue.
///
/// 评估表达式并返回为 FactValue。
pub fn evaluate_expr_to_fact(expr: &str, db: &LayeredFactDatabase) -> Option<FactValue> {
    let result = evaluate_expr(expr, db)?;

    // Return as Int if the result is a whole number, otherwise Float
    if result.fract() == 0.0 && result.abs() < i64::MAX as f64 {
        Some(FactValue::Int(result as i64))
    } else {
        Some(FactValue::Float(result))
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Op(char),
    LParen,
    RParen,
}

/// Tokenize an expression string, resolving $variables to their values.
fn tokenize(expr: &str, db: &LayeredFactDatabase) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if c == '$' {
            // Variable reference: $key or $namespace:key
            i += 1;
            let start = i;
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == ':')
            {
                i += 1;
            }
            let key = &expr[start..i];

            // Look up the value in the database
            let value = match db.get_by_str(key) {
                Some(FactValue::Int(v)) => *v as f64,
                Some(FactValue::Float(v)) => *v,
                Some(FactValue::Bool(v)) => {
                    if *v {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => {
                    // Unknown variable, return None
                    return None;
                }
            };
            tokens.push(Token::Number(value));
            continue;
        }

        if c.is_ascii_digit()
            || (c == '-'
                && i + 1 < chars.len()
                && chars[i + 1].is_ascii_digit()
                && (tokens.is_empty()
                    || matches!(tokens.last(), Some(Token::Op(_)) | Some(Token::LParen))))
        {
            // Number literal
            let start = i;
            if c == '-' {
                i += 1;
            }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let num_str = &expr[start..i];
            let num: f64 = num_str.parse().ok()?;
            tokens.push(Token::Number(num));
            continue;
        }

        match c {
            '+' | '-' | '*' | '/' | '%' => {
                // For '-', check if it's a unary minus (negation)
                if c == '-'
                    && (tokens.is_empty()
                        || matches!(tokens.last(), Some(Token::Op(_)) | Some(Token::LParen)))
                {
                    // Parse the number including the minus sign
                    let start = i;
                    i += 1;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    // Check if we actually got digits after the minus
                    if i > start + 1 {
                        let num_str = &expr[start..i];
                        let num: f64 = num_str.parse().ok()?;
                        tokens.push(Token::Number(num));
                        continue;
                    } else {
                        // It's just a minus sign, treat as operator
                        i = start;
                    }
                }
                tokens.push(Token::Op(c));
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            _ => {
                // Unknown character
                return None;
            }
        }
    }

    Some(tokens)
}

/// Parse expression with operator precedence.
/// Returns (result, next_index).
fn parse_expr(tokens: &[Token], start: usize) -> Option<(f64, usize)> {
    parse_additive(tokens, start)
}

fn parse_additive(tokens: &[Token], start: usize) -> Option<(f64, usize)> {
    let (mut left, mut idx) = parse_multiplicative(tokens, start)?;

    while idx < tokens.len() {
        match &tokens[idx] {
            Token::Op('+') => {
                let (right, next) = parse_multiplicative(tokens, idx + 1)?;
                left += right;
                idx = next;
            }
            Token::Op('-') => {
                let (right, next) = parse_multiplicative(tokens, idx + 1)?;
                left -= right;
                idx = next;
            }
            _ => break,
        }
    }

    Some((left, idx))
}

fn parse_multiplicative(tokens: &[Token], start: usize) -> Option<(f64, usize)> {
    let (mut left, mut idx) = parse_primary(tokens, start)?;

    while idx < tokens.len() {
        match &tokens[idx] {
            Token::Op('*') => {
                let (right, next) = parse_primary(tokens, idx + 1)?;
                left *= right;
                idx = next;
            }
            Token::Op('/') => {
                let (right, next) = parse_primary(tokens, idx + 1)?;
                if right != 0.0 {
                    left /= right;
                } else {
                    left = 0.0; // Division by zero = 0
                }
                idx = next;
            }
            Token::Op('%') => {
                let (right, next) = parse_primary(tokens, idx + 1)?;
                if right != 0.0 {
                    left %= right;
                } else {
                    left = 0.0; // Mod by zero = 0
                }
                idx = next;
            }
            _ => break,
        }
    }

    Some((left, idx))
}

fn parse_primary(tokens: &[Token], start: usize) -> Option<(f64, usize)> {
    if start >= tokens.len() {
        return None;
    }

    match &tokens[start] {
        Token::Number(n) => Some((*n, start + 1)),
        Token::LParen => {
            let (result, idx) = parse_expr(tokens, start + 1)?;
            // Expect closing paren
            if idx < tokens.len() && matches!(&tokens[idx], Token::RParen) {
                Some((result, idx + 1))
            } else {
                None // Missing closing paren
            }
        }
        Token::Op('-') => {
            // Unary minus
            let (val, idx) = parse_primary(tokens, start + 1)?;
            Some((-val, idx))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_number() {
        let db = LayeredFactDatabase::default();
        assert_eq!(evaluate_expr("42", &db), Some(42.0));
        assert_eq!(evaluate_expr("3.14", &db), Some(3.14));
        assert_eq!(evaluate_expr("-5", &db), Some(-5.0));
    }

    #[test]
    fn test_arithmetic() {
        let db = LayeredFactDatabase::default();
        assert_eq!(evaluate_expr("1 + 2", &db), Some(3.0));
        assert_eq!(evaluate_expr("10 - 3", &db), Some(7.0));
        assert_eq!(evaluate_expr("4 * 5", &db), Some(20.0));
        assert_eq!(evaluate_expr("20 / 4", &db), Some(5.0));
        assert_eq!(evaluate_expr("10 % 3", &db), Some(1.0));
    }

    #[test]
    fn test_precedence() {
        let db = LayeredFactDatabase::default();
        assert_eq!(evaluate_expr("2 + 3 * 4", &db), Some(14.0));
        assert_eq!(evaluate_expr("(2 + 3) * 4", &db), Some(20.0));
    }

    #[test]
    fn test_variable() {
        let mut db = LayeredFactDatabase::default();
        db.set_local("x", 10i64);
        db.set_local("y", 5i64);

        assert_eq!(evaluate_expr("$x", &db), Some(10.0));
        assert_eq!(evaluate_expr("$x + $y", &db), Some(15.0));
        assert_eq!(evaluate_expr("$x * 2 + $y", &db), Some(25.0));
    }

    #[test]
    fn test_namespaced_variable() {
        let mut db = LayeredFactDatabase::default();
        db.set_local("menu:selection", 3i64);

        assert_eq!(evaluate_expr("$menu:selection", &db), Some(3.0));
        assert_eq!(evaluate_expr("$menu:selection - 1", &db), Some(2.0));
    }
}
