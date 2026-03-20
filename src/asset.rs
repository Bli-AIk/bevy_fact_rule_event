//! Data-driven rule definitions that can be loaded from RON files.
//!
//! 可从 RON 文件加载的数据驱动规则定义。

mod action_defs;
mod enum_registry;
mod loader;
mod rule_defs;
mod value_defs;

pub use action_defs::{ActionDef, CoreActionDef};
pub use enum_registry::EnumRegistry;
pub use loader::{ActionHandler, ActionHandlerRegistry, FreAssetLoader};
pub use rule_defs::{FreAsset, RuleDef, RuleScopeDef};
pub use value_defs::{
    ActionEventKind, FactModificationDef, FactValueDef, LocalFactValue, RuleEventDef,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::FactValue;

    #[test]
    fn test_fre_asset_with_facts() {
        let fre_data = r#"
(
    facts: {
        "counter": Int(0),
        "enabled": Bool(true),
    },
    rules: [
        (
            id: "test_rule",
            event: Event("test_event"),
            conditions: ["$counter == 3"],
            modifications: [
                Set(key: "triggered", value: Bool(true)),
                Increment(key: "counter", amount: 1),
            ],
            outputs: ["result_event"],
            priority: 10,
        ),
    ],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.facts.len(), 2);
        assert_eq!(asset.rules.len(), 1);
        assert_eq!(asset.rules[0].id, "test_rule");
        assert_eq!(asset.rules[0].event.to_event_id(), "test_event");
        assert_eq!(asset.rules[0].priority, 10);
    }

    #[test]
    fn test_fre_asset_action_event_format() {
        let fre_data = r#"
(
    rules: [
        (
            event: ActionEvent(action: "Up", kind: JustPressed),
            conditions: ["$selection > 0"],
            actions: [
                SetLocalFact("selection", Expr("$selection - 1")),
                Log(message: "moved up"),
            ],
        ),
        (
            event: ActionEvent(action: "Confirm", kind: JustPressed),
            conditions: ["$depth == 0"],
            actions: [
                SetLocalFact("depth", Int(1)),
                EmitEvent("confirmed"),
            ],
        ),
    ],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.rules.len(), 2);
        assert_eq!(asset.rules[0].event.to_event_id(), "action:up:just_pressed");
        assert_eq!(asset.rules[0].conditions, vec!["$selection > 0"]);
        assert_eq!(
            asset.rules[1].event.to_event_id(),
            "action:confirm:just_pressed"
        );
        assert_eq!(asset.rules[1].conditions, vec!["$depth == 0"]);
    }

    #[test]
    fn test_fre_asset_string_event() {
        let fre_data = r#"
(
    rules: [
        (
            event: Event("custom_event"),
            actions: [
                Log(message: "Custom event fired"),
            ],
        ),
    ],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.rules.len(), 1);
        assert_eq!(asset.rules[0].event.to_event_id(), "custom_event");
    }

    #[test]
    fn test_local_fact_value_variants() {
        let fre_data = r#"
(
    rules: [
        (
            event: Event("test"),
            actions: [
                SetLocalFact("int_val", Int(42)),
                SetLocalFact("float_val", Float(3.14)),
                SetLocalFact("bool_val", Bool(true)),
                SetLocalFact("str_val", String("hello")),
                SetLocalFact("expr_val", Expr("$x + 1")),
                SetLocalFact("enum_val", Enum("main")),
            ],
        ),
    ],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.rules[0].actions.len(), 6);
    }

    #[test]
    fn test_enum_registry_and_resolve_facts() {
        let fre_data = r#"
(
    scope: View,
    enums: {
        "depth": ["main", "submenu", "options"],
        "menu_context": ["fight", "act", "item", "mercy"],
    },
    facts: {
        "depth": Enum("main"),
        "menu_context": Enum("act"),
        "selection": Int(0),
    },
    rules: [],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.enums.len(), 2);
        assert_eq!(asset.enums["depth"], vec!["main", "submenu", "options"]);

        let mut registry = EnumRegistry::default();
        registry.register_from_asset(&asset);

        assert_eq!(registry.resolve("depth", "main"), Some(0));
        assert_eq!(registry.resolve("depth", "submenu"), Some(1));
        assert_eq!(registry.resolve("menu_context", "act"), Some(1));

        let resolved = asset.resolve_facts(&registry);
        assert_eq!(resolved["depth"], FactValue::Int(0));
        assert_eq!(resolved["menu_context"], FactValue::Int(1));
        assert_eq!(resolved["selection"], FactValue::Int(0));
    }

    #[test]
    fn test_fre_asset_with_actions_and_conditions() {
        let fre_data = r#"
(
    rules: [
        (
            id: "test_actions",
            event: Event("do_stuff"),
            conditions: ["$counter > 0"],
            actions: [
                Log(message: "test"),
                SetLocalFact("depth", Int(1)),
                EmitEvent("test_event"),
            ],
        ),
    ],
)
"#;

        let asset: FreAsset = ron::from_str(fre_data).unwrap();
        assert_eq!(asset.rules.len(), 1);
        assert_eq!(asset.rules[0].actions.len(), 3);
        assert_eq!(asset.rules[0].actions[0].action_type(), "Log");
        assert_eq!(asset.rules[0].actions[1].action_type(), "SetLocalFact");
        assert_eq!(asset.rules[0].actions[2].action_type(), "EmitEvent");
    }
}
