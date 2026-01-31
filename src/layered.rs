//! # layered.rs
//!
//! Layered Fact Database for scoped game state management.
//!
//! 分层事实数据库，用于作用域化的游戏状态管理。
//!
//! ## Architecture
//!
//! The layered database provides two tiers of storage:
//! - **Global Layer**: Persistent data across game states (e.g., player name, save progress)
//! - **Local Layer**: Temporary data for current context (e.g., battle turn count, room state)
//!
//! ## 架构
//!
//! 分层数据库提供两层存储：
//! - **全局层**: 跨游戏状态的持久数据（如玩家名称、存档进度）
//! - **局部层**: 当前上下文的临时数据（如战斗回合数、房间状态）

use crate::database::{FactDatabase, FactKey, FactReader, FactValue};
use bevy::prelude::*;

/// Layered fact database with global and local scopes.
///
/// 具有全局和局部作用域的分层事实数据库。
///
/// # Read Priority
/// When reading a fact, the local layer is checked first. If not found, the global layer is checked.
///
/// # 读取优先级
/// 读取事实时，首先检查局部层。如果未找到，则检查全局层。
///
/// # Write Behavior
/// - `set` / `set_local`: Write to local layer (default)
/// - `set_global`: Write to global layer (use sparingly)
///
/// # 写入行为
/// - `set` / `set_local`: 写入局部层（默认）
/// - `set_global`: 写入全局层（谨慎使用）
#[derive(Resource, Default, Debug)]
pub struct LayeredFactDatabase {
    /// Global layer: persistent data across game states.
    ///
    /// 全局层：跨游戏状态的持久数据。
    global: FactDatabase,

    /// Local layer: temporary data for current context.
    ///
    /// 局部层：当前上下文的临时数据。
    local: FactDatabase,
}

impl LayeredFactDatabase {
    /// Create a new empty layered fact database.
    ///
    /// 创建一个新的空分层事实数据库。
    pub fn new() -> Self {
        Self {
            global: FactDatabase::new(),
            local: FactDatabase::new(),
        }
    }

    // ========================================================================
    // Read Operations (Local-first, fallback to Global)
    // 读取操作（优先局部层，回退到全局层）
    // ========================================================================

    /// Get a fact value, checking local layer first, then global.
    ///
    /// 获取事实值，首先检查局部层，然后检查全局层。
    pub fn get(&self, key: &FactKey) -> Option<&FactValue> {
        self.local.get(key).or_else(|| self.global.get(key))
    }

    /// Get a fact value by string key.
    ///
    /// 通过字符串键获取事实值。
    pub fn get_by_str(&self, key: &str) -> Option<&FactValue> {
        self.local
            .get_by_str(key)
            .or_else(|| self.global.get_by_str(key))
    }

    /// Get an integer fact value.
    ///
    /// 获取整数事实值。
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
        // Need to check both layers manually for string references
        self.local
            .get_string(key)
            .or_else(|| self.global.get_string(key))
    }

    /// Check if a fact exists in either layer.
    ///
    /// 检查事实是否存在于任一层。
    pub fn contains(&self, key: &str) -> bool {
        self.local.contains(key) || self.global.contains(key)
    }

    /// Check if a fact exists in the local layer only.
    ///
    /// 检查事实是否仅存在于局部层。
    pub fn contains_local(&self, key: &str) -> bool {
        self.local.contains(key)
    }

    /// Check if a fact exists in the global layer only.
    ///
    /// 检查事实是否仅存在于全局层。
    pub fn contains_global(&self, key: &str) -> bool {
        self.global.contains(key)
    }

    // ========================================================================
    // Write Operations
    // 写入操作
    // ========================================================================

    /// Set a fact value in the local layer (default write target).
    ///
    /// 在局部层设置事实值（默认写入目标）。
    pub fn set(&mut self, key: impl Into<FactKey>, value: impl Into<FactValue>) {
        self.local.set(key, value);
    }

    /// Alias for `set` - explicitly writes to local layer.
    ///
    /// `set` 的别名 - 显式写入局部层。
    pub fn set_local(&mut self, key: impl Into<FactKey>, value: impl Into<FactValue>) {
        self.local.set(key, value);
    }

    /// Set a fact value in the global layer.
    /// Use sparingly - only for data that must persist across state transitions.
    ///
    /// 在全局层设置事实值。
    /// 谨慎使用 - 仅用于必须跨状态转换持久化的数据。
    pub fn set_global(&mut self, key: impl Into<FactKey>, value: impl Into<FactValue>) {
        self.global.set(key, value);
    }

    /// Increment an integer fact in the local layer.
    /// If the fact doesn't exist, it will be created with the increment value.
    ///
    /// 在局部层增加整数事实。
    /// 如果事实不存在，将使用增量值创建。
    pub fn increment(&mut self, key: &str, amount: i64) {
        let current = self.get_int(key).unwrap_or(0);
        self.local.set(key, current + amount);
    }

    /// Increment an integer fact in the global layer.
    ///
    /// 在全局层增加整数事实。
    pub fn increment_global(&mut self, key: &str, amount: i64) {
        let current = self.get_int(key).unwrap_or(0);
        self.global.set(key, current + amount);
    }

    /// Remove a fact from the local layer.
    ///
    /// 从局部层移除事实。
    pub fn remove(&mut self, key: &str) -> Option<FactValue> {
        self.local.remove(key)
    }

    /// Remove a fact from the global layer.
    ///
    /// 从全局层移除事实。
    pub fn remove_global(&mut self, key: &str) -> Option<FactValue> {
        self.global.remove(key)
    }

    // ========================================================================
    // Layer Management
    // 层管理
    // ========================================================================

    /// Clear all facts from the local layer.
    /// Call this when transitioning between game states.
    ///
    /// 清空局部层的所有事实。
    /// 在游戏状态转换时调用此方法。
    pub fn clear_local(&mut self) {
        self.local.clear();
    }

    /// Clear all facts from the global layer.
    /// Use with caution - this removes all persistent data.
    ///
    /// 清空全局层的所有事实。
    /// 谨慎使用 - 这将移除所有持久数据。
    pub fn clear_global(&mut self) {
        self.global.clear();
    }

    /// Clear both layers.
    ///
    /// 清空两层。
    pub fn clear_all(&mut self) {
        self.local.clear();
        self.global.clear();
    }

    /// Promote a fact from local layer to global layer.
    /// The fact is moved (removed from local, added to global).
    ///
    /// 将事实从局部层提升到全局层。
    /// 事实被移动（从局部层移除，添加到全局层）。
    pub fn promote_to_global(&mut self, key: &str) -> bool {
        if let Some(value) = self.local.remove(key) {
            self.global.set(key, value);
            true
        } else {
            false
        }
    }

    /// Copy a fact from local layer to global layer (keeping both copies).
    ///
    /// 将事实从局部层复制到全局层（保留两份副本）。
    pub fn copy_to_global(&mut self, key: &str) -> bool {
        if let Some(value) = self.local.get_by_str(key).cloned() {
            self.global.set(key, value);
            true
        } else {
            false
        }
    }

    /// Demote a fact from global layer to local layer.
    /// The fact is moved (removed from global, added to local).
    ///
    /// 将事实从全局层降级到局部层。
    /// 事实被移动（从全局层移除，添加到局部层）。
    pub fn demote_to_local(&mut self, key: &str) -> bool {
        if let Some(value) = self.global.remove(key) {
            self.local.set(key, value);
            true
        } else {
            false
        }
    }

    // ========================================================================
    // Direct Layer Access (for advanced use cases)
    // 直接层访问（用于高级用例）
    // ========================================================================

    /// Get immutable reference to the local layer.
    ///
    /// 获取局部层的不可变引用。
    pub fn local(&self) -> &FactDatabase {
        &self.local
    }

    /// Get mutable reference to the local layer.
    ///
    /// 获取局部层的可变引用。
    pub fn local_mut(&mut self) -> &mut FactDatabase {
        &mut self.local
    }

    /// Get immutable reference to the global layer.
    ///
    /// 获取全局层的不可变引用。
    pub fn global(&self) -> &FactDatabase {
        &self.global
    }

    /// Get mutable reference to the global layer.
    ///
    /// 获取全局层的可变引用。
    pub fn global_mut(&mut self) -> &mut FactDatabase {
        &mut self.global
    }

    // ========================================================================
    // Statistics
    // 统计信息
    // ========================================================================

    /// Get the total number of facts across both layers.
    ///
    /// 获取两层中事实的总数。
    pub fn len(&self) -> usize {
        self.local.len() + self.global.len()
    }

    /// Get the number of facts in the local layer.
    ///
    /// 获取局部层中事实的数量。
    pub fn local_len(&self) -> usize {
        self.local.len()
    }

    /// Get the number of facts in the global layer.
    ///
    /// 获取全局层中事实的数量。
    pub fn global_len(&self) -> usize {
        self.global.len()
    }

    /// Check if both layers are empty.
    ///
    /// 检查两层是否都为空。
    pub fn is_empty(&self) -> bool {
        self.local.is_empty() && self.global.is_empty()
    }
}

impl FactReader for LayeredFactDatabase {
    fn get(&self, key: &FactKey) -> Option<&FactValue> {
        self.local.get(key).or_else(|| self.global.get(key))
    }

    fn get_by_str(&self, key: &str) -> Option<&FactValue> {
        self.local
            .get_by_str(key)
            .or_else(|| self.global.get_by_str(key))
    }

    fn contains(&self, key: &str) -> bool {
        self.local.contains(key) || self.global.contains(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layered_read_priority() {
        let mut db = LayeredFactDatabase::new();

        // Set in global layer
        db.set_global("shared_key", 100i64);
        assert_eq!(db.get_int("shared_key"), Some(100));

        // Override in local layer - should take priority
        db.set_local("shared_key", 200i64);
        assert_eq!(db.get_int("shared_key"), Some(200));

        // Clear local - should fall back to global
        db.clear_local();
        assert_eq!(db.get_int("shared_key"), Some(100));
    }

    #[test]
    fn test_layer_isolation() {
        let mut db = LayeredFactDatabase::new();

        db.set_local("local_only", "local_value");
        db.set_global("global_only", "global_value");

        assert!(db.contains_local("local_only"));
        assert!(!db.contains_global("local_only"));

        assert!(db.contains_global("global_only"));
        assert!(!db.contains_local("global_only"));
    }

    #[test]
    fn test_promote_to_global() {
        let mut db = LayeredFactDatabase::new();

        db.set_local("temp_score", 42i64);
        assert!(db.contains_local("temp_score"));
        assert!(!db.contains_global("temp_score"));

        db.promote_to_global("temp_score");
        assert!(!db.contains_local("temp_score"));
        assert!(db.contains_global("temp_score"));
        assert_eq!(db.get_int("temp_score"), Some(42));
    }

    #[test]
    fn test_increment_across_layers() {
        let mut db = LayeredFactDatabase::new();

        // Start with global value
        db.set_global("counter", 10i64);

        // Increment writes to local layer
        db.increment("counter", 5);

        // Local layer now has 15, global still has 10
        assert_eq!(db.get_int("counter"), Some(15)); // Local takes priority
        assert_eq!(db.global().get_int("counter"), Some(10)); // Global unchanged
    }

    #[test]
    fn test_copy_to_global() {
        let mut db = LayeredFactDatabase::new();

        db.set_local("data", "important");
        assert!(db.copy_to_global("data"));

        // Both layers now have the value
        assert!(db.contains_local("data"));
        assert!(db.contains_global("data"));
        assert_eq!(db.local().get_string("data"), Some("important"));
        assert_eq!(db.global().get_string("data"), Some("important"));
    }

    #[test]
    fn test_copy_to_global_nonexistent() {
        let mut db = LayeredFactDatabase::new();
        assert!(!db.copy_to_global("nonexistent"));
    }

    #[test]
    fn test_demote_to_local() {
        let mut db = LayeredFactDatabase::new();

        db.set_global("global_data", 100i64);
        assert!(db.demote_to_local("global_data"));

        assert!(!db.contains_global("global_data"));
        assert!(db.contains_local("global_data"));
        assert_eq!(db.get_int("global_data"), Some(100));
    }

    #[test]
    fn test_demote_to_local_nonexistent() {
        let mut db = LayeredFactDatabase::new();
        assert!(!db.demote_to_local("nonexistent"));
    }

    #[test]
    fn test_promote_to_global_nonexistent() {
        let mut db = LayeredFactDatabase::new();
        assert!(!db.promote_to_global("nonexistent"));
    }

    #[test]
    fn test_remove_operations() {
        let mut db = LayeredFactDatabase::new();

        db.set_local("local_key", 1i64);
        db.set_global("global_key", 2i64);

        // Remove from local
        let removed_local = db.remove("local_key");
        assert_eq!(removed_local, Some(FactValue::Int(1)));
        assert!(!db.contains_local("local_key"));

        // Remove from global
        let removed_global = db.remove_global("global_key");
        assert_eq!(removed_global, Some(FactValue::Int(2)));
        assert!(!db.contains_global("global_key"));
    }

    #[test]
    fn test_clear_all() {
        let mut db = LayeredFactDatabase::new();

        db.set_local("local", 1i64);
        db.set_global("global", 2i64);
        assert!(!db.is_empty());

        db.clear_all();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
        assert_eq!(db.local_len(), 0);
        assert_eq!(db.global_len(), 0);
    }

    #[test]
    fn test_len_operations() {
        let mut db = LayeredFactDatabase::new();
        assert_eq!(db.len(), 0);
        assert_eq!(db.local_len(), 0);
        assert_eq!(db.global_len(), 0);
        assert!(db.is_empty());

        db.set_local("l1", 1i64);
        db.set_local("l2", 2i64);
        db.set_global("g1", 3i64);

        assert_eq!(db.local_len(), 2);
        assert_eq!(db.global_len(), 1);
        assert_eq!(db.len(), 3);
        assert!(!db.is_empty());
    }

    #[test]
    fn test_get_typed_values() {
        let mut db = LayeredFactDatabase::new();

        db.set_local("int_val", 42i64);
        db.set_local("float_val", 3.14f64);
        db.set_local("bool_val", true);
        db.set_local("str_val", "hello");

        assert_eq!(db.get_int("int_val"), Some(42));
        assert_eq!(db.get_float("float_val"), Some(3.14));
        assert_eq!(db.get_bool("bool_val"), Some(true));
        assert_eq!(db.get_string("str_val"), Some("hello"));

        // Test defaults
        assert_eq!(db.get_int_or("int_val", 0), 42);
        assert_eq!(db.get_int_or("missing", 100), 100);
    }

    #[test]
    fn test_contains_both_layers() {
        let mut db = LayeredFactDatabase::new();

        db.set_local("local", 1i64);
        db.set_global("global", 2i64);

        // contains() checks both layers
        assert!(db.contains("local"));
        assert!(db.contains("global"));
        assert!(!db.contains("missing"));
    }

    #[test]
    fn test_get_by_fact_key() {
        let mut db = LayeredFactDatabase::new();

        db.set_local("test_key", 42i64);
        let key = FactKey::new("test_key");

        assert_eq!(db.get(&key), Some(&FactValue::Int(42)));
    }

    #[test]
    fn test_increment_global() {
        let mut db = LayeredFactDatabase::new();

        db.set_global("global_counter", 10i64);
        db.increment_global("global_counter", 5);

        assert_eq!(db.global().get_int("global_counter"), Some(15));
    }

    #[test]
    fn test_increment_creates_if_missing() {
        let mut db = LayeredFactDatabase::new();

        // Should create with the increment value
        db.increment("new_counter", 10);
        assert_eq!(db.get_int("new_counter"), Some(10));

        db.increment_global("new_global_counter", 20);
        assert_eq!(db.global().get_int("new_global_counter"), Some(20));
    }

    #[test]
    fn test_direct_layer_access() {
        let mut db = LayeredFactDatabase::new();

        // Access local layer directly
        db.local_mut().set("direct_local", 1i64);
        assert_eq!(db.local().get_int("direct_local"), Some(1));

        // Access global layer directly
        db.global_mut().set("direct_global", 2i64);
        assert_eq!(db.global().get_int("direct_global"), Some(2));
    }

    #[test]
    fn test_fact_reader_trait_impl() {
        let mut db = LayeredFactDatabase::new();
        db.set_global("global_fact", 100i64);
        db.set_local("local_fact", 200i64);

        fn check_reader(reader: &impl FactReader) {
            assert!(reader.contains("global_fact"));
            assert!(reader.contains("local_fact"));
            assert!(!reader.contains("missing"));
            assert_eq!(reader.get_int("global_fact"), Some(100));
            assert_eq!(reader.get_int("local_fact"), Some(200));
        }

        check_reader(&db);
    }

    #[test]
    fn test_string_fallback_to_global() {
        let mut db = LayeredFactDatabase::new();

        db.set_global("player_name", "GlobalPlayer");
        assert_eq!(db.get_string("player_name"), Some("GlobalPlayer"));

        // Override with local
        db.set_local("player_name", "LocalPlayer");
        assert_eq!(db.get_string("player_name"), Some("LocalPlayer"));

        // Clear local, should fallback
        db.clear_local();
        assert_eq!(db.get_string("player_name"), Some("GlobalPlayer"));
    }
}
