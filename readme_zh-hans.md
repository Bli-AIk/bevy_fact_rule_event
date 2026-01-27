# bevy_fact_rule_event

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_fact_rule_event.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_fact_rule_event.svg"/> <br> <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中（v0.1.0）

**bevy_fact_rule_event** — Bevy 引擎的通用数据驱动事实-规则-事件（FRE）系统。

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 介绍

`bevy_fact_rule_event` 是一个数据驱动系统，通过声明式规则引擎将游戏逻辑与代码分离。  
它解决了复杂的硬编码游戏逻辑问题，让设计师能够通过外部数据文件定义行为，允许用户在不重新编译代码的情况下修改游戏行为。

使用 `bevy_fact_rule_event`，你只需要在 RON 文件中定义规则并将其作为资产加载 - 系统会根据游戏事件自动评估条件并执行操作。  
未来还计划支持可视化规则编辑器和实时规则热重载。

## 核心理念

> "事件不包含逻辑，数据不包含行为，逻辑只存在于规则中。"

FRE 系统强制实施关注点的清晰分离：
- **事实（Facts, F）**：集中式键值数据库，用于存储游戏状态
- **规则（Rules, R）**：声明式逻辑，基于条件转换状态
- **事件（Events, E）**：触发规则评估的信号广播

## 功能

* **数据驱动规则**：在 RON 文件中定义游戏逻辑，无需修改代码
* **集中式状态管理**：所有游戏事实存储在可查询的数据库中
* **条件逻辑**：支持嵌套逻辑运算符的复杂条件评估
* **自动资产加载**：与 Bevy 资产系统无缝集成
* **事件广播**：游戏系统之间的解耦通信
* **类型安全值**：支持 Int、Float、Bool 和 String 事实类型
* （计划中）可视化规则编辑器
* （计划中）热重载支持

## 使用方法

1. **安装 Rust**（如果尚未安装）：

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加到 Cargo.toml**：

   ```toml
   [dependencies]
   bevy_fact_rule_event = "0.1.0"
   ```

3. **基本使用**：

   ```rust
   use bevy::prelude::*;
   use bevy_fact_rule_event::prelude::*;

   fn main() {
       App::new()
           .add_plugins(DefaultPlugins)
           .add_plugins(FREPlugin)  // 添加 FRE 插件
           .add_systems(Startup, setup_rules)
           .run();
   }

   fn setup_rules(
       asset_server: Res<AssetServer>,
       mut commands: Commands,
   ) {
       // 从文件加载规则
       let rules_handle: Handle<RuleSetAsset> = asset_server.load("rules/game_rules.rule.ron");
       commands.spawn(rules_handle);
   }
   ```

4. **创建规则文件**（`assets/rules/game_rules.rule.ron`）：

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

5. **在游戏代码中发出事件**：

   ```rust
   fn player_collision_system(
       mut events: ResMut<PendingFactEvents>,
   ) {
       // 触发规则评估
       events.emit("player_hit");
   }
   ```

## 依赖

本项目使用以下 crate：

| Crate                                   | 版本     | 描述    |
|-----------------------------------------|--------|-------|
| [bevy](https://crates.io/crates/bevy)   | 0.17.2 | 游戏引擎  |
| [serde](https://crates.io/crates/serde) | 1.0    | 序列化框架 |

## 贡献指南

欢迎贡献！
无论你想修复错误、添加功能或改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可证

本项目可依据以下任意一种许可证进行分发：

* Apache License 2.0（[LICENSE-APACHE](LICENSE-APACHE)
  或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)）
* MIT License（[LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT)）

可任选其一。
