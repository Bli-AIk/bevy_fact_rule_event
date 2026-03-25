//! # action_defs.rs
//!
//! # action_defs.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the serializable action shapes that can appear inside FRE assets. It provides
//! the generic `ActionDef` trait used across the crate, plus the built-in `CoreActionDef` enum
//! that covers logging, local fact updates, event emission, and custom host actions.
//!
//! 定义了 FRE 资产里可序列化的动作形状。它提供整个 crate 共用的泛型 `ActionDef`
//! trait，以及内置的 `CoreActionDef` 枚举，用来表示日志、本地事实更新、事件发射和宿主自定义动作。

use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::value_defs::LocalFactValue;

pub trait ActionDef:
    std::fmt::Debug + Clone + Send + Sync + Serialize + serde::de::DeserializeOwned + TypePath + 'static
{
    fn action_type(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub enum CoreActionDef {
    Log {
        message: String,
    },
    SetLocalFact(String, LocalFactValue),
    EmitEvent(String),
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
