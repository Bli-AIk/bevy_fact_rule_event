//! # systems.rs
//!
//! Core systems for processing the FRE loop.
//!
//! FRE 循环处理的核心系统。

use crate::event::FactEvent;
use crate::layered::LayeredFactDatabase;
use crate::rule::LayeredRuleRegistry;
use bevy::prelude::*;

/// Resource to queue output events between systems.
///
/// 用于在系统之间排队输出事件的资源。
#[derive(Resource, Default)]
pub struct PendingFactEvents {
    pub events: Vec<FactEvent>,
}

/// Main system for processing the FRE loop using LayeredFactDatabase and LayeredRuleRegistry:
/// Listen to Events -> Find matching Rules (grouped by priority) -> Check Fact conditions
/// -> Execute Actions/Modifications -> Queue output Events
///
/// Priority and matching rules:
/// 1. Rules are grouped by priority (higher priority groups checked first)
/// 2. Within each group, rules are sorted by condition count (fewer conditions first)
/// 3. When a rule matches and consumes the event, no more rules are checked
/// 4. When a rule matches but doesn't consume the event, continue checking in the same group
///
/// 使用 LayeredFactDatabase 和 LayeredRuleRegistry 处理 FRE 循环的主系统：
/// 监听事件 -> 查找匹配规则（按优先级分组）-> 检查事实条件
/// -> 执行动作/修改 -> 排队输出事件
///
/// 优先级和匹配规则：
/// 1. 规则按优先级分组（高优先级组先检查）
/// 2. 每组内按条件数量排序（条件少的先匹配）
/// 3. 当规则匹配并消费事件时，不再检查更多规则
/// 4. 当规则匹配但不消费事件时，继续检查同一组内的规则
pub fn process_rules_system(
    mut commands: Commands,
    mut events: MessageReader<FactEvent>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    registry: Res<LayeredRuleRegistry>,
    mut pending_events: ResMut<PendingFactEvents>,
) {
    // Collect events to process
    let events_to_process: Vec<FactEvent> = events.read().cloned().collect();

    for event in events_to_process {
        // Get all rules grouped by priority
        let rule_groups = registry.get_matching_rules_grouped(&event);

        'outer: for group in rule_groups {
            for rule in group {
                // Check condition using LayeredFactDatabase (local-first, then global)
                if !rule.check_condition(&*layered_db) {
                    continue;
                }

                info!(
                    "FRE: Rule '{}' triggered by event '{}' (priority: {}, conditions: {})",
                    rule.id,
                    event.id.0,
                    rule.priority,
                    rule.condition_expressions.len()
                );

                // Execute actions
                for action in &rule.actions {
                    action.execute(&event, &layered_db, &mut commands);
                }

                // Apply modifications to LayeredFactDatabase (local layer)
                for modification in &rule.modifications {
                    modification.apply(&mut layered_db);
                }

                // Queue output events for next frame
                for output_id in &rule.outputs {
                    pending_events
                        .events
                        .push(FactEvent::new(output_id.clone()));
                }

                // If this rule consumes the event, stop processing all rules
                if rule.consume_event {
                    break 'outer;
                }
                // Otherwise, continue checking rules in this priority group
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
    use crate::rule::{FactModification, Rule, RuleCondition, RuleRegistry};

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
        let mut db = LayeredFactDatabase::new();

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
