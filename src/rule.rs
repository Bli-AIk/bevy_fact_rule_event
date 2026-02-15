//! # rule.rs
//!
//! Rule definitions - the logic layer of FRE.
//! Rules contain triggers, conditions (expressions), modifications, and outputs.
//!
//! 规则定义 - FRE 的逻辑层。
//! 规则包含触发器、条件（表达式）、修改和输出。

use crate::database::FactValue;
use crate::event::{FactEvent, FactEventId};
use crate::layered::LayeredFactDatabase;
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

/// Rule scope - determines the lifetime and isolation of rules.
///
/// 规则作用域 - 决定规则的生命周期和隔离性。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Deserialize, serde::Serialize,
)]
pub enum RuleScope {
    /// Global rules - persist for the entire application lifetime.
    /// Examples: pause menu, debug commands, achievement triggers.
    ///
    /// 全局规则 - 在整个应用生命周期内持续存在。
    /// 示例：暂停菜单、调试命令、成就触发。
    Global,

    /// Local rules - scoped to the current scene/state.
    /// Automatically cleared when exiting the scene.
    /// Examples: room-specific interactions, battle rules.
    ///
    /// 局部规则 - 限定于当前场景/状态。
    /// 退出场景时自动清除。
    /// 示例：房间特定交互、战斗规则。
    #[default]
    Local,

    /// View rules - scoped to a specific View entity.
    /// Automatically cleared when the View is despawned.
    /// Examples: UI navigation within a specific view.
    ///
    /// 视图规则 - 限定于特定 View 实体。
    /// View 被销毁时自动清除。
    /// 示例：特定视图内的 UI 导航。
    View,
}

/// Modification to apply to the fact database.
///
/// 应用于事实数据库的修改。
#[derive(Clone, Debug)]
pub enum FactModification {
    /// Set a fact to a specific value.
    ///
    /// 将事实设置为特定值。
    Set(String, FactValue),

    /// Increment an integer fact by a value.
    ///
    /// 将整数事实增加指定值。
    Increment(String, i64),

    /// Remove a fact.
    ///
    /// 移除一个事实。
    Remove(String),

    /// Toggle a boolean fact.
    ///
    /// 切换布尔事实。
    Toggle(String),
}

impl FactModification {
    /// Apply the modification to the layered fact database (local layer by default).
    ///
    /// 将修改应用于分层事实数据库（默认为局部层）。
    pub fn apply(&self, db: &mut LayeredFactDatabase) {
        match self {
            FactModification::Set(key, value) => {
                db.set_local(key.as_str(), value.clone());
            }
            FactModification::Increment(key, amount) => {
                db.increment(key, *amount);
            }
            FactModification::Remove(key) => {
                db.remove(key);
            }
            FactModification::Toggle(key) => {
                let current = db.get_bool(key).unwrap_or(false);
                db.set_local(key.as_str(), !current);
            }
        }
    }
}

/// A rule definition containing trigger, conditions (expressions), modifications, and outputs.
///
/// 包含触发器、条件（表达式）、修改和输出的规则定义。
#[derive(Clone)]
pub struct Rule {
    /// Unique identifier for this rule.
    ///
    /// 此规则的唯一标识符。
    pub id: String,

    /// Scope of this rule (Global/Local/View).
    ///
    /// 此规则的作用域（Global/Local/View）。
    pub scope: RuleScope,

    /// Event ID that triggers this rule.
    ///
    /// 触发此规则的事件 ID。
    pub trigger: FactEventId,

    /// Expression-based conditions (list of expression strings).
    /// All expressions must evaluate to true for the rule to fire.
    /// These are evaluated by the game engine's expression evaluator.
    ///
    /// 基于表达式的条件（表达式字符串列表）。
    /// 所有表达式都必须评估为真才能触发规则。
    /// 这些由游戏引擎的表达式评估器评估。
    pub condition_expressions: Vec<String>,

    /// Modifications to apply to the fact database.
    ///
    /// 应用于事实数据库的修改。
    pub modifications: Vec<FactModification>,

    /// Events to emit after rule execution.
    ///
    /// 规则执行后要发出的事件。
    pub outputs: Vec<FactEventId>,

    /// Whether this rule is enabled.
    ///
    /// 此规则是否启用。
    pub enabled: bool,

    /// Priority for rule ordering (higher = first, rules are grouped by priority).
    ///
    /// 规则排序的优先级（越高越先，规则按优先级分组）。
    pub priority: i32,

    /// Whether this rule consumes the event after execution.
    /// If true (default), no other rules in lower priority groups will be checked.
    /// If false, continue checking rules within the same priority group.
    ///
    /// 此规则执行后是否消费事件。
    /// 如果为 true（默认），将不检查更低优先级组的规则。
    /// 如果为 false，继续检查同一优先级组内的规则。
    pub consume_event: bool,
}

impl Rule {
    /// Create a new rule builder.
    ///
    /// 创建新的规则构建器。
    pub fn builder(id: impl Into<String>, trigger: impl Into<FactEventId>) -> RuleBuilder {
        RuleBuilder::new(id, trigger)
    }

    /// Check if this rule should trigger for the given event.
    ///
    /// 检查此规则是否应该为给定事件触发。
    pub fn matches_event(&self, event: &FactEvent) -> bool {
        self.enabled && self.trigger == event.id
    }
}

/// Builder for constructing rules.
///
/// 用于构建规则的构建器。
pub struct RuleBuilder {
    id: String,
    scope: RuleScope,
    trigger: FactEventId,
    condition_expressions: Vec<String>,
    modifications: Vec<FactModification>,
    outputs: Vec<FactEventId>,
    enabled: bool,
    priority: i32,
    consume_event: bool,
}

impl RuleBuilder {
    /// Create a new rule builder.
    ///
    /// 创建新的规则构建器。
    pub fn new(id: impl Into<String>, trigger: impl Into<FactEventId>) -> Self {
        Self {
            id: id.into(),
            scope: RuleScope::default(),
            trigger: trigger.into(),
            condition_expressions: Vec::new(),
            modifications: Vec::new(),
            outputs: Vec::new(),
            enabled: true,
            priority: 0,
            consume_event: true,
        }
    }

    /// Set the scope for this rule.
    ///
    /// 设置此规则的作用域。
    pub fn scope(mut self, scope: RuleScope) -> Self {
        self.scope = scope;
        self
    }

    /// Add a condition expression to this rule.
    ///
    /// 向此规则添加条件表达式。
    pub fn condition_expr(mut self, expr: impl Into<String>) -> Self {
        self.condition_expressions.push(expr.into());
        self
    }

    /// Add a modification to this rule.
    ///
    /// 向此规则添加修改。
    pub fn modify(mut self, modification: FactModification) -> Self {
        self.modifications.push(modification);
        self
    }

    /// Add an output event to this rule.
    ///
    /// 向此规则添加输出事件。
    pub fn output(mut self, event_id: impl Into<FactEventId>) -> Self {
        self.outputs.push(event_id.into());
        self
    }

    /// Set the priority of this rule.
    ///
    /// 设置此规则的优先级。
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set whether this rule is enabled.
    ///
    /// 设置此规则是否启用。
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether this rule consumes the event.
    ///
    /// 设置此规则是否消费事件。
    pub fn consume_event(mut self, consume: bool) -> Self {
        self.consume_event = consume;
        self
    }

    /// Build the rule.
    ///
    /// 构建规则。
    pub fn build(self) -> Rule {
        Rule {
            id: self.id,
            scope: self.scope,
            trigger: self.trigger,
            condition_expressions: self.condition_expressions,
            modifications: self.modifications,
            outputs: self.outputs,
            enabled: self.enabled,
            priority: self.priority,
            consume_event: self.consume_event,
        }
    }
}

/// Registry for storing and managing rules.
///
/// 用于存储和管理规则的注册表。
#[derive(Resource, Default)]
pub struct RuleRegistry {
    rules: HashMap<String, Rule>,
    /// Rules sorted by priority (cached).
    ///
    /// 按优先级排序的规则（缓存）。
    sorted_rules: Vec<String>,
    dirty: bool,
}

impl RuleRegistry {
    /// Create a new empty rule registry.
    ///
    /// 创建新的空规则注册表。
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            sorted_rules: Vec::new(),
            dirty: false,
        }
    }

    /// Register a new rule.
    ///
    /// 注册新规则。
    pub fn register(&mut self, rule: Rule) {
        self.rules.insert(rule.id.clone(), rule);
        self.dirty = true;
    }

    /// Unregister a rule by ID.
    ///
    /// 按 ID 注销规则。
    pub fn unregister(&mut self, rule_id: &str) -> Option<Rule> {
        let rule = self.rules.remove(rule_id);
        if rule.is_some() {
            self.dirty = true;
        }
        rule
    }

    /// Get a rule by ID.
    ///
    /// 按 ID 获取规则。
    pub fn get(&self, rule_id: &str) -> Option<&Rule> {
        self.rules.get(rule_id)
    }

    /// Get a mutable reference to a rule by ID.
    ///
    /// 按 ID 获取规则的可变引用。
    pub fn get_mut(&mut self, rule_id: &str) -> Option<&mut Rule> {
        self.rules.get_mut(rule_id)
    }

    /// Enable or disable a rule.
    ///
    /// 启用或禁用规则。
    pub fn set_enabled(&mut self, rule_id: &str, enabled: bool) {
        if let Some(rule) = self.rules.get_mut(rule_id) {
            rule.enabled = enabled;
        }
    }

    /// Get all rules that match a given event, grouped by priority and sorted by condition count.
    /// Returns groups from highest to lowest priority.
    /// Within each group, rules are sorted by condition count (fewer conditions first).
    ///
    /// 获取匹配给定事件的所有规则，按优先级分组并按条件数量排序。
    /// 返回从高到低优先级的组。
    /// 在每个组内，规则按条件数量排序（条件少的在前）。
    pub fn get_matching_rules_grouped(&self, event: &FactEvent) -> Vec<Vec<&Rule>> {
        // Group matching rules by priority
        let mut groups: BTreeMap<i32, Vec<&Rule>> = BTreeMap::new();

        for rule in self.rules.values() {
            if rule.matches_event(event) {
                groups.entry(rule.priority).or_default().push(rule);
            }
        }

        // Sort each group by condition count (fewer conditions first)
        for group in groups.values_mut() {
            group.sort_by_key(|r| r.condition_expressions.len());
        }

        // Return groups in descending priority order (high to low)
        groups.into_iter().rev().map(|(_, rules)| rules).collect()
    }

    /// Get all rules that match a given event, sorted by priority.
    /// Deprecated: Use get_matching_rules_grouped for proper priority grouping.
    ///
    /// 获取匹配给定事件的所有规则，按优先级排序。
    /// 已弃用：使用 get_matching_rules_grouped 进行正确的优先级分组。
    pub fn get_matching_rules(&mut self, event: &FactEvent) -> Vec<&Rule> {
        // Rebuild sorted list if dirty
        if self.dirty {
            self.sorted_rules = self.rules.keys().cloned().collect();
            self.sorted_rules.sort_by(|a, b| {
                let rule_a = self.rules.get(a);
                let rule_b = self.rules.get(b);
                match (rule_a, rule_b) {
                    (Some(a), Some(b)) => {
                        // First by priority (descending)
                        let priority_cmp = b.priority.cmp(&a.priority);
                        if priority_cmp != std::cmp::Ordering::Equal {
                            return priority_cmp;
                        }
                        // Then by condition count (ascending)
                        a.condition_expressions
                            .len()
                            .cmp(&b.condition_expressions.len())
                    }
                    _ => std::cmp::Ordering::Equal,
                }
            });
            self.dirty = false;
        }

        self.sorted_rules
            .iter()
            .filter_map(|id| self.rules.get(id))
            .filter(|rule| rule.matches_event(event))
            .collect()
    }

    /// Get the number of registered rules.
    ///
    /// 获取已注册规则的数量。
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Check if the registry is empty.
    ///
    /// 检查注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Clear all rules from the registry.
    ///
    /// 清除注册表中的所有规则。
    pub fn clear(&mut self) {
        self.rules.clear();
        self.sorted_rules.clear();
        self.dirty = false;
    }

    /// Iterate over all rules in the registry.
    ///
    /// 迭代注册表中的所有规则。
    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.values()
    }
}

/// Layered rule registry that manages rules with different scopes.
/// Rules are separated into Global, Local, and View layers with different lifecycles.
///
/// 分层规则注册表，管理不同作用域的规则。
/// 规则按 Global、Local 和 View 层分离，具有不同的生命周期。
#[derive(Resource, Default)]
pub struct LayeredRuleRegistry {
    /// Global rules - persist for the entire application lifetime.
    ///
    /// 全局规则 - 在整个应用生命周期内持续存在。
    global: RuleRegistry,

    /// Local rules - scoped to the current scene/state.
    ///
    /// 局部规则 - 限定于当前场景/状态。
    local: RuleRegistry,

    /// View rules - keyed by View entity, cleared when View is despawned.
    ///
    /// 视图规则 - 按 View 实体键控，View 销毁时清除。
    view: HashMap<Entity, RuleRegistry>,
}

impl LayeredRuleRegistry {
    /// Create a new empty layered rule registry.
    ///
    /// 创建新的空分层规则注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a rule to the appropriate layer based on its scope.
    /// Note: View-scoped rules must use `register_view_rule` instead.
    ///
    /// 根据作用域将规则注册到相应层。
    /// 注意：View 作用域的规则必须使用 `register_view_rule`。
    pub fn register(&mut self, rule: Rule) {
        match rule.scope {
            RuleScope::Global => self.global.register(rule),
            RuleScope::Local => self.local.register(rule),
            RuleScope::View => {
                // View-scoped rules MUST be registered via register_view_rule() with an Entity.
                // This is a programming error - using Local as fallback but logging as error.
                // View 作用域规则必须通过 register_view_rule() 与 Entity 一起注册。
                // 这是编程错误 - 使用 Local 作为回退但记录为错误。
                error!(
                    "BUG: View-scoped rule '{}' registered without view entity! \
                    Use register_view_rule(entity, rule) instead. \
                    Falling back to Local scope which may cause rule leakage across scenes.",
                    rule.id
                );
                self.local.register(rule);
            }
        }
    }

    /// Register a rule to a specific View's registry.
    ///
    /// 将规则注册到特定 View 的注册表。
    pub fn register_view_rule(&mut self, view_entity: Entity, rule: Rule) {
        self.view.entry(view_entity).or_default().register(rule);
    }

    /// Clear all Local layer rules.
    /// Called when exiting a scene/state.
    ///
    /// 清除所有 Local 层规则。
    /// 在退出场景/状态时调用。
    pub fn clear_local(&mut self) {
        self.local.clear();
        info!("LayeredRuleRegistry: Cleared local layer rules");
    }

    /// Clear rules for a specific View entity.
    /// Called when a View is despawned.
    ///
    /// 清除特定 View 实体的规则。
    /// 在 View 销毁时调用。
    pub fn clear_view(&mut self, view_entity: Entity) {
        if self.view.remove(&view_entity).is_some() {
            info!(
                "LayeredRuleRegistry: Cleared rules for view entity {:?}",
                view_entity
            );
        }
    }

    /// Get all matching rules grouped by priority, from all layers.
    /// Rules are grouped by priority (high to low), and within each group
    /// sorted by condition count (fewer conditions first).
    ///
    /// 获取所有层中匹配的规则，按优先级分组。
    /// 规则按优先级分组（高到低），每组内按条件数量排序（条件少的在前）。
    pub fn get_matching_rules_grouped(&self, event: &FactEvent) -> Vec<Vec<&Rule>> {
        let mut all_groups: BTreeMap<i32, Vec<&Rule>> = BTreeMap::new();

        // Collect from all layers
        for rule in self.global.iter() {
            if rule.matches_event(event) {
                all_groups.entry(rule.priority).or_default().push(rule);
            }
        }
        for rule in self.local.iter() {
            if rule.matches_event(event) {
                all_groups.entry(rule.priority).or_default().push(rule);
            }
        }
        for registry in self.view.values() {
            for rule in registry.iter() {
                if rule.matches_event(event) {
                    all_groups.entry(rule.priority).or_default().push(rule);
                }
            }
        }

        // Sort each group by condition count (fewer first)
        for group in all_groups.values_mut() {
            group.sort_by_key(|r| r.condition_expressions.len());
        }

        // Return in descending priority order
        all_groups
            .into_iter()
            .rev()
            .map(|(_, rules)| rules)
            .collect()
    }

    /// Get a flat list of all matching rules, sorted by priority then condition count.
    ///
    /// 获取所有匹配规则的扁平列表，按优先级和条件数量排序。
    pub fn get_matching_rules(&self, event: &FactEvent) -> Vec<&Rule> {
        self.get_matching_rules_grouped(event)
            .into_iter()
            .flatten()
            .collect()
    }

    /// Get total number of rules across all layers.
    ///
    /// 获取所有层中规则的总数。
    pub fn len(&self) -> usize {
        self.global.len() + self.local.len() + self.view.values().map(|r| r.len()).sum::<usize>()
    }

    /// Check if all layers are empty.
    ///
    /// 检查所有层是否为空。
    pub fn is_empty(&self) -> bool {
        self.global.is_empty() && self.local.is_empty() && self.view.values().all(|r| r.is_empty())
    }

    /// Get a reference to a rule by ID, searching all layers.
    ///
    /// 按 ID 获取规则的引用，搜索所有层。
    pub fn get(&self, rule_id: &str) -> Option<&Rule> {
        self.global
            .get(rule_id)
            .or_else(|| self.local.get(rule_id))
            .or_else(|| self.view.values().find_map(|r| r.get(rule_id)))
    }

    /// Iterate over all rules in the Global layer.
    ///
    /// 迭代 Global 层中的所有规则。
    pub fn global_iter(&self) -> impl Iterator<Item = &Rule> {
        self.global.iter()
    }

    /// Iterate over all rules in the Local layer.
    ///
    /// 迭代 Local 层中的所有规则。
    pub fn local_iter(&self) -> impl Iterator<Item = &Rule> {
        self.local.iter()
    }

    /// Iterate over all View layers with their entity keys.
    ///
    /// 迭代所有 View 层及其实体键。
    pub fn view_iter(&self) -> impl Iterator<Item = (Entity, &RuleRegistry)> {
        self.view.iter().map(|(e, r)| (*e, r))
    }

    /// Iterate over all rules across all layers.
    ///
    /// 迭代所有层中的所有规则。
    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.global
            .iter()
            .chain(self.local.iter())
            .chain(self.view.values().flat_map(|r| r.iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FactDatabase;

    #[test]
    fn test_rule_builder() {
        let rule = Rule::builder("test_rule", "test_event")
            .condition_expr("$counter == 3")
            .modify(FactModification::Set(
                "result".to_string(),
                FactValue::Bool(true),
            ))
            .output("result_event")
            .priority(10)
            .build();

        assert_eq!(rule.id, "test_rule");
        assert_eq!(rule.trigger.0, "test_event");
        assert_eq!(rule.priority, 10);
        assert!(rule.enabled);
        assert_eq!(rule.condition_expressions, vec!["$counter == 3"]);
    }

    #[test]
    fn test_fact_modification_set() {
        let mut db = LayeredFactDatabase::new();
        let mod_set = FactModification::Set("key".to_string(), FactValue::Int(42));
        mod_set.apply(&mut db);
        assert_eq!(db.get_int("key"), Some(42));
    }

    #[test]
    fn test_fact_modification_increment() {
        let mut db = LayeredFactDatabase::new();
        db.set("counter", 10i64);
        let mod_inc = FactModification::Increment("counter".to_string(), 5);
        mod_inc.apply(&mut db);
        assert_eq!(db.get_int("counter"), Some(15));
    }

    #[test]
    fn test_fact_modification_remove() {
        let mut db = LayeredFactDatabase::new();
        db.set("to_remove", 100i64);
        assert!(db.contains("to_remove"));

        let mod_remove = FactModification::Remove("to_remove".to_string());
        mod_remove.apply(&mut db);
        assert!(!db.contains_local("to_remove"));
    }

    #[test]
    fn test_fact_modification_toggle() {
        let mut db = LayeredFactDatabase::new();
        db.set("flag", false);

        let mod_toggle = FactModification::Toggle("flag".to_string());
        mod_toggle.apply(&mut db);
        assert_eq!(db.get_bool("flag"), Some(true));

        mod_toggle.apply(&mut db);
        assert_eq!(db.get_bool("flag"), Some(false));
    }

    #[test]
    fn test_fact_modification_toggle_missing_key() {
        let mut db = LayeredFactDatabase::new();
        // Toggle on missing key should default to false, then toggle to true
        let mod_toggle = FactModification::Toggle("missing".to_string());
        mod_toggle.apply(&mut db);
        assert_eq!(db.get_bool("missing"), Some(true));
    }

    #[test]
    fn test_rule_registry_basic() {
        let mut registry = RuleRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        let rule = Rule::builder("rule1", "event1").build();
        registry.register(rule);

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.get("rule1").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_rule_registry_unregister() {
        let mut registry = RuleRegistry::new();
        let rule = Rule::builder("rule1", "event1").build();
        registry.register(rule);

        let unregistered = registry.unregister("rule1");
        assert!(unregistered.is_some());
        assert!(registry.is_empty());

        // Unregister non-existent
        let unregistered_none = registry.unregister("nonexistent");
        assert!(unregistered_none.is_none());
    }

    #[test]
    fn test_rule_registry_set_enabled() {
        let mut registry = RuleRegistry::new();
        let rule = Rule::builder("rule1", "event1").build();
        registry.register(rule);

        assert!(registry.get("rule1").unwrap().enabled);

        registry.set_enabled("rule1", false);
        assert!(!registry.get("rule1").unwrap().enabled);

        registry.set_enabled("rule1", true);
        assert!(registry.get("rule1").unwrap().enabled);
    }

    #[test]
    fn test_rule_registry_get_matching_rules() {
        let mut registry = RuleRegistry::new();

        let rule1 = Rule::builder("rule1", "event_a").priority(10).build();
        let rule2 = Rule::builder("rule2", "event_a").priority(5).build();
        let rule3 = Rule::builder("rule3", "event_b").priority(20).build();

        registry.register(rule1);
        registry.register(rule2);
        registry.register(rule3);

        let event_a = FactEvent::new("event_a");
        let matching = registry.get_matching_rules(&event_a);

        // Should match rule1 and rule2, sorted by priority (higher first)
        assert_eq!(matching.len(), 2);
        assert_eq!(matching[0].id, "rule1"); // priority 10
        assert_eq!(matching[1].id, "rule2"); // priority 5
    }

    #[test]
    fn test_rule_registry_iter() {
        let mut registry = RuleRegistry::new();
        registry.register(Rule::builder("r1", "e1").build());
        registry.register(Rule::builder("r2", "e2").build());
        registry.register(Rule::builder("r3", "e3").build());

        let count = registry.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_rule_builder_enabled_false() {
        let rule = Rule::builder("disabled_rule", "event")
            .enabled(false)
            .build();

        assert!(!rule.enabled);
    }

    #[test]
    fn test_rule_matches_disabled() {
        let mut registry = RuleRegistry::new();
        let rule = Rule::builder("rule1", "event_a").enabled(false).build();
        registry.register(rule);

        let event_a = FactEvent::new("event_a");
        let matching = registry.get_matching_rules(&event_a);

        // Disabled rules should not match
        assert!(matching.is_empty());
    }
}
