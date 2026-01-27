//! # database.rs
//!
//! Centralized fact database for storing game state as key-value pairs.
//!
//! 集中式事实数据库，用于将游戏状态存储为键值对。

use bevy::prelude::*;
use std::collections::HashMap;

/// Unique identifier for a fact in the database.
///
/// 数据库中事实的唯一标识符。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactKey(pub String);

impl FactKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

impl From<&str> for FactKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for FactKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Value types supported by the fact database.
///
/// 事实数据库支持的值类型。
#[derive(Debug, Clone, PartialEq)]
pub enum FactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl FactValue {
    /// Get the value as an integer, if it is one.
    ///
    /// 如果值是整数，则获取该值。
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FactValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as a float, if it is one.
    ///
    /// 如果值是浮点数，则获取该值。
    pub fn as_float(&self) -> Option<f64> {
        match self {
            FactValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as a boolean, if it is one.
    ///
    /// 如果值是布尔值，则获取该值。
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FactValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as a string, if it is one.
    ///
    /// 如果值是字符串，则获取该值。
    pub fn as_string(&self) -> Option<&str> {
        match self {
            FactValue::String(v) => Some(v),
            _ => None,
        }
    }
}

impl From<i64> for FactValue {
    fn from(v: i64) -> Self {
        FactValue::Int(v)
    }
}

impl From<i32> for FactValue {
    fn from(v: i32) -> Self {
        FactValue::Int(v as i64)
    }
}

impl From<f64> for FactValue {
    fn from(v: f64) -> Self {
        FactValue::Float(v)
    }
}

impl From<f32> for FactValue {
    fn from(v: f32) -> Self {
        FactValue::Float(v as f64)
    }
}

impl From<bool> for FactValue {
    fn from(v: bool) -> Self {
        FactValue::Bool(v)
    }
}

impl From<String> for FactValue {
    fn from(v: String) -> Self {
        FactValue::String(v)
    }
}

impl From<&str> for FactValue {
    fn from(v: &str) -> Self {
        FactValue::String(v.to_string())
    }
}

/// Centralized database for storing facts (game state).
///
/// 用于存储事实（游戏状态）的集中式数据库。
#[derive(Resource, Default, Debug)]
pub struct FactDatabase {
    facts: HashMap<FactKey, FactValue>,
}

impl FactDatabase {
    /// Create a new empty fact database.
    ///
    /// 创建一个新的空事实数据库。
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }

    /// Set a fact value in the database.
    ///
    /// 在数据库中设置一个事实值。
    pub fn set(&mut self, key: impl Into<FactKey>, value: impl Into<FactValue>) {
        self.facts.insert(key.into(), value.into());
    }

    /// Get a fact value from the database.
    ///
    /// 从数据库中获取一个事实值。
    pub fn get(&self, key: &FactKey) -> Option<&FactValue> {
        self.facts.get(key)
    }

    /// Get a fact value by string key.
    ///
    /// 通过字符串键获取事实值。
    pub fn get_by_str(&self, key: &str) -> Option<&FactValue> {
        self.facts.get(&FactKey(key.to_string()))
    }

    /// Get an integer fact value, returning a default if not found or wrong type.
    ///
    /// 获取整数事实值，如果未找到或类型错误则返回默认值。
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get_by_str(key).and_then(|v| v.as_int())
    }

    /// Get an integer fact value with a default.
    ///
    /// 获取整数事实值，带有默认值。
    pub fn get_int_or(&self, key: &str, default: i64) -> i64 {
        self.get_int(key).unwrap_or(default)
    }

    /// Get a float fact value.
    ///
    /// 获取浮点数事实值。
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.get_by_str(key).and_then(|v| v.as_float())
    }

    /// Get a boolean fact value.
    ///
    /// 获取布尔事实值。
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_by_str(key).and_then(|v| v.as_bool())
    }

    /// Get a string fact value.
    ///
    /// 获取字符串事实值。
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get_by_str(key).and_then(|v| v.as_string())
    }

    /// Check if a fact exists in the database.
    ///
    /// 检查数据库中是否存在某个事实。
    pub fn contains(&self, key: &str) -> bool {
        self.facts.contains_key(&FactKey(key.to_string()))
    }

    /// Remove a fact from the database.
    ///
    /// 从数据库中移除一个事实。
    pub fn remove(&mut self, key: &str) -> Option<FactValue> {
        self.facts.remove(&FactKey(key.to_string()))
    }

    /// Increment an integer fact by a given amount.
    /// If the fact doesn't exist, it will be created with the increment value.
    ///
    /// 将整数事实增加指定的量。
    /// 如果事实不存在，将使用增量值创建。
    pub fn increment(&mut self, key: &str, amount: i64) {
        let current = self.get_int(key).unwrap_or(0);
        self.set(key, current + amount);
    }

    /// Get all facts as an iterator.
    ///
    /// 获取所有事实的迭代器。
    pub fn iter(&self) -> impl Iterator<Item = (&FactKey, &FactValue)> {
        self.facts.iter()
    }

    /// Get the number of facts in the database.
    ///
    /// 获取数据库中事实的数量。
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Check if the database is empty.
    ///
    /// 检查数据库是否为空。
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Clear all facts from the database.
    ///
    /// 清除数据库中的所有事实。
    pub fn clear(&mut self) {
        self.facts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_database_basic_operations() {
        let mut db = FactDatabase::new();

        db.set("health", 100i64);
        db.set("name", "Player");
        db.set("alive", true);
        db.set("speed", 1.5f64);

        assert_eq!(db.get_int("health"), Some(100));
        assert_eq!(db.get_string("name"), Some("Player"));
        assert_eq!(db.get_bool("alive"), Some(true));
        assert_eq!(db.get_float("speed"), Some(1.5));
    }

    #[test]
    fn test_fact_database_increment() {
        let mut db = FactDatabase::new();

        db.increment("counter", 1);
        assert_eq!(db.get_int("counter"), Some(1));

        db.increment("counter", 5);
        assert_eq!(db.get_int("counter"), Some(6));
    }
}
