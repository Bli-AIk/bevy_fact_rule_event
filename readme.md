# bevy_fact_rule_event

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_fact_rule_event.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_fact_rule_event.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development (v0.1.0)

**bevy_fact_rule_event** — A generic, data-driven Fact-Rule-Event (FRE) system for Bevy engine.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

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
* **Centralized State Management**: All game facts stored in a queryable database
* **Conditional Logic**: Complex condition evaluation with nested logic operators
* **Automatic Asset Loading**: Seamless integration with Bevy's asset system
* **Event Broadcasting**: Decoupled communication between game systems
* **Type-Safe Values**: Support for Int, Float, Bool, and String fact types
* (Planned) Visual Rule Editor
* (Planned) Hot-Reloading Support

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add to Cargo.toml**:

   ```toml
   [dependencies]
   bevy_fact_rule_event = "0.1.0"
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
       let rules_handle: Handle<RuleSetAsset> = asset_server.load("rules/game_rules.rule.ron");
       commands.spawn(rules_handle);
   }
   ```

4. **Create a rule file** (`assets/rules/game_rules.rule.ron`):

   ```ron
   (
       version: 1,
       initial_facts: {
           "player_health": Int(100),
           "score": Int(0),
       },
       rules: [
           (
               id: "damage_player",
               trigger: "player_hit",
               condition: GreaterThan(key: "player_health", value: Int(0)),
               modifications: [
                   Decrement(key: "player_health", amount: 10),
               ],
               outputs: ["health_changed"],
           ),
           (
               id: "game_over",
               trigger: "health_changed",
               condition: LessEqual(key: "player_health", value: Int(0)),
               actions: ["GameOver"],
               outputs: ["game_ended"],
           ),
       ],
   )
   ```

5. **Emit events in your game code**:

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
| [bevy](https://crates.io/crates/bevy) | 0.17.2   | Game engine |
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
