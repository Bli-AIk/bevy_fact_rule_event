# bevy_fact_rule_event

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_fact_rule_event.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_fact_rule_event.svg"/> <br> <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**bevy_fact_rule_event** — Bevy 引擎的通用数据驱动事实-规则-事件（FRE）系统。

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 致谢

整个 FRE 系统在极大程度上受到了 **aarthificial** 在数据驱动游戏逻辑方面卓越工作的启发。我们对这些奠基性的想法表示由衷的感激与敬畏：

- [📺 Dynamic Conversations - Legacy Devlog #23](https://www.youtube.com/watch?v=1LlF5p5Od6A) 
- [📺 Implicit Choices - Legacy Devlog #24](https://www.youtube.com/watch?v=yAIH7GGZ9L0)

## 介绍

`bevy_fact_rule_event` 是一个数据驱动系统，通过声明式规则引擎将游戏逻辑与代码分离。  
它解决了复杂的硬编码游戏逻辑问题，让设计师能够通过外部数据文件定义行为，允许用户在不重新编译代码的情 况下修改游戏行为。

使用 `bevy_fact_rule_event`，你只需要在 RON 文件中定义规则并将其作为资源加载 - 系统会根据游戏事件自动评估条件并执行操作。  
未来还计划支持可视化规则编辑器和实时规则热重载。

## 核心理念

> "事件不包含逻辑，数据不包含行为，逻辑只存在于规则中。"

FRE 系统强制实施关注点的清晰分离：
- **事实（Facts, F）**：集中式键值数据库，用于存储游戏状态
- **规则（Rules, R）**：声明式逻辑，基于条件转换状态
- **事件（Events, E）**：触发规则评估的信号广播

## 功能

* **数据驱动规则**：在 RON 文件中定义游戏逻辑，无需修改代码
* **分层事实数据库**：支持全局层和局部层的层级状态管理
  - 全局层：在场景切换后仍然保留的持久化事实（如玩家属性）
  - 局部层：作用于当前上下文的临时事实（如战斗状态）
* **集中式状态管理**：所有游戏事实存储在可查询的数据库中
* **条件逻辑**：支持嵌套逻辑运算符的复杂条件评估
* **自动资源加载**：与 Bevy 资源系统无缝集成
* **事件广播**：游戏系统之间的解耦通信
* **类型安全值**：支持 Int、Float、Bool 和 String 事实类型
* **双向同步**：事实可以与 ECS 组件同步，实现响应式 UI 更新
* （计划中）可视化规则编辑器
* （计划中）热重载支持

## Bevy 版本支持

| `bevy` | `bevy_fact_rule_event` |
|--------|------------------------|
| 0.18   | 0.3.0                  |
| 0.17   | < 0.3.0                |

## 架构概述

```
┌─────────────────────────────────────────────────────────────┐
│                   LayeredFactDatabase                        │
│                   （分层事实数据库）                           │
├─────────────────────────────────────────────────────────────┤
│  全局层（持久化）              │  局部层（临时）                │
│  ─────────────────────────     │  ──────────────────────     │
│  • player_hp, player_lv        │  • battle_turn_count        │
│  • player_gold, player_name    │  • current_enemy_hp         │
│  • inventory_items             │  • dialogue_state           │
│                                │  （退出上下文时清空）          │
├─────────────────────────────────────────────────────────────┤
│           ↓ 读取（局部层优先于全局层）↓                        │
│           ↑ 写入（选择目标层）       ↑                        │
└─────────────────────────────────────────────────────────────┘
            ↕                                      ↕
    ┌───────────────┐                    ┌──────────────────┐
    │    规则引擎    │←── 触发 ──────────→│    游戏事件       │
    │  （评估条件）   │                    │  （FactEvent）    │
    └───────────────┘                    └──────────────────┘
```

## 使用方法

1. **安装 Rust**（如果尚未安装）：

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加到 Cargo.toml**：

   ```toml
   [dependencies]
   bevy_fact_rule_event = "0.3.0"
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
       let rules_handle: Handle<FreAsset> = asset_server.load("rules/game_rules.fre.ron");
       commands.spawn(rules_handle);
   }
   ```

4. **使用分层数据库**：

   ```rust
   fn update_player_stats(
       mut layered_db: ResMut<LayeredFactDatabase>,
   ) {
       // 写入全局层（持久化）
       layered_db.set_global("player_hp", 100i64);
       layered_db.set_global("player_name", "Chara".to_string());
       
       // 写入局部层（临时，用于当前上下文）
       layered_db.set("battle_turn", 1i64);
       
       // 读取（局部层优先于全局层）
       let hp = layered_db.get_int("player_hp").unwrap_or(20);
       let name = layered_db.get_string("player_name").unwrap_or("???");
       
       // 离开上下文时清空局部层
       layered_db.clear_local();
   }
   ```

5. **创建规则文件**（`assets/rules/game_rules.fre.ron`）：

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

6. **在游戏代码中发出事件**：

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

| Crate                                   | 版本   | 描述    |
|-----------------------------------------|------|-------|
| [bevy](https://crates.io/crates/bevy)   | 0.18 | 游戏引擎  |
| [serde](https://crates.io/crates/serde) | 1.0  | 序列化框架 |

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