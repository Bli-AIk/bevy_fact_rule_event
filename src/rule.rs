//! # rule.rs
//!
//! Rule definitions - the logic layer of FRE.
//! Rules contain triggers, conditions, actions, modifications, and outputs.
//!
//! 规则定义 - FRE 的逻辑层。
//! 规则包含触发器、条件、动作、修改和输出。

use crate::database::{FactReader, FactValue};
use crate::event::{FactEvent, FactEventId};
use crate::layered::LayeredFactDatabase;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Condition predicate for checking facts.
///
/// 用于检查事实的条件谓词。
#[derive(Clone)]
pub enum RuleCondition {
    /// Check if a fact equals a specific value.
    ///
    /// 检查事实是否等于特定值。
    Equals(String, FactValue),

    /// Check if an integer fact is greater than a value.
    ///
    /// 检查整数事实是否大于某个值。
    GreaterThan(String, i64),

    /// Check if an integer fact is less than a value.
    ///
    /// 检查整数事实是否小于某个值。
    LessThan(String, i64),

    /// Check if an integer fact is greater than or equal to a value.
    ///
    /// 检查整数事实是否大于或等于某个值。
    GreaterOrEqual(String, i64),

    /// Check if an integer fact is less than or equal to a value.
    ///
    /// 检查整数事实是否小于或等于某个值。
    LessOrEqual(String, i64),

    /// Check if a fact exists.
    ///
    /// 检查事实是否存在。
    Exists(String),

    /// Check if a fact does not exist.
    ///
    /// 检查事实是否不存在。
    NotExists(String),

    /// Check if a boolean fact is true.
    ///
    /// 检查布尔事实是否为真。
    IsTrue(String),

    /// Check if a boolean fact is false.
    ///
    /// 检查布尔事实是否为假。
    IsFalse(String),

    /// Logical AND of multiple conditions.
    ///
    /// 多个条件的逻辑与。
    And(Vec<RuleCondition>),

    /// Logical OR of multiple conditions.
    ///
    /// 多个条件的逻辑或。
    Or(Vec<RuleCondition>),

    /// Logical NOT of a condition.
    ///
    /// 条件的逻辑非。
    Not(Box<RuleCondition>),

    /// Always true (no condition).
    ///
    /// 总是为真（无条件）。
    Always,
}

impl RuleCondition {
    /// Evaluate the condition against any fact reader (FactDatabase or LayeredFactDatabase).
    ///
    /// 根据任何事实读取器（FactDatabase 或 LayeredFactDatabase）评估条件。
    pub fn evaluate(&self, db: &impl FactReader) -> bool {
        match self {
            RuleCondition::Equals(key, value) => db.get_by_str(key) == Some(value),

            RuleCondition::GreaterThan(key, threshold) => {
                db.get_int(key).is_some_and(|v| v > *threshold)
            }

            RuleCondition::LessThan(key, threshold) => {
                db.get_int(key).is_some_and(|v| v < *threshold)
            }

            RuleCondition::GreaterOrEqual(key, threshold) => {
                db.get_int(key).is_some_and(|v| v >= *threshold)
            }

            RuleCondition::LessOrEqual(key, threshold) => {
                db.get_int(key).is_some_and(|v| v <= *threshold)
            }

            RuleCondition::Exists(key) => db.contains(key),

            RuleCondition::NotExists(key) => !db.contains(key),

            RuleCondition::IsTrue(key) => db.get_bool(key) == Some(true),

            RuleCondition::IsFalse(key) => db.get_bool(key) == Some(false),

            RuleCondition::And(conditions) => conditions.iter().all(|c| c.evaluate(db)),

            RuleCondition::Or(conditions) => conditions.iter().any(|c| c.evaluate(db)),

            RuleCondition::Not(condition) => !condition.evaluate(db),

            RuleCondition::Always => true,
        }
    }
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

/// Action to execute when a rule is triggered.
/// Actions are callbacks that can modify game state.
///
/// 规则触发时执行的动作。
/// 动作是可以修改游戏状态的回调。
pub type RuleActionFn =
    Arc<dyn Fn(&FactEvent, &LayeredFactDatabase, &mut bevy::ecs::system::Commands) + Send + Sync>;

/// Wrapper for rule actions.
///
/// 规则动作的包装器。
#[derive(Clone)]
pub struct RuleAction {
    pub action: RuleActionFn,
}

impl RuleAction {
    /// Create a new action from a closure.
    ///
    /// 从闭包创建新动作。
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&FactEvent, &LayeredFactDatabase, &mut bevy::ecs::system::Commands)
            + Send
            + Sync
            + 'static,
    {
        Self {
            action: Arc::new(f),
        }
    }

    /// Execute the action.
    ///
    /// 执行动作。
    pub fn execute(&self, event: &FactEvent, db: &LayeredFactDatabase, commands: &mut Commands) {
        (self.action)(event, db, commands);
    }
}

/// A rule definition containing trigger, condition, actions, modifications, and outputs.
///
/// 包含触发器、条件、动作、修改和输出的规则定义。
#[derive(Clone)]
pub struct Rule {
    /// Unique identifier for this rule.
    ///
    /// 此规则的唯一标识符。
    pub id: String,

    /// Event ID that triggers this rule.
    ///
    /// 触发此规则的事件 ID。
    pub trigger: FactEventId,

    /// Condition to check before executing.
    ///
    /// 执行前要检查的条件。
    pub condition: RuleCondition,

    /// Actions to execute when triggered and condition is met.
    ///
    /// 触发且条件满足时要执行的动作。
    pub actions: Vec<RuleAction>,

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

    /// Priority for rule ordering (higher = first).
    ///
    /// 规则排序的优先级（越高越先）。
    pub priority: i32,
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

    /// Evaluate the condition against any fact reader.
    ///
    /// 根据任何事实读取器评估条件。
    pub fn check_condition(&self, db: &impl FactReader) -> bool {
        self.condition.evaluate(db)
    }
}

/// Builder for constructing rules.
///
/// 用于构建规则的构建器。
pub struct RuleBuilder {
    id: String,
    trigger: FactEventId,
    condition: RuleCondition,
    actions: Vec<RuleAction>,
    modifications: Vec<FactModification>,
    outputs: Vec<FactEventId>,
    enabled: bool,
    priority: i32,
}

impl RuleBuilder {
    /// Create a new rule builder.
    ///
    /// 创建新的规则构建器。
    pub fn new(id: impl Into<String>, trigger: impl Into<FactEventId>) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into(),
            condition: RuleCondition::Always,
            actions: Vec::new(),
            modifications: Vec::new(),
            outputs: Vec::new(),
            enabled: true,
            priority: 0,
        }
    }

    /// Set the condition for this rule.
    ///
    /// 设置此规则的条件。
    pub fn condition(mut self, condition: RuleCondition) -> Self {
        self.condition = condition;
        self
    }

    /// Add an action to this rule.
    ///
    /// 向此规则添加动作。
    pub fn action(mut self, action: RuleAction) -> Self {
        self.actions.push(action);
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

    /// Build the rule.
    ///
    /// 构建规则。
    pub fn build(self) -> Rule {
        Rule {
            id: self.id,
            trigger: self.trigger,
            condition: self.condition,
            actions: self.actions,
            modifications: self.modifications,
            outputs: self.outputs,
            enabled: self.enabled,
            priority: self.priority,
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

    /// Get all rules that match a given event, sorted by priority.
    ///
    /// 获取匹配给定事件的所有规则，按优先级排序。
    pub fn get_matching_rules(&mut self, event: &FactEvent) -> Vec<&Rule> {
        // Rebuild sorted list if dirty
        if self.dirty {
            self.sorted_rules = self.rules.keys().cloned().collect();
            self.sorted_rules.sort_by(|a, b| {
                let pa = self.rules.get(a).map(|r| r.priority).unwrap_or(0);
                let pb = self.rules.get(b).map(|r| r.priority).unwrap_or(0);
                pb.cmp(&pa) // Higher priority first
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_condition_evaluation() {
        let mut db = FactDatabase::new();
        db.set("counter", 5i64);
        db.set("flag", true);

        assert!(RuleCondition::Equals("counter".to_string(), FactValue::Int(5)).evaluate(&db));
        assert!(RuleCondition::GreaterThan("counter".to_string(), 3).evaluate(&db));
        assert!(RuleCondition::LessThan("counter".to_string(), 10).evaluate(&db));
        assert!(RuleCondition::IsTrue("flag".to_string()).evaluate(&db));
        assert!(RuleCondition::Exists("counter".to_string()).evaluate(&db));
        assert!(RuleCondition::NotExists("missing".to_string()).evaluate(&db));
    }

    #[test]
    fn test_rule_builder() {
        let rule = Rule::builder("test_rule", "test_event")
            .condition(RuleCondition::Equals(
                "counter".to_string(),
                FactValue::Int(3),
            ))
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
    }
}
