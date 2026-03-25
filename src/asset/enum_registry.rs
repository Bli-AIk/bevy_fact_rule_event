//! # enum_registry.rs
//!
//! # enum_registry.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Provides the enum registry used when FRE assets want symbolic variants in their fact
//! definitions. It stores both forward and reverse mappings and converts schema-level enum values
//! into concrete runtime `FactValue`s.
//!
//! 提供了 FRE 资产在事实定义里使用符号化枚举值时所依赖的枚举注册表。它同时保存正向
//! 和反向映射，并把 schema 层的枚举值转换成真正的运行时 `FactValue`。

use bevy::prelude::*;
use std::collections::HashMap;

use crate::database::FactValue;

use super::action_defs::ActionDef;
use super::rule_defs::FreAsset;
use super::value_defs::FactValueDef;

#[derive(Resource, Default, Debug, Clone)]
pub struct EnumRegistry {
    mappings: HashMap<String, HashMap<String, i64>>,
    reverse: HashMap<String, HashMap<i64, String>>,
}

impl EnumRegistry {
    pub fn register_from_asset<A: ActionDef>(&mut self, asset: &FreAsset<A>) {
        for (group, variants) in &asset.enums {
            self.register(group, variants);
        }
    }

    pub fn register(&mut self, group: &str, variants: &[String]) {
        let forward: HashMap<String, i64> = variants
            .iter()
            .enumerate()
            .map(|(i, variant)| (variant.clone(), i as i64))
            .collect();
        let backward: HashMap<i64, String> = variants
            .iter()
            .enumerate()
            .map(|(i, variant)| (i as i64, variant.clone()))
            .collect();
        self.mappings.insert(group.to_string(), forward);
        self.reverse.insert(group.to_string(), backward);
    }

    pub fn resolve(&self, group: &str, variant: &str) -> Option<i64> {
        self.mappings.get(group)?.get(variant).copied()
    }

    pub fn reverse_resolve(&self, group: &str, id: i64) -> Option<&str> {
        self.reverse
            .get(group)?
            .get(&id)
            .map(|value| value.as_str())
    }

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
