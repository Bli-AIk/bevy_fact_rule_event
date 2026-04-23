//! # rule_defs.rs
//!
//! # rule_defs.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the asset-side schema for FRE rules. It contains the serializable rule and
//! asset structs, the scope definition used by `.fre.ron`, and the conversion helpers that turn
//! authored rules into runtime registry entries.
//!
//! 定义了 FRE 规则在资源侧的 schema。它包含可序列化的规则和资源结构、`.fre.ron`
//! 使用的作用域定义，以及把作者写下的规则转换成运行时注册表条目的辅助函数。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::database::FactValue;
use crate::event::FactEventId;
use crate::rule::{Rule, RuleRegistry, RuleScope};

use super::action_defs::{ActionDef, CoreActionDef};
use super::enum_registry::EnumRegistry;
use super::value_defs::{FactModificationDef, FactValueDef, RuleEventDef};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct RuleDef<A: ActionDef = CoreActionDef> {
    #[serde(default)]
    pub id: String,
    #[serde(alias = "trigger")]
    pub event: RuleEventDef,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub actions: Vec<A>,
    #[serde(default)]
    pub modifications: Vec<FactModificationDef>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub priority: i32,
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
    pub fn to_rule(&self) -> Rule<A> {
        self.to_rule_with_index(0, RuleScope::default())
    }

    pub fn to_rule_with_index(&self, index: usize, scope: RuleScope) -> Rule<A> {
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

#[derive(Asset, bevy::reflect::TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct FreAsset<A: ActionDef = CoreActionDef> {
    #[serde(default)]
    pub scope: RuleScopeDef,
    #[serde(default)]
    pub enums: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub facts: HashMap<String, FactValueDef>,
    #[serde(default)]
    pub rules: Vec<RuleDef<A>>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleScopeDef {
    Global,
    #[default]
    Local,
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
    pub fn scope(&self) -> RuleScope {
        self.scope.into()
    }

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

    pub fn get_facts(&self) -> &HashMap<String, FactValueDef> {
        &self.facts
    }

    pub fn resolve_facts(&self, registry: &EnumRegistry) -> HashMap<String, FactValue> {
        self.facts
            .iter()
            .map(|(key, def)| {
                let value = registry.resolve_fact_value_def(key, def);
                (key.clone(), value)
            })
            .collect()
    }

    pub fn get_rule_defs(&self) -> &[RuleDef<A>] {
        &self.rules
    }

    pub fn get_enums(&self) -> &HashMap<String, Vec<String>> {
        &self.enums
    }
}
