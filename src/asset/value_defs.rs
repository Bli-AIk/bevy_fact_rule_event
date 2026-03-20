use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::FactValue;
use crate::rule::FactModification;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactValueDef {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    StringList(Vec<String>),
    IntList(Vec<i64>),
    Enum(String),
}

impl From<FactValueDef> for FactValue {
    fn from(def: FactValueDef) -> Self {
        match def {
            FactValueDef::Int(value) => FactValue::Int(value),
            FactValueDef::Float(value) => FactValue::Float(value),
            FactValueDef::Bool(value) => FactValue::Bool(value),
            FactValueDef::String(value) => FactValue::String(value),
            FactValueDef::StringList(value) => FactValue::StringList(value),
            FactValueDef::IntList(value) => FactValue::IntList(value),
            FactValueDef::Enum(variant) => {
                warn!(
                    "FactValueDef::Enum('{}') converted without EnumRegistry — stored as String",
                    variant
                );
                FactValue::String(variant)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactModificationDef {
    Set { key: String, value: FactValueDef },
    Increment { key: String, amount: i64 },
    Add { key: String, value: f64 },
    Sub { key: String, value: f64 },
    Mul { key: String, value: f64 },
    Div { key: String, value: f64 },
    Mod { key: String, value: i64 },
    Clamp { key: String, min: f64, max: f64 },
    Wrap { key: String, min: i64, max: i64 },
    Eval { key: String, expr: String },
    Remove(String),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionEventKind {
    JustPressed,
    Pressed,
    JustReleased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleEventDef {
    Event(String),
    ActionEvent {
        action: String,
        kind: ActionEventKind,
    },
}

impl Default for RuleEventDef {
    fn default() -> Self {
        RuleEventDef::Event(String::new())
    }
}

impl RuleEventDef {
    pub fn to_event_id(&self) -> String {
        match self {
            RuleEventDef::Event(id) => id.clone(),
            RuleEventDef::ActionEvent { action, kind } => {
                let kind_str = match kind {
                    ActionEventKind::JustPressed => "just_pressed",
                    ActionEventKind::Pressed => "pressed",
                    ActionEventKind::JustReleased => "just_released",
                };
                format!("action:{}:{}", action.to_lowercase(), kind_str)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalFactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Expr(String),
    Enum(String),
}
