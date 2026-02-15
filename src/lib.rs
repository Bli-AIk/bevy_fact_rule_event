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
//!
//! ## Data-Driven Rules
//! Rules can be defined in RON files and loaded as assets:
//! ```ron
//! (
//!     facts: {
//!         "counter": Int(0),
//!     },
//!     rules: [
//!         (
//!             id: "increment_counter",
//!             trigger: "button_pressed",
//!             condition: Always,
//!             modifications: [
//!                 Increment(key: "counter", amount: 1),
//!             ],
//!             outputs: ["counter_updated"],
//!         ),
//!     ],
//! )
//! ```

pub mod asset;
mod database;
mod event;
mod layered;
mod rule;
mod systems;

pub use asset::{
    ActionEventKind, ActionHandlerRegistry, FactModificationDef, FactValueDef, FreAsset,
    FreAssetLoader, LocalFactValue, RuleActionDef, RuleConditionDef, RuleDef, RuleEventDef,
    RuleScopeDef,
};

pub use database::{FactDatabase, FactKey, FactReader, FactValue};
pub use event::{FactEvent, FactEventId};
pub use layered::LayeredFactDatabase;
pub use rule::{
    FactModification, LayeredRuleRegistry, Rule, RuleAction, RuleCondition, RuleRegistry, RuleScope,
};
pub use systems::PendingFactEvents;

use bevy::asset::AssetApp;
use bevy::prelude::*;

/// Main plugin for the FRE system.
///
/// FRE 系统的主插件。
pub struct FREPlugin;

impl Plugin for FREPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LayeredFactDatabase>()
            .init_resource::<LayeredRuleRegistry>()
            .init_resource::<ActionHandlerRegistry>()
            .init_resource::<systems::PendingFactEvents>()
            .init_asset::<FreAsset>()
            .register_asset_loader(FreAssetLoader)
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
