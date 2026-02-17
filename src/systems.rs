//! # systems.rs
//!
//! Core systems for processing the FRE loop.
//!
//! FRE 循环处理的核心系统。

use crate::event::FactEvent;
use crate::layered::LayeredFactDatabase;
use crate::rule::{LayeredRuleRegistry, Rule};
use bevy::prelude::*;
use std::sync::Arc;

/// Resource to queue output events between systems.
/// Provides deduplication to prevent duplicate events from multiple rule processors.
///
/// 用于在系统之间排队输出事件的资源。
/// 提供去重功能，防止多个规则处理器产生重复事件。
#[derive(Resource, Default)]
pub struct PendingFactEvents {
    pub events: Vec<FactEvent>,
    /// Track rule IDs that have already emitted outputs this frame to avoid duplicates.
    ///
    /// 跟踪本帧已发出 outputs 的规则 ID，以避免重复。
    emitted_by_rule: std::collections::HashSet<String>,
}

impl PendingFactEvents {
    /// Queue an output event from a rule, with deduplication.
    /// Returns true if the event was queued, false if it was already queued by this rule.
    ///
    /// 从规则排队输出事件，带去重。
    /// 如果事件被排队返回 true，如果此规则已排队过则返回 false。
    pub fn queue_output(&mut self, rule_id: &str, event: FactEvent) -> bool {
        let key = format!("{}:{}", rule_id, event.id.0);
        if self.emitted_by_rule.contains(&key) {
            return false;
        }
        self.emitted_by_rule.insert(key);
        self.events.push(event);
        true
    }

    /// Clear the emitted tracking for the next frame.
    /// Called after events are drained.
    ///
    /// 清除发出跟踪以准备下一帧。
    /// 在事件被排空后调用。
    pub fn clear_tracking(&mut self) {
        self.emitted_by_rule.clear();
    }
}

/// Trait for evaluating rule condition expressions.
/// Implement this to provide custom condition evaluation logic.
///
/// 用于评估规则条件表达式的 trait。
/// 实现此 trait 以提供自定义条件评估逻辑。
pub trait ConditionEvaluatorTrait: Send + Sync + 'static {
    /// Evaluate all condition expressions for a rule.
    /// Returns true if all conditions pass or if there are no conditions.
    ///
    /// 评估规则的所有条件表达式。
    /// 如果所有条件都通过或没有条件，返回 true。
    fn evaluate(&self, conditions: &[String], facts: &LayeredFactDatabase) -> bool;
}

/// Default condition evaluator that always returns true (matches "Always" behavior).
///
/// 默认条件评估器，始终返回 true（匹配 "Always" 行为）。
#[derive(Default)]
pub struct DefaultConditionEvaluator;

impl ConditionEvaluatorTrait for DefaultConditionEvaluator {
    fn evaluate(&self, _conditions: &[String], _facts: &LayeredFactDatabase) -> bool {
        // Default: if no conditions, return true; otherwise also return true (no evaluation)
        // This maintains backward compatibility - rules without conditions always match
        true
    }
}

/// Resource that holds the condition evaluator function.
/// Games should replace this with their own evaluator that understands their expression syntax.
///
/// 持有条件评估器函数的资源。
/// 游戏应该用自己的评估器替换它，以理解其表达式语法。
#[derive(Resource)]
pub struct ConditionEvaluator {
    evaluator: Arc<dyn ConditionEvaluatorTrait>,
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self {
            evaluator: Arc::new(DefaultConditionEvaluator),
        }
    }
}

impl ConditionEvaluator {
    /// Create a new condition evaluator with a custom implementation.
    ///
    /// 使用自定义实现创建新的条件评估器。
    pub fn new<T: ConditionEvaluatorTrait>(evaluator: T) -> Self {
        Self {
            evaluator: Arc::new(evaluator),
        }
    }

    /// Evaluate conditions for a rule.
    ///
    /// 评估规则的条件。
    pub fn evaluate(&self, rule: &Rule, facts: &LayeredFactDatabase) -> bool {
        if rule.condition_expressions.is_empty() {
            return true; // No conditions = always match
        }
        self.evaluator.evaluate(&rule.condition_expressions, facts)
    }
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
    mut events: MessageReader<FactEvent>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    registry: Res<LayeredRuleRegistry>,
    mut pending_events: ResMut<PendingFactEvents>,
    condition_evaluator: Res<ConditionEvaluator>,
) {
    // Collect events to process
    let events_to_process: Vec<FactEvent> = events.read().cloned().collect();

    for event in events_to_process {
        // Get all rules grouped by priority
        let rule_groups = registry.get_matching_rules_grouped(&event);

        'outer: for group in rule_groups {
            for rule in group {
                // Evaluate condition expressions using the provided evaluator
                if !condition_evaluator.evaluate(rule, &layered_db) {
                    trace!("FRE: Rule '{}' skipped - conditions not met", rule.id);
                    continue;
                }

                info!(
                    "FRE: Rule '{}' triggered by event '{}' (priority: {}, conditions: {})",
                    rule.id,
                    event.id.0,
                    rule.priority,
                    rule.condition_expressions.len()
                );

                // Apply modifications to LayeredFactDatabase (local layer)
                for modification in &rule.modifications {
                    modification.apply(&mut layered_db);
                }

                // Queue output events for next frame (with deduplication)
                for output_id in &rule.outputs {
                    pending_events.queue_output(&rule.id, FactEvent::new(output_id.clone()));
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
    // Clear deduplication tracking for the next frame
    pending_events.clear_tracking();
}

/// Run condition: returns true if there are events to process.
/// 运行条件：如果有事件需要处理则返回 true。
pub fn has_fact_events(events: MessageReader<FactEvent>) -> bool {
    !events.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::FactValue;
    use crate::rule::{FactModification, Rule, RuleRegistry};

    #[test]
    fn test_rule_registry_matching() {
        let mut registry = RuleRegistry::new();

        let rule1 = Rule::builder("rule1", "event_a").build();

        let rule2 = Rule::builder("rule2", "event_b").build();

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
