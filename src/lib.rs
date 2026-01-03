//! # bevy_fact_rule_event
//!
//! A generic, data-driven Fact-Rule-Event (FRE) system for Bevy.
//!
//! ## Core Philosophy
//! "Events don't contain logic, data doesn't contain behavior, logic only exists in rules."
//!
//! ## Architecture
//! - **Fact (F)**: Centralized key-value database for game state
//! - **Rule (R)**: Declarative rules with triggers, conditions, actions, and modifications
//! - **Event (E)**: Signal broadcasts that trigger rule evaluation

mod database;
mod event;
mod rule;
mod systems;

pub use database::{FactDatabase, FactKey, FactValue};
pub use event::{FactEvent, FactEventId};
pub use rule::{FactModification, Rule, RuleAction, RuleCondition, RuleRegistry};

use bevy::prelude::*;

/// Main plugin for the FRE system.
///
/// FRE 系统的主插件。
pub struct FREPlugin;

impl Plugin for FREPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactDatabase>()
            .init_resource::<RuleRegistry>()
            .init_resource::<systems::PendingFactEvents>()
            .add_message::<FactEvent>()
            .add_systems(
                Update,
                (
                    systems::emit_pending_events_system,
                    systems::process_rules_system,
                )
                    .chain(),
            );
    }
}
