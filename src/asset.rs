//! # asset.rs
//!
//! Data-driven rule definitions that can be loaded from RON files.
//! This module provides serializable types that map to runtime Rule structures.
//!
//! 可从 RON 文件加载的数据驱动规则定义。
//! 本模块提供可序列化类型，映射到运行时 Rule 结构。

use crate::database::FactValue;
use crate::event::FactEventId;
use crate::rule::{FactModification, Rule, RuleRegistry, RuleScope};
use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// ActionDef Trait & CoreActionDef
// ============================================================================

/// Trait for game-specific action definitions.
/// FRE crate is generic over this — consumers define their own action enum.
///
/// 游戏特定动作定义的 trait。
/// FRE crate 对此进行泛型化 — 消费者定义自己的动作枚举。
pub trait ActionDef:
    std::fmt::Debug
    + Clone
    + Send
    + Sync
    + Serialize
    + serde::de::DeserializeOwned
    + bevy::reflect::TypePath
    + 'static
{
    /// Returns a string identifier for this action type (for handler dispatch).
    ///
    /// 返回此动作类型的字符串标识符（用于处理程序分发）。
    fn action_type(&self) -> &str;
}

/// Built-in minimal action definitions provided by the FRE crate.
/// Games can use this directly or define their own enum implementing `ActionDef`.
///
/// FRE crate 提供的内置最小动作定义。
/// 游戏可以直接使用此类型，也可以定义自己的枚举实现 `ActionDef`。
#[derive(Debug, Clone, Serialize, Deserialize, bevy::reflect::TypePath)]
pub enum CoreActionDef {
    /// Log a message (for debugging).
    Log { message: String },
    /// Set a local fact on the active ViewRoot.
    /// Value can be a literal or an expression (e.g., "$selection - 1").
    ///
    /// 在活跃的 ViewRoot 上设置局部 fact。
    /// 值可以是字面量或表达式（例如 "$selection - 1"）。
    SetLocalFact(String, LocalFactValue),
    /// Emit an FRE event (for chaining rules).
    ///
    /// 发出 FRE 事件（用于规则链）。
    EmitEvent(String),
    /// Custom action identified by name (handled by game code).
    Custom {
        action_type: String,
        params: HashMap<String, String>,
    },
}

impl ActionDef for CoreActionDef {
    fn action_type(&self) -> &str {
        match self {
            CoreActionDef::Log { .. } => "Log",
            CoreActionDef::SetLocalFact(_, _) => "SetLocalFact",
            CoreActionDef::EmitEvent(_) => "EmitEvent",
            CoreActionDef::Custom { action_type, .. } => action_type.as_str(),
        }
    }
}

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
    /// Enum variant — resolved to Int at load time via EnumRegistry.
    /// In RON: `Enum("main")`, at runtime equivalent to `Int(0)`.
    ///
    /// 枚举变体 — 加载时通过 EnumRegistry 解析为 Int。
    /// RON 中写作 `Enum("main")`，运行时等价于 `Int(0)`。
    Enum(String),
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
            FactValueDef::Enum(variant) => {
                // Enum without registry resolution — store as String fallback.
                // Callers should use EnumRegistry::resolve_fact_value_def() instead.
                warn!(
                    "FactValueDef::Enum('{}') converted without EnumRegistry — stored as String",
                    variant
                );
                FactValue::String(variant)
            }
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

    /// Increment an integer fact by a value (legacy, use Add for new code).
    Increment { key: String, amount: i64 },

    /// Add a numeric value to a fact.
    Add { key: String, value: f64 },

    /// Subtract a numeric value from a fact.
    Sub { key: String, value: f64 },

    /// Multiply a fact by a numeric value.
    Mul { key: String, value: f64 },

    /// Divide a fact by a numeric value.
    Div { key: String, value: f64 },

    /// Apply modulo operation to a fact.
    Mod { key: String, value: i64 },

    /// Clamp a fact value between min and max.
    Clamp { key: String, min: f64, max: f64 },

    /// Wrap a fact value within a range [min, max).
    Wrap { key: String, min: i64, max: i64 },

    /// Evaluate an expression and store the result in a fact.
    Eval { key: String, expr: String },

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
            FactModificationDef::Add { key, value } => FactModification::Add(key, value),
            FactModificationDef::Sub { key, value } => FactModification::Sub(key, value),
            FactModificationDef::Mul { key, value } => FactModification::Mul(key, value),
            FactModificationDef::Div { key, value } => FactModification::Div(key, value),
            FactModificationDef::Mod { key, value } => FactModification::Mod(key, value),
            FactModificationDef::Clamp { key, min, max } => FactModification::Clamp(key, min, max),
            FactModificationDef::Wrap { key, min, max } => FactModification::Wrap(key, min, max),
            FactModificationDef::Eval { key, expr } => FactModification::Eval(key, expr),
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
    ///
    /// Note: Action names are converted to lowercase to match the event format
    /// emitted by the FRE bridge.
    ///
    /// 注意：动作名被转换为小写以匹配 FRE 桥接发出的事件格式。
    pub fn to_event_id(&self) -> String {
        match self {
            RuleEventDef::Event(id) => id.clone(),
            RuleEventDef::ActionEvent { action, kind } => {
                let kind_str = match kind {
                    ActionEventKind::JustPressed => "just_pressed",
                    ActionEventKind::Pressed => "pressed",
                    ActionEventKind::JustReleased => "just_released",
                };
                // Convert action name to lowercase to match fre_bridge event format
                // 将动作名转换为小写以匹配 fre_bridge 的事件格式
                format!("action:{}:{}", action.to_lowercase(), kind_str)
            }
        }
    }
}

/// Serde helper: returns `true` for default boolean fields.
#[allow(dead_code)]
fn default_true() -> bool {
    true
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
    /// Enum variant — resolved to Int at load time via EnumRegistry.
    ///
    /// 枚举变体 — 加载时通过 EnumRegistry 解析为 Int。
    Enum(String),
}

// ============================================================================
// Serializable Rule Definition
// ============================================================================

/// A single rule definition that can be loaded from RON.
///
/// 可从 RON 加载的单个规则定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct RuleDef<A: ActionDef = CoreActionDef> {
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

    /// Actions to execute (data-driven, handled by game code).
    #[serde(default)]
    pub actions: Vec<A>,

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

    /// Whether executing this rule consumes the event (prevents other rules from matching).
    /// Defaults to true.
    ///
    /// 执行此规则是否消费事件（阻止其他规则匹配）。
    /// 默认为 true。
    #[serde(default = "default_consume_event")]
    pub consume_event: bool,
}

fn default_enabled() -> bool {
    true
}

fn default_consume_event() -> bool {
    true
}

impl<A: ActionDef> RuleDef<A> {
    /// Convert to a runtime Rule (without actions, which need game-specific handling).
    ///
    /// 转换为运行时 Rule（不含动作，动作需要游戏特定处理）。
    pub fn to_rule(&self) -> Rule<A> {
        self.to_rule_with_index(0, RuleScope::default())
    }

    /// Convert to a runtime Rule with an index suffix for unique ID generation
    /// and a scope inherited from the parent FreAsset.
    ///
    /// 转换为运行时 Rule，使用索引后缀生成唯一 ID 和从父 FreAsset 继承的作用域。
    pub fn to_rule_with_index(&self, index: usize, scope: RuleScope) -> Rule<A> {
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
            scope,
            trigger: FactEventId::new(self.event.to_event_id()),
            condition_expressions: self.conditions.clone(),
            modifications: self.modifications.iter().cloned().map(Into::into).collect(),
            outputs: self.outputs.iter().map(FactEventId::new).collect(),
            enabled: self.enabled,
            priority: self.priority,
            consume_event: self.consume_event,
            actions: self.actions.clone(),
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
#[derive(Asset, bevy::reflect::TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct FreAsset<A: ActionDef = CoreActionDef> {
    /// Scope for all rules in this asset.
    /// Defaults to Local if not specified.
    ///
    /// 此资产中所有规则的作用域。
    /// 如未指定，默认为 Local。
    #[serde(default)]
    pub scope: RuleScopeDef,

    /// Enum definitions — maps group names to ordered variant lists.
    /// Example: `{ "depth": ["main", "submenu", "options"] }` → main=0, submenu=1, options=2
    ///
    /// 枚举定义 — 将组名映射到有序变体列表。
    /// 示例：`{ "depth": ["main", "submenu", "options"] }` → main=0, submenu=1, options=2
    #[serde(default)]
    pub enums: HashMap<String, Vec<String>>,

    /// Facts to set when this asset is loaded.
    /// 加载此资产时设置的事实。
    #[serde(default)]
    pub facts: HashMap<String, FactValueDef>,

    /// The rules defined in this set.
    /// 此集合中定义的规则。
    #[serde(default)]
    pub rules: Vec<RuleDef<A>>,
}

/// Serializable rule scope for RON files.
///
/// RON 文件的可序列化规则作用域。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleScopeDef {
    /// Global rules - persist for app lifetime.
    Global,
    /// Local rules - cleared on scene exit (default).
    #[default]
    Local,
    /// View rules - cleared when View despawns.
    View,
}

impl From<RuleScopeDef> for RuleScope {
    fn from(def: RuleScopeDef) -> Self {
        match def {
            RuleScopeDef::Global => RuleScope::Global,
            RuleScopeDef::Local => RuleScope::Local,
            RuleScopeDef::View => RuleScope::View,
        }
    }
}

impl<A: ActionDef> FreAsset<A> {
    /// Get the scope for rules in this asset.
    ///
    /// 获取此资产中规则的作用域。
    pub fn scope(&self) -> RuleScope {
        self.scope.into()
    }

    /// Register all rules from this asset into a basic RuleRegistry.
    ///
    /// 将此资产中的所有规则注册到基础 RuleRegistry。
    pub fn register_rules(&self, registry: &mut RuleRegistry<A>) {
        let scope = self.scope();
        for (idx, rule_def) in self.rules.iter().enumerate() {
            let rule = rule_def.to_rule_with_index(idx, scope);
            info!(
                "FRE: Registering rule '{}' from asset (scope: {:?})",
                rule.id, scope
            );
            registry.register(rule);
        }
    }

    /// Register all rules from this asset into a LayeredRuleRegistry.
    /// Rules are automatically placed in the correct layer based on their scope.
    ///
    /// 将此资产中的所有规则注册到 LayeredRuleRegistry。
    /// 规则会根据其作用域自动放置到正确的层。
    pub fn register_rules_layered(&self, registry: &mut crate::rule::LayeredRuleRegistry<A>) {
        let scope = self.scope();
        for (idx, rule_def) in self.rules.iter().enumerate() {
            let rule = rule_def.to_rule_with_index(idx, scope);
            info!(
                "FRE: Registering rule '{}' from asset to layered registry (scope: {:?})",
                rule.id, scope
            );
            registry.register(rule);
        }
    }

    /// Get the facts defined in this asset.
    ///
    /// 获取此资产中定义的事实。
    pub fn get_facts(&self) -> &HashMap<String, FactValueDef> {
        &self.facts
    }

    /// Resolve all facts, converting Enum variants to Int using the asset's own enums field.
    /// Returns resolved (key, FactValue) pairs.
    ///
    /// 解析所有 facts，使用资产自身的 enums 字段将 Enum 变体转换为 Int。
    pub fn resolve_facts(&self, registry: &EnumRegistry) -> HashMap<String, FactValue> {
        self.facts
            .iter()
            .map(|(key, def)| {
                let value = registry.resolve_fact_value_def(key, def);
                (key.clone(), value)
            })
            .collect()
    }

    /// Get the rule definitions for custom action handling.
    ///
    /// 获取用于自定义动作处理的规则定义。
    pub fn get_rule_defs(&self) -> &[RuleDef<A>] {
        &self.rules
    }

    /// Get the enum definitions.
    ///
    /// 获取枚举定义。
    pub fn get_enums(&self) -> &HashMap<String, Vec<String>> {
        &self.enums
    }
}

// ============================================================================
// Enum Registry
// ============================================================================

/// Global registry for enum mappings.
/// Maps enum group names to variant name ↔ integer ID mappings.
///
/// 全局枚举注册表。
/// 将枚举组名映射到变体名 ↔ 整数 ID 映射。
///
/// # Example
/// ```ignore
/// // RON: enums: { "depth": ["main", "submenu", "options"] }
/// // → "main" = 0, "submenu" = 1, "options" = 2
/// registry.register("depth", &["main".into(), "submenu".into(), "options".into()]);
/// assert_eq!(registry.resolve("depth", "submenu"), Some(1));
/// ```
#[derive(Resource, Default, Debug, Clone)]
pub struct EnumRegistry {
    /// enum_group_name → { variant_name → integer_id }
    mappings: HashMap<String, HashMap<String, i64>>,
    /// enum_group_name → { integer_id → variant_name } (reverse mapping, for debug)
    reverse: HashMap<String, HashMap<i64, String>>,
}

impl EnumRegistry {
    /// Register a set of enum definitions from a FreAsset.
    ///
    /// 从 FreAsset 注册一组枚举定义。
    pub fn register_from_asset<A: ActionDef>(&mut self, asset: &FreAsset<A>) {
        for (group, variants) in &asset.enums {
            self.register(group, variants);
        }
    }

    /// Register enum variants for a group. Later registrations for the same group are merged.
    ///
    /// 为枚举组注册变体。同一组的后续注册会合并。
    pub fn register(&mut self, group: &str, variants: &[String]) {
        let forward: HashMap<String, i64> = variants
            .iter()
            .enumerate()
            .map(|(i, v)| (v.clone(), i as i64))
            .collect();
        let backward: HashMap<i64, String> = variants
            .iter()
            .enumerate()
            .map(|(i, v)| (i as i64, v.clone()))
            .collect();
        self.mappings.insert(group.to_string(), forward);
        self.reverse.insert(group.to_string(), backward);
    }

    /// Resolve an enum variant to its integer ID.
    ///
    /// 解析枚举变体为整数 ID。
    pub fn resolve(&self, group: &str, variant: &str) -> Option<i64> {
        self.mappings.get(group)?.get(variant).copied()
    }

    /// Reverse-resolve an integer ID to its variant name (for debug display).
    ///
    /// 将整数 ID 反向解析为变体名（用于调试显示）。
    pub fn reverse_resolve(&self, group: &str, id: i64) -> Option<&str> {
        self.reverse.get(group)?.get(&id).map(|s| s.as_str())
    }

    /// Resolve a FactValueDef::Enum to FactValue::Int using this registry.
    /// Non-Enum variants pass through unchanged.
    ///
    /// 使用此注册表将 FactValueDef::Enum 解析为 FactValue::Int。
    /// 非 Enum 变体原样传递。
    pub fn resolve_fact_value_def(&self, key: &str, def: &FactValueDef) -> FactValue {
        match def {
            FactValueDef::Enum(variant) => {
                if let Some(id) = self.resolve(key, variant) {
                    FactValue::Int(id)
                } else {
                    warn!(
                        "Unknown enum variant '{}' for group '{}', storing as String",
                        variant, key
                    );
                    FactValue::String(variant.clone())
                }
            }
            other => other.clone().into(),
        }
    }
}

// ============================================================================
// Asset Loader
// ============================================================================

/// Asset loader for .fre.ron files.
///
/// .fre.ron 文件的资产加载器。
pub struct FreAssetLoader<A: ActionDef = CoreActionDef>(std::marker::PhantomData<A>);

impl<A: ActionDef> Default for FreAssetLoader<A> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

// TypePath is required by Bevy's AssetLoader machinery.
impl<A: ActionDef> bevy::reflect::TypePath for FreAssetLoader<A> {
    fn type_path() -> &'static str {
        // Use a static path; concrete type info comes from the generic parameter's TypePath.
        "bevy_fact_rule_event::asset::FreAssetLoader"
    }

    fn short_type_path() -> &'static str {
        "FreAssetLoader"
    }
}

impl<A: ActionDef> AssetLoader for FreAssetLoader<A> {
    type Asset = FreAsset<A>;
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
            let asset = ron::de::from_bytes::<FreAsset<A>>(&bytes)?;
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
pub type ActionHandler<A> =
    Box<dyn Fn(&A, &crate::LayeredFactDatabase, &mut Commands) + Send + Sync>;

/// Registry for custom action handlers.
/// Games can register handlers for specific action types.
///
/// 自定义动作处理程序的注册表。
/// 游戏可以为特定动作类型注册处理程序。
pub struct ActionHandlerRegistry<A: ActionDef = CoreActionDef> {
    handlers: HashMap<String, ActionHandler<A>>,
}

impl<A: ActionDef> Default for ActionHandlerRegistry<A> {
    fn default() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
}

// Manual Resource impl to avoid requiring A: Default.
unsafe impl<A: ActionDef> Send for ActionHandlerRegistry<A> {}
unsafe impl<A: ActionDef> Sync for ActionHandlerRegistry<A> {}

impl<A: ActionDef> Resource for ActionHandlerRegistry<A> {}

impl<A: ActionDef> ActionHandlerRegistry<A> {
    /// Register a handler for a specific action type.
    ///
    /// 为特定动作类型注册处理程序。
    pub fn register<F>(&mut self, action_type: &str, handler: F)
    where
        F: Fn(&A, &crate::LayeredFactDatabase, &mut Commands) + Send + Sync + 'static,
    {
        self.handlers
            .insert(action_type.to_string(), Box::new(handler));
    }

    /// Check if a handler is registered for the given action type.
    ///
    /// 检查是否为给定的动作类型注册了处理程序。
    pub fn has_handler(&self, action_type: &str) -> bool {
        self.handlers.contains_key(action_type)
    }

    /// Execute an action using the registered handler.
    /// Uses `action.action_type()` for handler dispatch.
    ///
    /// 使用注册的处理程序执行动作。
    /// 使用 `action.action_type()` 进行处理程序分发。
    pub fn execute(&self, action: &A, db: &crate::LayeredFactDatabase, commands: &mut Commands) {
        let action_type = action.action_type();

        if let Some(handler) = self.handlers.get(action_type) {
            handler(action, db, commands);
        } else {
            warn!(
                "FRE: No handler registered for action type '{}'",
                action_type
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fre_asset_with_facts() {
        // Test FRE format with facts field
        let fre_data = r#"
(
    facts: {
        "counter": Int(0),
        "enabled": Bool(true),
    },
    rules: [
        (
            id: "test_rule",
            event: Event("test_event"),
            conditions: ["$counter == 3"],
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

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.facts.len(), 2);
        assert_eq!(asset.rules.len(), 1);
        assert_eq!(asset.rules[0].id, "test_rule");
        assert_eq!(asset.rules[0].event.to_event_id(), "test_event");
        assert_eq!(asset.rules[0].priority, 10);
    }

    #[test]
    fn test_fre_asset_action_event_format() {
        // Test FRE format with ActionEvent and CoreActionDef actions
        let fre_data = r#"
(
    rules: [
        (
            event: ActionEvent(action: "Up", kind: JustPressed),
            conditions: ["$selection > 0"],
            actions: [
                SetLocalFact("selection", Expr("$selection - 1")),
                Log(message: "moved up"),
            ],
        ),
        (
            event: ActionEvent(action: "Confirm", kind: JustPressed),
            conditions: ["$depth == 0"],
            actions: [
                SetLocalFact("depth", Int(1)),
                EmitEvent("confirmed"),
            ],
        ),
    ],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.rules.len(), 2);

        // First rule: ActionEvent Up (lowercase)
        assert_eq!(asset.rules[0].event.to_event_id(), "action:up:just_pressed");
        assert_eq!(asset.rules[0].conditions, vec!["$selection > 0"]);

        // Second rule: ActionEvent Confirm (lowercase)
        assert_eq!(
            asset.rules[1].event.to_event_id(),
            "action:confirm:just_pressed"
        );
        assert_eq!(asset.rules[1].conditions, vec!["$depth == 0"]);
    }

    #[test]
    fn test_fre_asset_string_event() {
        // Test FRE format with string Event
        let fre_data = r#"
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

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.rules.len(), 1);
        assert_eq!(asset.rules[0].event.to_event_id(), "custom_event");
    }

    #[test]
    fn test_local_fact_value_variants() {
        let fre_data = r#"
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
                SetLocalFact("enum_val", Enum("main")),
            ],
        ),
    ],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.rules[0].actions.len(), 6);
    }

    #[test]
    fn test_enum_registry_and_resolve_facts() {
        let fre_data = r#"
(
    scope: View,
    enums: {
        "depth": ["main", "submenu", "options"],
        "menu_context": ["fight", "act", "item", "mercy"],
    },
    facts: {
        "depth": Enum("main"),
        "menu_context": Enum("act"),
        "selection": Int(0),
    },
    rules: [],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.enums.len(), 2);
        assert_eq!(asset.enums["depth"], vec!["main", "submenu", "options"]);

        let mut registry = EnumRegistry::default();
        registry.register_from_asset(&asset);

        assert_eq!(registry.resolve("depth", "main"), Some(0));
        assert_eq!(registry.resolve("depth", "submenu"), Some(1));
        assert_eq!(registry.resolve("menu_context", "act"), Some(1));

        let resolved = asset.resolve_facts(&registry);
        assert_eq!(resolved["depth"], FactValue::Int(0));
        assert_eq!(resolved["menu_context"], FactValue::Int(1));
        assert_eq!(resolved["selection"], FactValue::Int(0));
    }

    #[test]
    fn test_fre_asset_with_actions_and_conditions() {
        let fre_data = r#"
(
    rules: [
        (
            id: "test_actions",
            event: Event("do_stuff"),
            conditions: ["$counter > 0"],
            actions: [
                Log(message: "test"),
                SetLocalFact("depth", Int(1)),
                EmitEvent("test_event"),
            ],
        ),
    ],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.rules.len(), 1);
        assert_eq!(asset.rules[0].actions.len(), 3);
        assert_eq!(asset.rules[0].actions[0].action_type(), "Log");
        assert_eq!(asset.rules[0].actions[1].action_type(), "SetLocalFact");
        assert_eq!(asset.rules[0].actions[2].action_type(), "EmitEvent");
    }
}
