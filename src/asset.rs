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
}

impl From<FactValueDef> for FactValue {
    fn from(def: FactValueDef) -> Self {
        match def {
            FactValueDef::Int(v) => FactValue::Int(v),
            FactValueDef::Float(v) => FactValue::Float(v),
            FactValueDef::Bool(v) => FactValue::Bool(v),
            FactValueDef::String(v) => FactValue::String(v),
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

    /// Play a sound effect.
    PlaySound { path: String },

    /// Spawn an entity (requires game-specific handler).
    SpawnEntity { template: String },

    /// Custom action identified by name (handled by game code).
    Custom {
        action_type: String,
        params: HashMap<String, String>,
    },
}

// ============================================================================
// Serializable Rule Definition
// ============================================================================

/// A single rule definition that can be loaded from RON.
///
/// 可从 RON 加载的单个规则定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDef {
    /// Unique identifier for this rule.
    pub id: String,

    /// Event ID that triggers this rule.
    pub trigger: String,

    /// Condition to check before executing (defaults to Always).
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
        Rule {
            id: self.id.clone(),
            trigger: FactEventId::new(&self.trigger),
            condition: self.condition.clone().into(),
            actions: Vec::new(), // Actions are handled separately by game code
            modifications: self.modifications.iter().cloned().map(Into::into).collect(),
            outputs: self.outputs.iter().map(FactEventId::new).collect(),
            enabled: self.enabled,
            priority: self.priority,
        }
    }
}

// ============================================================================
// Rule Set Asset
// ============================================================================

/// A collection of rules that can be loaded from a RON file.
///
/// 可从 RON 文件加载的规则集合。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetAsset {
    /// Version number for format compatibility.
    #[serde(default = "default_version")]
    pub version: u32,

    /// Initial facts to set when this rule set is loaded.
    #[serde(default)]
    pub initial_facts: HashMap<String, FactValueDef>,

    /// The rules defined in this set.
    pub rules: Vec<RuleDef>,
}

fn default_version() -> u32 {
    1
}

impl RuleSetAsset {
    /// Register all rules from this asset into the registry.
    ///
    /// 将此资产中的所有规则注册到注册表。
    pub fn register_rules(&self, registry: &mut RuleRegistry) {
        for rule_def in &self.rules {
            let rule = rule_def.to_rule();
            info!("FRE: Registering rule '{}' from asset", rule.id);
            registry.register(rule);
        }
    }

    /// Get the initial facts defined in this asset.
    ///
    /// 获取此资产中定义的初始事实。
    pub fn get_initial_facts(&self) -> &HashMap<String, FactValueDef> {
        &self.initial_facts
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

/// Asset loader for .rules.ron files.
///
/// .rules.ron 文件的资产加载器。
#[derive(Default)]
pub struct RuleSetAssetLoader;

impl AssetLoader for RuleSetAssetLoader {
    type Asset = RuleSetAsset;
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
            let asset = ron::de::from_bytes::<RuleSetAsset>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rules.ron"]
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
            RuleActionDef::PlaySound { .. } => "PlaySound",
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
    fn test_rule_def_serialization() {
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
            trigger: "test_event",
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
        assert_eq!(asset.rules[0].trigger, "test_event");
        assert_eq!(asset.rules[0].priority, 10);
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
}
