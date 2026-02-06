# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
