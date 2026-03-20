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
