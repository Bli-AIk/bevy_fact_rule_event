# bevy_fact_rule_event

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_fact_rule_event.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_fact_rule_event.svg"/> <br> <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

**bevy_fact_rule_event** — Bevy 引擎的通用数据驱动事实-规则-事件（FRE）系统。

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 致谢

整个 FRE 系统在极大程度上受到了 **aarthificial** 在数据驱动游戏逻辑方面卓越工作的启发。我们对这些奠基性的想法表示由衷的感激与敬畏：

- [📺 Dynamic Conversations - Legacy Devlog #23](https://www.youtube.com/watch?v=1LlF5p5Od6A)
- [📺 Implicit Choices - Legacy Devlog #24](https://www.youtube.com/watch?v=yAIH7GGZ9L0)

## 简介

将诸如 *“如果玩家生命值小于10并且持有魔剑，则触发隐藏对话”* 这种复杂的逻辑硬编码到程序里，往往会导致难以维护的 `if/else`
迷宫，且每次微调都需要重新编译。

`bevy_fact_rule_event` 提供了一种数据驱动的替代方案。它是一个声明式的规则引擎，将游戏逻辑从你的 Rust 代码中分离出来，允许你在外部的
RON 文件中定义行为。

然而，请务必记住，这并非适用于所有项目的“万灵丹”。这是一种架构选择，专为那些相比于直接硬编码，更看重灵活性和策划主导迭代的项目而准备。

## 优点与缺点

与任何架构模式一样，FRE 系统也是一种权衡。

### 优点 ✅

- **解耦**：游戏逻辑与引擎和系统代码完全分离。
- **迭代速度**：无需等待重新编译，通过编辑 RON 文件即可修改游戏行为。
- **策划友好**：让非程序员也能随时微调任务、对话和平衡性。
- **集中状态**：事实（Fact）数据库为整个游戏状态提供了单一的可信数据源。

### 缺点 ❌

- **间接性**：相比传统的代码调试，追踪某个行为的具体触发原因可能会更加困难。
- **性能开销**：评估规则会带来一定的性能损耗；虽然对于对话系统来说微不足道，但不适合高频的物理逻辑。
- **复杂性**：对于极其简单的游戏，管理外部规则文件引入的开销可能反而超过它带来的便利。

## 核心理念

> "事件不包含逻辑，数据不包含行为，逻辑只存在于规则中。"

FRE 系统强制实施关注点的清晰分离：

- **事实（Facts, F）**：集中式键值数据库，用于存储游戏状态
- **规则（Rules, R）**：声明式逻辑，基于条件转换状态
- **事件（Events, E）**：触发规则评估的信号广播

## 功能特性

* 🗂️ **数据驱动规则**：在 RON 文件中定义游戏逻辑，无需修改代码
* 🥞 **分层事实数据库**：支持全局层和局部层的层级状态管理
    - 全局层：在场景切换后仍然保留的持久化事实（如玩家属性）
    - 局部层：作用于当前上下文的临时事实（如战斗状态）
* 🗄️ **集中式状态管理**：所有游戏事实存储在可查询的数据库中
* 🔀 **条件逻辑**：支持嵌套逻辑运算符的复杂条件评估
* 📥 **自动资源加载**：与 Bevy 资源系统无缝集成
* 📡 **事件广播**：游戏系统之间的解耦通信
* 🛡️ **类型安全值**：支持 Int、Float、Bool 和 String 事实类型
* 🔄 **双向同步**：事实可以与 ECS 组件同步，实现响应式 UI 更新
* 👁️ **（计划中）可视化规则编辑器**
* 🔥 **（计划中）热重载支持**

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