//! # event.rs
//!
//! Event system for FRE - events are pure signals without logic.
//!
//! FRE 的事件系统 - 事件是不包含逻辑的纯信号。

use bevy::prelude::*;

/// Unique identifier for an event type.
///
/// 事件类型的唯一标识符。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactEventId(pub String);

impl FactEventId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for FactEventId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for FactEventId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A fact event - a signal that can trigger rules.
/// Events are pure data carriers with no logic.
///
/// 事实事件 - 可以触发规则的信号。
/// 事件是不包含逻辑的纯数据载体。
#[derive(Message, Debug, Clone)]
pub struct FactEvent {
    /// The unique identifier for this event type.
    ///
    /// 此事件类型的唯一标识符。
    pub id: FactEventId,

    /// Optional entity associated with this event.
    ///
    /// 与此事件关联的可选实体。
    pub entity: Option<Entity>,

    /// Optional additional data as key-value pairs.
    ///
    /// 作为键值对的可选附加数据。
    pub data: std::collections::HashMap<String, String>,
}

impl FactEvent {
    /// Create a new event with the given ID.
    ///
    /// 使用给定的 ID 创建新事件。
    pub fn new(id: impl Into<FactEventId>) -> Self {
        Self {
            id: id.into(),
            entity: None,
            data: std::collections::HashMap::new(),
        }
    }

    /// Create a new event with the given ID and entity.
    ///
    /// 使用给定的 ID 和实体创建新事件。
    pub fn with_entity(id: impl Into<FactEventId>, entity: Entity) -> Self {
        Self {
            id: id.into(),
            entity: Some(entity),
            data: std::collections::HashMap::new(),
        }
    }

    /// Add data to the event.
    ///
    /// 向事件添加数据。
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Get data from the event.
    ///
    /// 从事件获取数据。
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}
