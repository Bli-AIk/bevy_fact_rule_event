//! # rule.rs
//!
//! Rule definitions - the logic layer of FRE.
//! Rules contain triggers, conditions (expressions), modifications, and outputs.
//!
//! 规则定义 - FRE 的逻辑层。
//! 规则包含触发器、条件（表达式）、修改和输出。

use crate::asset::{ActionDef, CoreActionDef};
use crate::database::FactValue;
use crate::event::{FactEvent, FactEventId};
use crate::expr;
use crate::layered::LayeredFactDatabase;
use bevy::prelude::*;

mod layered_registry;
mod registry;

pub use layered_registry::LayeredRuleRegistry;
pub use registry::RuleRegistry;

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

    /// Increment an integer fact by a whole-number value.
    ///
    /// 将整数事实增加指定的整数值。
    Increment(String, i64),

    /// Add a numeric value to a fact.
    ///
    /// 向事实添加数值。
    Add(String, f64),

    /// Subtract a numeric value from a fact.
    ///
    /// 从事实减去数值。
    Sub(String, f64),

    /// Multiply a fact by a numeric value.
    ///
    /// 将事实乘以数值。
    Mul(String, f64),

    /// Divide a fact by a numeric value.
    ///
    /// 将事实除以数值。
    Div(String, f64),

    /// Apply modulo operation to a fact.
    ///
    /// 对事实应用取模运算。
    Mod(String, i64),

    /// Clamp a fact value between min and max.
    ///
    /// 将事实值限制在 min 和 max 之间。
    Clamp(String, f64, f64),

    /// Wrap a fact value within a range [min, max).
    ///
    /// 将事实值包裹在范围 [min, max) 内。
    Wrap(String, i64, i64),

    /// Evaluate an expression and store the result in a fact.
    ///
    /// 评估表达式并将结果存储在事实中。
    Eval(String, String),

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
            FactModification::Add(key, amount) => {
                db.add(key, *amount);
            }
            FactModification::Sub(key, amount) => {
                db.sub(key, *amount);
            }
            FactModification::Mul(key, factor) => {
                db.mul(key, *factor);
            }
            FactModification::Div(key, divisor) => {
                db.div(key, *divisor);
            }
            FactModification::Mod(key, divisor) => {
                db.modulo(key, *divisor);
            }
            FactModification::Clamp(key, min, max) => {
                db.clamp(key, *min, *max);
            }
            FactModification::Wrap(key, min, max) => {
                db.wrap(key, *min, *max);
            }
            FactModification::Eval(key, expression) => {
                if let Some(value) = expr::evaluate_expr_to_fact(expression, db) {
                    db.set_local(key.as_str(), value);
                }
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
pub struct Rule<A: ActionDef = CoreActionDef> {
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

    /// Actions to execute when this rule fires.
    /// These are game-specific actions that are processed by the bridge layer.
    ///
    /// 当此规则触发时要执行的动作。
    /// 这些是由桥接层处理的游戏特定动作。
    pub actions: Vec<A>,
}

impl<A: ActionDef> Rule<A> {
    /// Create a new rule builder.
    ///
    /// 创建新的规则构建器。
    pub fn builder(id: impl Into<String>, trigger: impl Into<FactEventId>) -> RuleBuilder<A> {
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
pub struct RuleBuilder<A: ActionDef = CoreActionDef> {
    id: String,
    scope: RuleScope,
    trigger: FactEventId,
    condition_expressions: Vec<String>,
    modifications: Vec<FactModification>,
    outputs: Vec<FactEventId>,
    enabled: bool,
    priority: i32,
    consume_event: bool,
    actions: Vec<A>,
}

impl<A: ActionDef> RuleBuilder<A> {
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
            actions: Vec::new(),
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
    pub fn build(self) -> Rule<A> {
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
            actions: self.actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::CoreActionDef;

    #[test]
    fn test_rule_builder() {
        let rule = Rule::<CoreActionDef>::builder("test_rule", "test_event")
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
        let mut registry = RuleRegistry::<CoreActionDef>::new();
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
        let mut registry = RuleRegistry::<CoreActionDef>::new();
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
        let mut registry = RuleRegistry::<CoreActionDef>::new();
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
        let mut registry = RuleRegistry::<CoreActionDef>::new();

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
        let mut registry = RuleRegistry::<CoreActionDef>::new();
        registry.register(Rule::builder("r1", "e1").build());
        registry.register(Rule::builder("r2", "e2").build());
        registry.register(Rule::builder("r3", "e3").build());

        let count = registry.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_rule_builder_enabled_false() {
        let rule = Rule::<CoreActionDef>::builder("disabled_rule", "event")
            .enabled(false)
            .build();

        assert!(!rule.enabled);
    }

    #[test]
    fn test_rule_matches_disabled() {
        let mut registry = RuleRegistry::<CoreActionDef>::new();
        let rule = Rule::builder("rule1", "event_a").enabled(false).build();
        registry.register(rule);

        let event_a = FactEvent::new("event_a");
        let matching = registry.get_matching_rules(&event_a);

        // Disabled rules should not match
        assert!(matching.is_empty());
    }
}
