//! # systems.rs
//!
//! Core systems for processing the FRE loop.
//!
//! FRE 循环处理的核心系统。

use crate::database::FactDatabase;
use crate::event::FactEvent;
use crate::rule::RuleRegistry;
use bevy::prelude::*;

/// Resource to queue output events between systems.
///
/// 用于在系统之间排队输出事件的资源。
#[derive(Resource, Default)]
pub struct PendingFactEvents {
    pub events: Vec<FactEvent>,
}

/// Main system for processing the FRE loop:
/// Listen to Events -> Find matching Rules -> Check Fact conditions -> Execute Actions/Modifications -> Queue output Events
///
/// 处理 FRE 循环的主系统：
/// 监听事件 -> 查找匹配规则 -> 检查事实条件 -> 执行动作/修改 -> 排队输出事件
pub fn process_rules_system(
    mut commands: Commands,
    mut events: MessageReader<FactEvent>,
    mut db: ResMut<FactDatabase>,
    mut registry: ResMut<RuleRegistry>,
    mut pending_events: ResMut<PendingFactEvents>,
) {
    // Collect events to process
    let events_to_process: Vec<FactEvent> = events.read().cloned().collect();

    for event in events_to_process {
        // Get all rules that match this event
        let matching_rules: Vec<_> = registry
            .get_matching_rules(&event)
            .into_iter()
            .cloned()
            .collect();

        for rule in matching_rules {
            // Check condition
            if !rule.check_condition(&db) {
                continue;
            }

            info!(
                "FRE: Rule '{}' triggered by event '{}'",
                rule.id, event.id.0
            );

            // Execute actions
            for action in &rule.actions {
                action.execute(&event, &db, &mut commands);
            }

            // Apply modifications
            for modification in &rule.modifications {
                modification.apply(&mut db);
            }

            // Queue output events for next frame
            for output_id in &rule.outputs {
                pending_events
                    .events
                    .push(FactEvent::new(output_id.clone()));
            }
        }
    }
}

/// System to emit pending events from the previous frame.
///
/// 发出上一帧待处理事件的系统。
pub fn emit_pending_events_system(
    mut pending_events: ResMut<PendingFactEvents>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    for event in pending_events.events.drain(..) {
        event_writer.write(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::FactValue;
    use crate::rule::{FactModification, Rule, RuleCondition};

    #[test]
    fn test_rule_registry_matching() {
        let mut registry = RuleRegistry::new();

        let rule1 = Rule::builder("rule1", "event_a")
            .condition(RuleCondition::Always)
            .build();

        let rule2 = Rule::builder("rule2", "event_b")
            .condition(RuleCondition::Always)
            .build();

        registry.register(rule1);
        registry.register(rule2);

        let event_a = FactEvent::new("event_a");
        let matching = registry.get_matching_rules(&event_a);

        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, "rule1");
    }

    #[test]
    fn test_fact_modification_apply() {
        let mut db = FactDatabase::new();

        FactModification::Set("counter".to_string(), FactValue::Int(0)).apply(&mut db);
        assert_eq!(db.get_int("counter"), Some(0));

        FactModification::Increment("counter".to_string(), 5).apply(&mut db);
        assert_eq!(db.get_int("counter"), Some(5));

        FactModification::Toggle("flag".to_string()).apply(&mut db);
        assert_eq!(db.get_bool("flag"), Some(true));

        FactModification::Toggle("flag".to_string()).apply(&mut db);
        assert_eq!(db.get_bool("flag"), Some(false));
    }
}
