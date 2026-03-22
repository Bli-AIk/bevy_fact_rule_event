//! # layered_registry.rs
//!
//! # layered_registry.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file extends the basic FRE registry with lifecycle-aware layers. It keeps global, local,
//! and per-view rule registries separate so callers can clear transient rule scopes without
//! disturbing rules that should survive across scenes or UI instances.
//!
//! 这个文件在基础 FRE 注册表之上增加了带生命周期语义的分层结构。它把 global、local 和
//! per-view 的规则注册表分开维护，这样调用方就能清理短生命周期的规则，而不影响应当跨场景或
//! 跨 UI 实例持续存在的规则。

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::{Entity, Resource, error, info};

use super::{ActionDef, CoreActionDef, FactEvent, Rule, RuleRegistry, RuleScope};

/// Layered rule registry that manages rules with different scopes.
/// Rules are separated into Global, Local, and View layers with different lifecycles.
///
/// 分层规则注册表，管理不同作用域的规则。
/// 规则按 Global、Local 和 View 层分离，具有不同的生命周期。
pub struct LayeredRuleRegistry<A: ActionDef = CoreActionDef> {
    global: RuleRegistry<A>,
    local: RuleRegistry<A>,
    view: HashMap<Entity, RuleRegistry<A>>,
}

impl<A: ActionDef> Default for LayeredRuleRegistry<A> {
    fn default() -> Self {
        Self {
            global: RuleRegistry::default(),
            local: RuleRegistry::default(),
            view: HashMap::new(),
        }
    }
}

impl<A: ActionDef> Resource for LayeredRuleRegistry<A> {}

impl<A: ActionDef> LayeredRuleRegistry<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, rule: Rule<A>) {
        match rule.scope {
            RuleScope::Global => self.global.register(rule),
            RuleScope::Local => self.local.register(rule),
            RuleScope::View => {
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

    pub fn register_view_rule(&mut self, view_entity: Entity, rule: Rule<A>) {
        self.view.entry(view_entity).or_default().register(rule);
    }

    pub fn clear_local(&mut self) {
        self.local.clear();
        info!("LayeredRuleRegistry: Cleared local layer rules");
    }

    pub fn clear_view(&mut self, view_entity: Entity) {
        if self.view.remove(&view_entity).is_some() {
            info!(
                "LayeredRuleRegistry: Cleared rules for view entity {:?}",
                view_entity
            );
        }
    }

    pub fn get_matching_rules_grouped(&self, event: &FactEvent) -> Vec<Vec<&Rule<A>>> {
        let mut all_groups: BTreeMap<i32, Vec<&Rule<A>>> = BTreeMap::new();

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
        for rule in self
            .view
            .values()
            .flat_map(|registry| registry.iter())
            .filter(|rule| rule.matches_event(event))
        {
            all_groups.entry(rule.priority).or_default().push(rule);
        }

        for group in all_groups.values_mut() {
            group.sort_by_key(|r| r.condition_expressions.len());
        }

        all_groups
            .into_iter()
            .rev()
            .map(|(_, rules)| rules)
            .collect()
    }

    pub fn get_matching_rules(&self, event: &FactEvent) -> Vec<&Rule<A>> {
        self.get_matching_rules_grouped(event)
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.global.len() + self.local.len() + self.view.values().map(|r| r.len()).sum::<usize>()
    }

    pub fn is_empty(&self) -> bool {
        self.global.is_empty() && self.local.is_empty() && self.view.values().all(|r| r.is_empty())
    }

    pub fn get(&self, rule_id: &str) -> Option<&Rule<A>> {
        self.global
            .get(rule_id)
            .or_else(|| self.local.get(rule_id))
            .or_else(|| self.view.values().find_map(|r| r.get(rule_id)))
    }

    pub fn global_iter(&self) -> impl Iterator<Item = &Rule<A>> {
        self.global.iter()
    }

    pub fn local_iter(&self) -> impl Iterator<Item = &Rule<A>> {
        self.local.iter()
    }

    pub fn view_iter(&self) -> impl Iterator<Item = (Entity, &RuleRegistry<A>)> {
        self.view
            .iter()
            .map(|(entity, registry)| (*entity, registry))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule<A>> {
        self.global
            .iter()
            .chain(self.local.iter())
            .chain(self.view.values().flat_map(|registry| registry.iter()))
    }
}
