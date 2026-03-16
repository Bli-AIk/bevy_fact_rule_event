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
pub mod expr;
mod layered;
mod rule;
mod systems;

pub use asset::{
    ActionDef, ActionEventKind, ActionHandlerRegistry, CoreActionDef, EnumRegistry,
    FactModificationDef, FactValueDef, FreAsset, FreAssetLoader, LocalFactValue, RuleDef,
    RuleEventDef, RuleScopeDef,
};

pub use database::{CombinedFactReader, FactDatabase, FactReader, FactValue};
pub use event::{FactEvent, FactEventId};
pub use layered::LayeredFactDatabase;
pub use rule::{FactModification, LayeredRuleRegistry, Rule, RuleRegistry, RuleScope};
pub use systems::{ConditionEvaluator, ConditionEvaluatorTrait, PendingFactEvents};

use bevy::asset::AssetApp;
use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::prelude::*;

/// Main plugin for the FRE system.
///
/// FRE 系统的主插件。
pub struct FREPlugin<A: ActionDef = CoreActionDef> {
    pub schedule: Option<InternedScheduleLabel>,
    _marker: std::marker::PhantomData<A>,
}

impl<A: ActionDef> Default for FREPlugin<A> {
    fn default() -> Self {
        Self {
            schedule: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<A: ActionDef> Plugin for FREPlugin<A> {
    fn build(&self, app: &mut App) {
        let schedule = self.schedule.unwrap_or(Update.intern());
        app.init_resource::<LayeredFactDatabase>()
            .init_resource::<LayeredRuleRegistry<A>>()
            .init_resource::<ActionHandlerRegistry<A>>()
            .init_resource::<EnumRegistry>()
            .init_resource::<PendingFactEvents>()
            .init_resource::<ConditionEvaluator>()
            .init_asset::<FreAsset<A>>()
            .register_asset_loader(FreAssetLoader::<A>::default())
            .add_message::<FactEvent>()
            .add_systems(
                schedule,
                (
                    systems::emit_pending_events_system,
                    systems::process_rules_system::<A>.run_if(systems::has_fact_events),
                )
                    .chain(),
            );
    }
}
