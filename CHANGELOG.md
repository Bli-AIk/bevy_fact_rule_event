# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/Bli-AIk/bevy_fact_rule_event/compare/v0.4.0...v0.4.1) - 2026-04-24

### Added

- *(rule)* add action method to rule builder

### Documentation

- *(bevy_fact_rule_event)* update and refine documentation

### Fixed

- *(core)* update crate dependencies and clean up docs

### Refactor

- standardize terminology from "资产" to "资源"
- *(core)* add debug feature and derive Reflect

## [0.4.0](https://github.com/Bli-AIk/bevy_fact_rule_event/compare/v0.3.0...v0.4.0) - 2026-03-25

### Added

- *(ci)* add tokei lint checks to crate workflows
- *(bevy_fact_rule_event)* make plugin schedule configurable
- *(database)* add set_if_changed method to FactDatabase and LayeredFactDatabase
- *(fact-modifications)* extend fact modification operations with arithmetic and expression support
- *(bevy_fact_rule_event)* add extensible condition evaluator for rules
- *(fre)* add rule output event queuing and improve dialogue handling
- *(database)* add list type support for FactValue and FactReader

### Fixed

- *(expr)* correct expression logic
- *(bevy_fact_rule_event)* correct sound asset path
- *(asset)* convert action names to lowercase in event IDs

### Miscellaneous Tasks

- *(deps)* update ron dependency from 0.10 to 0.12
- *(lint)* improve #[expect] reason detection in tokei scripts
- add clippy configuration
- *(crates)* add readme and repository fields to Cargo.toml files

### Refactor

- split asset and rule registries and enforce lint thresholds ([#15](https://github.com/Bli-AIk/bevy_fact_rule_event/pull/15))
- [**breaking**] Generic `ActionDef` trait and `EnumRegistry` ([#14](https://github.com/Bli-AIk/bevy_fact_rule_event/pull/14))
- *(deps)* update bevy dependencies to disable default features
- extract UI functions to reduce nesting depth
- *(asset)* add has_handler method and reorder execute
- *(expr, rule)* fix unary minus parsing and modulo operator
- *(example_mod)* update dialogue formatting and layout
- *(rule)* remove RuleCondition and RuleAction, use expression-based conditions

## [0.3.0](https://github.com/Bli-AIk/bevy_fact_rule_event/compare/v0.2.0...v0.3.0) - 2026-02-11

### Added

- [**breaking**] upgrade to bevy 0.18

## [0.2.0](https://github.com/Bli-AIk/bevy_fact_rule_event/compare/v0.1.0...v0.2.0) - 2026-02-06

### Added

- *(rule)* add rule scoping and layered registry with consume_event flag
- *(fact)* add IntList support for HP values and stats arrays
- *(core)* add StringList variant to FactValue for inventories and tags
- *(asset)* add unique ID generation for rules with index suffix
- *(bevy_fact_rule_event)* enhance rule system with ActionEvent support and expression conditions
- *(database)* add Clone trait to database resource
- Introduce LayeredFactDatabase for Scoped State Management ([#7](https://github.com/Bli-AIk/bevy_fact_rule_event/pull/7))

### Fixed

- *(rule)* upgrade view-scoped rule registration warning to error

### Refactor

- *(core)* update docstrings for condition fields
- *(bevy_fact_rule_event)* update asset type and test naming
- *(asset)* rename RuleSetAsset to FreAsset for unified data format
