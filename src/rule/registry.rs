//! # registry.rs
//!
//! # registry.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file contains the core in-memory registry for FRE rules. It stores rules by id, maintains
//! a priority-sorted cache for fast event matching, and exposes the operations needed to register,
//! remove, enable, and iterate rules at runtime.
//!
//! 这个文件包含 FRE 规则的核心内存注册表。它按 id 存储规则，维护一个按优先级排序的缓存以便
//! 快速匹配事件，并提供运行时注册、移除、启用和遍历规则所需的操作。

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::Resource;

use super::{ActionDef, CoreActionDef, FactEvent, Rule};

fn compare_by_priority<A: ActionDef>(a: &Rule<A>, b: &Rule<A>) -> std::cmp::Ordering {
    b.priority.cmp(&a.priority).then_with(|| {
        a.condition_expressions
            .len()
            .cmp(&b.condition_expressions.len())
    })
}

/// Registry for storing and managing rules.
///
/// 用于存储和管理规则的注册表。
pub struct RuleRegistry<A: ActionDef = CoreActionDef> {
    rules: HashMap<String, Rule<A>>,
    /// Rules sorted by priority (cached).
    ///
    /// 按优先级排序的规则（缓存）。
    sorted_rules: Vec<String>,
    dirty: bool,
}

impl<A: ActionDef> Default for RuleRegistry<A> {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
            sorted_rules: Vec::new(),
            dirty: false,
        }
    }
}

impl<A: ActionDef> Resource for RuleRegistry<A> {}

impl<A: ActionDef> RuleRegistry<A> {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            sorted_rules: Vec::new(),
            dirty: false,
        }
    }

    pub fn register(&mut self, rule: Rule<A>) {
        self.rules.insert(rule.id.clone(), rule);
        self.dirty = true;
    }

    pub fn unregister(&mut self, rule_id: &str) -> Option<Rule<A>> {
        let rule = self.rules.remove(rule_id);
        if rule.is_some() {
            self.dirty = true;
        }
        rule
    }

    pub fn get(&self, rule_id: &str) -> Option<&Rule<A>> {
        self.rules.get(rule_id)
    }

    pub fn get_mut(&mut self, rule_id: &str) -> Option<&mut Rule<A>> {
        self.rules.get_mut(rule_id)
    }

    pub fn set_enabled(&mut self, rule_id: &str, enabled: bool) {
        if let Some(rule) = self.rules.get_mut(rule_id) {
            rule.enabled = enabled;
        }
    }

    pub fn get_matching_rules_grouped(&self, event: &FactEvent) -> Vec<Vec<&Rule<A>>> {
        let mut groups: BTreeMap<i32, Vec<&Rule<A>>> = BTreeMap::new();

        for rule in self.rules.values() {
            if rule.matches_event(event) {
                groups.entry(rule.priority).or_default().push(rule);
            }
        }

        for group in groups.values_mut() {
            group.sort_by_key(|r| r.condition_expressions.len());
        }

        groups.into_iter().rev().map(|(_, rules)| rules).collect()
    }

    pub fn get_matching_rules(&mut self, event: &FactEvent) -> Vec<&Rule<A>> {
        if self.dirty {
            self.sorted_rules = self.rules.keys().cloned().collect();
            self.sorted_rules
                .sort_by(|a, b| match (self.rules.get(a), self.rules.get(b)) {
                    (Some(a), Some(b)) => compare_by_priority(a, b),
                    _ => std::cmp::Ordering::Equal,
                });
            self.dirty = false;
        }

        self.sorted_rules
            .iter()
            .filter_map(|id| self.rules.get(id))
            .filter(|rule| rule.matches_event(event))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn clear(&mut self) {
        self.rules.clear();
        self.sorted_rules.clear();
        self.dirty = false;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule<A>> {
        self.rules.values()
    }
}
