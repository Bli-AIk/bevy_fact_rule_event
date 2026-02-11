# bevy_fact_rule_event

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_fact_rule_event.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_fact_rule_event.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**bevy_fact_rule_event** — A generic, data-driven Fact-Rule-Event (FRE) system for Bevy engine.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Acknowledgements

This FRE system is deeply inspired by **aarthificial**'s brilliant work on data-driven game logic. We express our sincere gratitude and admiration for these foundational ideas:

- [📺 Dynamic Conversations - Legacy Devlog #23](https://www.youtube.com/watch?v=1LlF5p5Od6A) 
- [📺 Implicit Choices - Legacy Devlog #24](https://www.youtube.com/watch?v=yAIH7GGZ9L0)

## Introduction

`bevy_fact_rule_event` is a data-driven system that separates game logic from code through a declarative rule engine.  
It solves the problem of complex, hardcoded game logic by enabling designers to define behavior through external data files, allowing users to modify game behavior without recompiling code.

With `bevy_fact_rule_event`, you only need to define rules in RON files and load them as assets - the system automatically evaluates conditions and executes actions based on game events.  
In the future, it may also support visual rule editors and real-time rule hot-reloading.

## Core Philosophy

> "Events don't contain logic, data doesn't contain behavior, logic only exists in rules."

The FRE system enforces clean separation of concerns:
- **Facts (F)**: Centralized key-value database for game state
- **Rules (R)**: Declarative logic that transforms state based on conditions
- **Events (E)**: Signal broadcasts that trigger rule evaluation

## Features

* **Data-Driven Rules**: Define game logic in RON files without code changes
* **Layered Fact Database**: Hierarchical state management with Global and Local layers
  - Global layer: Persistent facts that survive scene changes (e.g., player stats)
  - Local layer: Temporary facts scoped to current context (e.g., battle state)
* **Centralized State Management**: All game facts stored in a queryable database
* **Conditional Logic**: Complex condition evaluation with nested logic operators
* **Automatic Asset Loading**: Seamless integration with Bevy's asset system
* **Event Broadcasting**: Decoupled communication between game systems
* **Type-Safe Values**: Support for Int, Float, Bool, and String fact types
* **Bidirectional Sync**: Facts can sync with ECS components for reactive UI updates
* (Planned) Visual Rule Editor
* (Planned) Hot-Reloading Support

## Bevy Version Support

| `bevy` | `bevy_fact_rule_event` |
|--------|------------------------|
| 0.18   | 0.3.0                  |
| 0.17   | < 0.3.0                |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   LayeredFactDatabase                        │
├─────────────────────────────────────────────────────────────┤
│  Global Layer (persistent)     │  Local Layer (temporary)   │
│  ─────────────────────────     │  ──────────────────────    │
│  • player_hp, player_lv        │  • battle_turn_count       │
│  • player_gold, player_name    │  • current_enemy_hp        │
│  • inventory_items             │  • dialogue_state          │
│                                │  (cleared on context exit) │
├─────────────────────────────────────────────────────────────┤
│              ↓ Read (local overrides global) ↓               │
│              ↑ Write (choose target layer)   ↑               │
└─────────────────────────────────────────────────────────────┘
            ↕                                      ↕
    ┌───────────────┐                    ┌──────────────────┐
    │  Rule Engine  │←── triggers ──────→│   Game Events    │
    │ (evaluates    │                    │ (FactEvent)      │
    │  conditions)  │                    └──────────────────┘
    └───────────────┘
```

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add to Cargo.toml**:

   ```toml
   [dependencies]
   bevy_fact_rule_event = "0.3.0"
   ```

3. **Basic usage**:

   ```rust
   use bevy::prelude::*;
   use bevy_fact_rule_event::prelude::*;

   fn main() {
       App::new()
           .add_plugins(DefaultPlugins)
           .add_plugins(FREPlugin)  // Add FRE plugin
           .add_systems(Startup, setup_rules)
           .run();
   }

   fn setup_rules(
       asset_server: Res<AssetServer>,
       mut commands: Commands,
   ) {
       // Load rules from file
       let rules_handle: Handle<FreAsset> = asset_server.load("rules/game_rules.fre.ron");
       commands.spawn(rules_handle);
   }
   ```

4. **Using the Layered Database**:

   ```rust
   fn update_player_stats(
       mut layered_db: ResMut<LayeredFactDatabase>,
   ) {
       // Write to global layer (persistent)
       layered_db.set_global("player_hp", 100i64);
       layered_db.set_global("player_name", "Chara".to_string());
       
       // Write to local layer (temporary, for current context)
       layered_db.set("battle_turn", 1i64);
       
       // Read (local layer takes priority over global)
       let hp = layered_db.get_int("player_hp").unwrap_or(20);
       let name = layered_db.get_string("player_name").unwrap_or("???");
       
       // Clear local layer when leaving context
       layered_db.clear_local();
   }
   ```

5. **Create a rule file** (`assets/rules/game_rules.fre.ron`):

   ```ron
   (
       facts: {
           "player_health": Int(100),
           "score": Int(0),
       },
       rules: [
           (
               id: "damage_player",
               event: Event("player_hit"),
               condition: GreaterThan(key: "player_health", value: Int(0)),
               modifications: [
                   Decrement(key: "player_health", amount: 10),
               ],
               outputs: ["health_changed"],
           ),
           (
               id: "game_over",
               event: Event("health_changed"),
               condition: LessEqual(key: "player_health", value: Int(0)),
               actions: ["GameOver"],
               outputs: ["game_ended"],
           ),
       ],
   )
   ```

6. **Emit events in your game code**:

   ```rust
   fn player_collision_system(
       mut events: ResMut<PendingFactEvents>,
   ) {
       // Trigger rule evaluation
       events.emit("player_hit");
   }
   ```

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [bevy](https://crates.io/crates/bevy) | 0.18   | Game engine |
| [serde](https://crates.io/crates/serde) | 1.0   | Serialization framework |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.