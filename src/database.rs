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
    /// List of strings - useful for inventories, tags, etc.
    /// 字符串列表 - 适用于物品栏、标签等。
    StringList(Vec<String>),
    /// List of integers - useful for HP values, stats arrays, etc.
    /// 整数列表 - 适用于 HP 值、属性数组等。
    IntList(Vec<i64>),
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

    /// Get the value as a string list, if it is one.
    ///
    /// 如果值是字符串列表，则获取该值。
    pub fn as_string_list(&self) -> Option<&[String]> {
        match self {
            FactValue::StringList(v) => Some(v),
            _ => None,
        }
    }

    /// Get the value as an integer list, if it is one.
    ///
    /// 如果值是整数列表，则获取该值。
    pub fn as_int_list(&self) -> Option<&[i64]> {
        match self {
            FactValue::IntList(v) => Some(v),
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

impl From<Vec<String>> for FactValue {
    fn from(v: Vec<String>) -> Self {
        FactValue::StringList(v)
    }
}

impl From<Vec<&str>> for FactValue {
    fn from(v: Vec<&str>) -> Self {
        FactValue::StringList(v.into_iter().map(|s| s.to_string()).collect())
    }
}

impl From<Vec<i64>> for FactValue {
    fn from(v: Vec<i64>) -> Self {
        FactValue::IntList(v)
    }
}

impl From<Vec<i32>> for FactValue {
    fn from(v: Vec<i32>) -> Self {
        FactValue::IntList(v.into_iter().map(|i| i as i64).collect())
    }
}

/// Trait for read-only fact database access.
/// Implemented by both `FactDatabase` and `LayeredFactDatabase`.
///
/// 事实数据库只读访问的 trait。
/// 由 `FactDatabase` 和 `LayeredFactDatabase` 实现。
pub trait FactReader {
    /// Get a fact value by key.
    fn get(&self, key: &FactKey) -> Option<&FactValue>;

    /// Get a fact value by string key.
    fn get_by_str(&self, key: &str) -> Option<&FactValue>;

    /// Get an integer fact value.
    fn get_int(&self, key: &str) -> Option<i64> {
        self.get_by_str(key).and_then(|v| v.as_int())
    }

    /// Get an integer fact value with a default.
    fn get_int_or(&self, key: &str, default: i64) -> i64 {
        self.get_int(key).unwrap_or(default)
    }

    /// Get a float fact value.
    fn get_float(&self, key: &str) -> Option<f64> {
        self.get_by_str(key).and_then(|v| v.as_float())
    }

    /// Get a boolean fact value.
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_by_str(key).and_then(|v| v.as_bool())
    }

    /// Get a string fact value.
    fn get_string(&self, key: &str) -> Option<&str> {
        self.get_by_str(key).and_then(|v| v.as_string())
    }

    /// Get a string list fact value.
    fn get_string_list(&self, key: &str) -> Option<&[String]> {
        self.get_by_str(key).and_then(|v| v.as_string_list())
    }

    /// Check if a fact exists.
    fn contains(&self, key: &str) -> bool;
}

/// Centralized database for storing facts (game state).
///
/// 用于存储事实（游戏状态）的集中式数据库。
#[derive(Resource, Default, Debug, Clone)]
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

impl FactReader for FactDatabase {
    fn get(&self, key: &FactKey) -> Option<&FactValue> {
        self.facts.get(key)
    }

    fn get_by_str(&self, key: &str) -> Option<&FactValue> {
        self.facts.get(&FactKey(key.to_string()))
    }

    fn contains(&self, key: &str) -> bool {
        self.facts.contains_key(&FactKey(key.to_string()))
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

    #[test]
    fn test_fact_value_type_accessors() {
        let int_val = FactValue::Int(42);
        let float_val = FactValue::Float(2.71);
        let bool_val = FactValue::Bool(true);
        let string_val = FactValue::String("test".to_string());

        // Test correct type accessors
        assert_eq!(int_val.as_int(), Some(42));
        assert_eq!(float_val.as_float(), Some(2.71));
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(string_val.as_string(), Some("test"));

        // Test wrong type accessors return None
        assert_eq!(int_val.as_float(), None);
        assert_eq!(float_val.as_int(), None);
        assert_eq!(bool_val.as_string(), None);
        assert_eq!(string_val.as_bool(), None);
    }

    #[test]
    fn test_fact_key_from_implementations() {
        let key_from_str: FactKey = "test_key".into();
        let key_from_string: FactKey = String::from("test_key").into();
        let key_new = FactKey::new("test_key");

        assert_eq!(key_from_str.0, "test_key");
        assert_eq!(key_from_string.0, "test_key");
        assert_eq!(key_new.0, "test_key");
        assert_eq!(key_from_str, key_from_string);
        assert_eq!(key_from_str, key_new);
    }

    #[test]
    fn test_fact_value_from_implementations() {
        let from_i64: FactValue = 42i64.into();
        let from_i32: FactValue = 42i32.into();
        let from_f64: FactValue = 2.71f64.into();
        let from_f32: FactValue = 2.71f32.into();
        let from_bool: FactValue = true.into();
        let from_string: FactValue = String::from("test").into();
        let from_str: FactValue = "test".into();

        assert_eq!(from_i64.as_int(), Some(42));
        assert_eq!(from_i32.as_int(), Some(42));
        assert!(from_f64.as_float().is_some());
        assert!(from_f32.as_float().is_some());
        assert_eq!(from_bool.as_bool(), Some(true));
        assert_eq!(from_string.as_string(), Some("test"));
        assert_eq!(from_str.as_string(), Some("test"));
    }

    #[test]
    fn test_fact_database_remove() {
        let mut db = FactDatabase::new();
        db.set("key", 100i64);
        assert!(db.contains("key"));

        let removed = db.remove("key");
        assert_eq!(removed, Some(FactValue::Int(100)));
        assert!(!db.contains("key"));

        // Remove non-existent key
        let removed_none = db.remove("nonexistent");
        assert_eq!(removed_none, None);
    }

    #[test]
    fn test_fact_database_clear() {
        let mut db = FactDatabase::new();
        db.set("key1", 1i64);
        db.set("key2", 2i64);
        db.set("key3", 3i64);
        assert_eq!(db.len(), 3);
        assert!(!db.is_empty());

        db.clear();
        assert_eq!(db.len(), 0);
        assert!(db.is_empty());
    }

    #[test]
    fn test_fact_database_iter() {
        let mut db = FactDatabase::new();
        db.set("a", 1i64);
        db.set("b", 2i64);
        db.set("c", 3i64);

        let count = db.iter().count();
        assert_eq!(count, 3);

        // Verify all keys are present
        let keys: Vec<_> = db.iter().map(|(k, _)| k.0.as_str()).collect();
        assert!(keys.contains(&"a"));
        assert!(keys.contains(&"b"));
        assert!(keys.contains(&"c"));
    }

    #[test]
    fn test_fact_database_get_int_or_default() {
        let mut db = FactDatabase::new();
        db.set("existing", 100i64);

        assert_eq!(db.get_int_or("existing", 0), 100);
        assert_eq!(db.get_int_or("missing", 42), 42);
    }

    #[test]
    fn test_fact_reader_trait() {
        let mut db = FactDatabase::new();
        db.set("health", 100i64);
        db.set("name", "Hero");
        db.set("alive", true);
        db.set("speed", 2.5f64);

        fn check_facts(reader: &impl FactReader) {
            assert_eq!(reader.get_int("health"), Some(100));
            assert_eq!(reader.get_int_or("health", 0), 100);
            assert_eq!(reader.get_int_or("missing", 50), 50);
            assert_eq!(reader.get_string("name"), Some("Hero"));
            assert_eq!(reader.get_bool("alive"), Some(true));
            assert_eq!(reader.get_float("speed"), Some(2.5));
            assert!(reader.contains("health"));
            assert!(!reader.contains("missing"));
        }

        check_facts(&db);
    }

    #[test]
    fn test_fact_database_overwrite() {
        let mut db = FactDatabase::new();
        db.set("key", 1i64);
        assert_eq!(db.get_int("key"), Some(1));

        db.set("key", 2i64);
        assert_eq!(db.get_int("key"), Some(2));

        // Can change type
        db.set("key", "string_value");
        assert_eq!(db.get_string("key"), Some("string_value"));
        assert_eq!(db.get_int("key"), None);
    }
}
