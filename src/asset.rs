//! # asset.rs
//!
//! Data-driven rule definitions that can be loaded from RON files.
//! This module provides serializable types that map to runtime Rule structures.
//!
//! 可从 RON 文件加载的数据驱动规则定义。
//! 本模块提供可序列化类型，映射到运行时 Rule 结构。

use crate::database::FactValue;
use crate::event::FactEventId;
use crate::rule::{FactModification, Rule, RuleCondition, RuleRegistry};
use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Serializable Value Types
// ============================================================================

/// Serializable fact value for RON files.
///
/// RON 文件的可序列化事实值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactValueDef {
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

impl From<FactValueDef> for FactValue {
    fn from(def: FactValueDef) -> Self {
        match def {
            FactValueDef::Int(v) => FactValue::Int(v),
            FactValueDef::Float(v) => FactValue::Float(v),
            FactValueDef::Bool(v) => FactValue::Bool(v),
            FactValueDef::String(v) => FactValue::String(v),
            FactValueDef::StringList(v) => FactValue::StringList(v),
            FactValueDef::IntList(v) => FactValue::IntList(v),
        }
    }
}

// ============================================================================
// Serializable Condition Types
// ============================================================================

/// Serializable condition definition for RON files.
///
/// RON 文件的可序列化条件定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleConditionDef {
    /// Check if a fact equals a specific value.
    Equals { key: String, value: FactValueDef },

    /// Check if an integer fact is greater than a value.
    GreaterThan { key: String, value: i64 },

    /// Check if an integer fact is less than a value.
    LessThan { key: String, value: i64 },

    /// Check if an integer fact is greater than or equal to a value.
    GreaterOrEqual { key: String, value: i64 },

    /// Check if an integer fact is less than or equal to a value.
    LessOrEqual { key: String, value: i64 },

    /// Check if a fact exists.
    Exists(String),

    /// Check if a fact does not exist.
    NotExists(String),

    /// Check if a boolean fact is true.
    IsTrue(String),

    /// Check if a boolean fact is false.
    IsFalse(String),

    /// Logical AND of multiple conditions.
    And(Vec<RuleConditionDef>),

    /// Logical OR of multiple conditions.
    Or(Vec<RuleConditionDef>),

    /// Logical NOT of a condition.
    Not(Box<RuleConditionDef>),

    /// Always true (no condition).
    Always,
}

impl From<RuleConditionDef> for RuleCondition {
    fn from(def: RuleConditionDef) -> Self {
        match def {
            RuleConditionDef::Equals { key, value } => RuleCondition::Equals(key, value.into()),
            RuleConditionDef::GreaterThan { key, value } => RuleCondition::GreaterThan(key, value),
            RuleConditionDef::LessThan { key, value } => RuleCondition::LessThan(key, value),
            RuleConditionDef::GreaterOrEqual { key, value } => {
                RuleCondition::GreaterOrEqual(key, value)
            }
            RuleConditionDef::LessOrEqual { key, value } => RuleCondition::LessOrEqual(key, value),
            RuleConditionDef::Exists(key) => RuleCondition::Exists(key),
            RuleConditionDef::NotExists(key) => RuleCondition::NotExists(key),
            RuleConditionDef::IsTrue(key) => RuleCondition::IsTrue(key),
            RuleConditionDef::IsFalse(key) => RuleCondition::IsFalse(key),
            RuleConditionDef::And(conditions) => {
                RuleCondition::And(conditions.into_iter().map(Into::into).collect())
            }
            RuleConditionDef::Or(conditions) => {
                RuleCondition::Or(conditions.into_iter().map(Into::into).collect())
            }
            RuleConditionDef::Not(condition) => RuleCondition::Not(Box::new((*condition).into())),
            RuleConditionDef::Always => RuleCondition::Always,
        }
    }
}

// ============================================================================
// Serializable Modification Types
// ============================================================================

/// Serializable modification definition for RON files.
///
/// RON 文件的可序列化修改定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactModificationDef {
    /// Set a fact to a specific value.
    Set { key: String, value: FactValueDef },

    /// Increment an integer fact by a value.
    Increment { key: String, amount: i64 },

    /// Remove a fact.
    Remove(String),

    /// Toggle a boolean fact.
    Toggle(String),
}

impl From<FactModificationDef> for FactModification {
    fn from(def: FactModificationDef) -> Self {
        match def {
            FactModificationDef::Set { key, value } => FactModification::Set(key, value.into()),
            FactModificationDef::Increment { key, amount } => {
                FactModification::Increment(key, amount)
            }
            FactModificationDef::Remove(key) => FactModification::Remove(key),
            FactModificationDef::Toggle(key) => FactModification::Toggle(key),
        }
    }
}

// ============================================================================
// Serializable Action Types
// ============================================================================

/// Kind of action event (press state).
///
/// 动作事件的类型（按下状态）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionEventKind {
    /// Just pressed (single frame).
    ///
    /// 刚按下（单帧）。
    JustPressed,

    /// Held down (continuous).
    ///
    /// 正在按住（持续）。
    Pressed,

    /// Just released (single frame).
    ///
    /// 刚释放（单帧）。
    JustReleased,
}

/// Serializable event definition for RON files.
/// Supports both string-based event IDs and structured ActionEvent.
///
/// RON 文件的可序列化事件定义。
/// 支持字符串事件 ID 和结构化 ActionEvent。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleEventDef {
    /// String-based event ID (e.g., "input_confirm", "player_died").
    ///
    /// 字符串事件 ID（例如 "input_confirm", "player_died"）。
    Event(String),

    /// Action event with specific action and kind.
    ///
    /// 带有特定动作和类型的动作事件。
    ActionEvent {
        /// Action name (e.g., "Up", "Confirm", "Cancel").
        action: String,
        /// Event kind (JustPressed, Pressed, JustReleased).
        kind: ActionEventKind,
    },
}

impl Default for RuleEventDef {
    fn default() -> Self {
        RuleEventDef::Event(String::new())
    }
}

impl RuleEventDef {
    /// Convert to a string event ID for matching.
    ///
    /// 转换为字符串事件 ID 用于匹配。
    pub fn to_event_id(&self) -> String {
        match self {
            RuleEventDef::Event(id) => id.clone(),
            RuleEventDef::ActionEvent { action, kind } => {
                let kind_str = match kind {
                    ActionEventKind::JustPressed => "just_pressed",
                    ActionEventKind::Pressed => "pressed",
                    ActionEventKind::JustReleased => "just_released",
                };
                format!("action:{}:{}", action, kind_str)
            }
        }
    }
}

/// Serializable action definition for RON files.
/// Actions are limited to what can be expressed in data (no closures).
///
/// RON 文件的可序列化动作定义。
/// 动作限于可用数据表达的内容（无闭包）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleActionDef {
    /// Log a message (for debugging).
    Log { message: String },

    /// Set a resource field (requires game-specific handler).
    SetResource {
        resource: String,
        field: String,
        value: FactValueDef,
    },

    /// Play a sound effect using fuzzy search.
    /// Uses audio::play_sound() which searches in the audios directory.
    ///
    /// 使用模糊搜索播放音效。
    /// 使用 audio::play_sound()，在 audios 目录中搜索。
    PlaySound(String),

    /// Play a sound effect using full path.
    /// Uses audio::play_sound_full_path() without adding prefixes.
    ///
    /// 使用完整路径播放音效。
    /// 使用 audio::play_sound_full_path()，不添加前缀。
    PlaySoundFullPath(String),

    /// Set a local fact on the active ViewRoot.
    /// Value can be a literal or an expression (e.g., "$selection - 1").
    ///
    /// 在活跃的 ViewRoot 上设置局部 fact。
    /// 值可以是字面量或表达式（例如 "$selection - 1"）。
    SetLocalFact(String, LocalFactValue),

    /// Close the active View.
    ///
    /// 关闭活跃的 View。
    CloseView,

    /// Switch to a specified state (e.g., "Normal", "Battle").
    ///
    /// 切换到指定状态（例如 "Normal", "Battle"）。
    SwitchState(String),

    /// Emit an FRE event (for chaining rules).
    ///
    /// 发出 FRE 事件（用于规则链）。
    EmitEvent(String),

    /// Spawn an entity (requires game-specific handler).
    SpawnEntity { template: String },

    /// Custom action identified by name (handled by game code).
    Custom {
        action_type: String,
        params: HashMap<String, String>,
    },
}

/// Value for SetLocalFact action - can be literal or expression.
///
/// SetLocalFact 动作的值 - 可以是字面量或表达式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalFactValue {
    /// Integer literal.
    Int(i64),
    /// Float literal.
    Float(f64),
    /// Boolean literal.
    Bool(bool),
    /// String literal.
    String(String),
    /// Expression to evaluate (e.g., "$selection - 1").
    /// Expressions support: $name (local fact), fact('name') (global fact),
    /// arithmetic (+, -, *, /), comparisons (==, !=, <, >, <=, >=).
    ///
    /// 要评估的表达式（例如 "$selection - 1"）。
    /// 表达式支持：$name（局部 fact）、fact('name')（全局 fact）、
    /// 算术运算（+、-、*、/）、比较运算（==、!=、<、>、<=、>=）。
    Expr(String),
}

// ============================================================================
// Serializable Rule Definition
// ============================================================================

/// A single rule definition that can be loaded from RON.
///
/// 可从 RON 加载的单个规则定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDef {
    /// Unique identifier for this rule (optional, auto-generated if not provided).
    #[serde(default)]
    pub id: String,

    /// Event that triggers this rule (supports both string and ActionEvent).
    /// Use `event: "event_name"` for string events.
    /// Use `event: ActionEvent(action: "Up", kind: JustPressed)` for action events.
    ///
    /// 触发此规则的事件（支持字符串和 ActionEvent）。
    #[serde(alias = "trigger")]
    pub event: RuleEventDef,

    /// Conditions to check before executing (list of expression strings).
    /// All conditions must be true for the rule to execute.
    /// Examples: ["$selection > 0"], ["$depth == 0", "$selection == 1"]
    ///
    /// 执行前要检查的条件（表达式字符串列表）。
    /// 所有条件都必须为真才能执行规则。
    #[serde(default)]
    pub conditions: Vec<String>,

    /// Legacy condition field (for backward compatibility, prefer `conditions`).
    #[serde(default = "default_condition")]
    pub condition: RuleConditionDef,

    /// Actions to execute (data-driven, handled by game code).
    #[serde(default)]
    pub actions: Vec<RuleActionDef>,

    /// Modifications to apply to the fact database.
    #[serde(default)]
    pub modifications: Vec<FactModificationDef>,

    /// Events to emit after rule execution.
    #[serde(default)]
    pub outputs: Vec<String>,

    /// Whether this rule is enabled (defaults to true).
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Priority for rule ordering (higher = first, defaults to 0).
    #[serde(default)]
    pub priority: i32,
}

fn default_condition() -> RuleConditionDef {
    RuleConditionDef::Always
}

fn default_enabled() -> bool {
    true
}

impl RuleDef {
    /// Convert to a runtime Rule (without actions, which need game-specific handling).
    ///
    /// 转换为运行时 Rule（不含动作，动作需要游戏特定处理）。
    pub fn to_rule(&self) -> Rule {
        self.to_rule_with_index(0)
    }

    /// Convert to a runtime Rule with an index suffix for unique ID generation.
    ///
    /// 转换为运行时 Rule，使用索引后缀生成唯一 ID。
    pub fn to_rule_with_index(&self, index: usize) -> Rule {
        // Generate ID if not provided, with index suffix for uniqueness
        let id = if self.id.is_empty() {
            format!(
                "rule_{}_{:03}",
                self.event.to_event_id().replace(':', "_"),
                index
            )
        } else {
            self.id.clone()
        };

        Rule {
            id,
            trigger: FactEventId::new(self.event.to_event_id()),
            condition: self.condition.clone().into(),
            condition_expressions: self.conditions.clone(),
            actions: Vec::new(), // Actions are handled separately by game code
            modifications: self.modifications.iter().cloned().map(Into::into).collect(),
            outputs: self.outputs.iter().map(FactEventId::new).collect(),
            enabled: self.enabled,
            priority: self.priority,
        }
    }

    /// Generate a rule ID for a given index, matching the logic used in to_rule_with_index.
    ///
    /// 为给定索引生成规则 ID，与 to_rule_with_index 中使用的逻辑匹配。
    pub fn generate_id(&self, index: usize) -> String {
        if self.id.is_empty() {
            format!(
                "rule_{}_{:03}",
                self.event.to_event_id().replace(':', "_"),
                index
            )
        } else {
            self.id.clone()
        }
    }
}

// ============================================================================
// Rule Set Asset
// ============================================================================

/// A collection of facts and rules that can be loaded from a RON file.
/// This is the unified FRE data asset format - pure data with no type identifier.
///
/// 可从 RON 文件加载的事实和规则集合。
/// 这是统一的 FRE 数据资产格式 - 纯数据，无类型标识。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct FreAsset {
    /// Facts to set when this asset is loaded.
    /// 加载此资产时设置的事实。
    #[serde(default)]
    pub facts: HashMap<String, FactValueDef>,

    /// The rules defined in this set.
    /// 此集合中定义的规则。
    #[serde(default)]
    pub rules: Vec<RuleDef>,
}

impl FreAsset {
    /// Register all rules from this asset into the registry.
    ///
    /// 将此资产中的所有规则注册到注册表。
    pub fn register_rules(&self, registry: &mut RuleRegistry) {
        for (idx, rule_def) in self.rules.iter().enumerate() {
            let rule = rule_def.to_rule_with_index(idx);
            info!("FRE: Registering rule '{}' from asset", rule.id);
            registry.register(rule);
        }
    }

    /// Get the facts defined in this asset.
    ///
    /// 获取此资产中定义的事实。
    pub fn get_facts(&self) -> &HashMap<String, FactValueDef> {
        &self.facts
    }

    /// Get the rule definitions for custom action handling.
    ///
    /// 获取用于自定义动作处理的规则定义。
    pub fn get_rule_defs(&self) -> &[RuleDef] {
        &self.rules
    }
}

// ============================================================================
// Asset Loader
// ============================================================================

/// Asset loader for .fre.ron files.
///
/// .fre.ron 文件的资产加载器。
#[derive(Default)]
pub struct FreAssetLoader;

impl AssetLoader for FreAssetLoader {
    type Asset = FreAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset = ron::de::from_bytes::<FreAsset>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fre.ron"]
    }
}

// ============================================================================
// Action Registry for Game-Specific Handlers
// ============================================================================

/// Type alias for action handler functions.
///
/// 动作处理函数的类型别名。
pub type ActionHandler =
    Box<dyn Fn(&RuleActionDef, &crate::LayeredFactDatabase, &mut Commands) + Send + Sync>;

/// Registry for custom action handlers.
/// Games can register handlers for specific action types.
///
/// 自定义动作处理程序的注册表。
/// 游戏可以为特定动作类型注册处理程序。
#[derive(Resource, Default)]
pub struct ActionHandlerRegistry {
    handlers: HashMap<String, ActionHandler>,
}

impl ActionHandlerRegistry {
    /// Register a handler for a specific action type.
    ///
    /// 为特定动作类型注册处理程序。
    pub fn register<F>(&mut self, action_type: &str, handler: F)
    where
        F: Fn(&RuleActionDef, &crate::LayeredFactDatabase, &mut Commands) + Send + Sync + 'static,
    {
        self.handlers
            .insert(action_type.to_string(), Box::new(handler));
    }

    /// Execute an action using the registered handler.
    ///
    /// 使用注册的处理程序执行动作。
    pub fn execute(
        &self,
        action: &RuleActionDef,
        db: &crate::LayeredFactDatabase,
        commands: &mut Commands,
    ) {
        let action_type = match action {
            RuleActionDef::Log { .. } => "Log",
            RuleActionDef::SetResource { .. } => "SetResource",
            RuleActionDef::PlaySound(_) => "PlaySound",
            RuleActionDef::PlaySoundFullPath(_) => "PlaySoundFullPath",
            RuleActionDef::SetLocalFact(_, _) => "SetLocalFact",
            RuleActionDef::CloseView => "CloseView",
            RuleActionDef::SwitchState(_) => "SwitchState",
            RuleActionDef::EmitEvent(_) => "EmitEvent",
            RuleActionDef::SpawnEntity { .. } => "SpawnEntity",
            RuleActionDef::Custom { action_type, .. } => action_type.as_str(),
        };

        if let Some(handler) = self.handlers.get(action_type) {
            handler(action, db, commands);
        } else {
            // Built-in handlers
            match action {
                RuleActionDef::Log { message } => {
                    info!("FRE Action Log: {}", message);
                }
                RuleActionDef::EmitEvent(event_id) => {
                    // EmitEvent is handled by the systems via outputs, log for debugging
                    debug!("FRE Action EmitEvent: {}", event_id);
                }
                _ => {
                    warn!(
                        "FRE: No handler registered for action type '{}'",
                        action_type
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_def_serialization_legacy() {
        // Test legacy format with `trigger` alias - must use Event wrapper
        let rule_set = r#"
(
    version: 1,
    initial_facts: {
        "counter": Int(0),
        "enabled": Bool(true),
    },
    rules: [
        (
            id: "test_rule",
            event: Event("test_event"),
            condition: Equals(key: "counter", value: Int(3)),
            modifications: [
                Set(key: "triggered", value: Bool(true)),
                Increment(key: "counter", amount: 1),
            ],
            outputs: ["result_event"],
            priority: 10,
        ),
    ],
)
"#;

        let asset: RuleSetAsset = ron::from_str(rule_set).unwrap();
        assert_eq!(asset.version, 1);
        assert_eq!(asset.rules.len(), 1);
        assert_eq!(asset.rules[0].id, "test_rule");
        assert_eq!(asset.rules[0].event.to_event_id(), "test_event");
        assert_eq!(asset.rules[0].priority, 10);
    }

    #[test]
    fn test_rule_def_serialization_new_format() {
        // Test new format with ActionEvent
        let rule_set = r#"
(
    version: 1,
    rules: [
        (
            event: ActionEvent(action: "Up", kind: JustPressed),
            conditions: ["$selection > 0"],
            actions: [
                SetLocalFact("selection", Expr("$selection - 1")),
                PlaySound("choice"),
            ],
        ),
        (
            event: ActionEvent(action: "Confirm", kind: JustPressed),
            conditions: ["$depth == 0"],
            actions: [
                SetLocalFact("depth", Int(1)),
                PlaySoundFullPath("audios/sfx/confirm.wav"),
            ],
        ),
    ],
)
"#;

        let asset: RuleSetAsset = ron::from_str(rule_set).unwrap();
        assert_eq!(asset.rules.len(), 2);

        // First rule: ActionEvent Up
        assert_eq!(asset.rules[0].event.to_event_id(), "action:Up:just_pressed");
        assert_eq!(asset.rules[0].conditions, vec!["$selection > 0"]);

        // Second rule: ActionEvent Confirm
        assert_eq!(
            asset.rules[1].event.to_event_id(),
            "action:Confirm:just_pressed"
        );
        assert_eq!(asset.rules[1].conditions, vec!["$depth == 0"]);
    }

    #[test]
    fn test_rule_def_serialization_string_event() {
        // Test new format with string Event
        let rule_set = r#"
(
    rules: [
        (
            event: Event("custom_event"),
            actions: [
                Log(message: "Custom event fired"),
            ],
        ),
    ],
)
"#;

        let asset: RuleSetAsset = ron::from_str(rule_set).unwrap();
        assert_eq!(asset.rules.len(), 1);
        assert_eq!(asset.rules[0].event.to_event_id(), "custom_event");
    }

    #[test]
    fn test_condition_conversion() {
        let def = RuleConditionDef::And(vec![
            RuleConditionDef::GreaterThan {
                key: "health".to_string(),
                value: 0,
            },
            RuleConditionDef::IsTrue("alive".to_string()),
        ]);

        let _condition: RuleCondition = def.into();
        // Conversion should not panic
    }

    #[test]
    fn test_local_fact_value_variants() {
        let rule_set = r#"
(
    rules: [
        (
            event: Event("test"),
            actions: [
                SetLocalFact("int_val", Int(42)),
                SetLocalFact("float_val", Float(3.14)),
                SetLocalFact("bool_val", Bool(true)),
                SetLocalFact("str_val", String("hello")),
                SetLocalFact("expr_val", Expr("$x + 1")),
            ],
        ),
    ],
)
"#;

        let asset: RuleSetAsset = ron::from_str(rule_set).unwrap();
        assert_eq!(asset.rules[0].actions.len(), 5);
    }
}
