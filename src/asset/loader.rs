//! # loader.rs
//!
//! # loader.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Owns the asset-loading and host-action dispatch layer for FRE files. It defines the
//! Bevy asset loader for `.fre.ron` assets and the registry that host applications use to bind
//! serialized action names to runtime command handlers.
//!
//! 负责 FRE 文件的资源加载层和宿主动作分发层。它定义了 `.fre.ron` 资源的 Bevy 加载器，
//! 以及宿主应用用来把序列化动作名绑定到运行时命令处理器的注册表。

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;
use std::collections::HashMap;

use super::action_defs::{ActionDef, CoreActionDef};
use super::rule_defs::FreAsset;

pub struct FreAssetLoader<A: ActionDef = CoreActionDef>(std::marker::PhantomData<A>);

impl<A: ActionDef> Default for FreAssetLoader<A> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<A: ActionDef> bevy::reflect::TypePath for FreAssetLoader<A> {
    fn type_path() -> &'static str {
        "bevy_fact_rule_event::asset::FreAssetLoader"
    }

    fn short_type_path() -> &'static str {
        "FreAssetLoader"
    }
}

impl<A: ActionDef> AssetLoader for FreAssetLoader<A> {
    type Asset = FreAsset<A>;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset = ron::de::from_bytes::<FreAsset<A>>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fre.ron"]
    }
}

pub type ActionHandler<A> =
    Box<dyn Fn(&A, &crate::LayeredFactDatabase, &mut Commands) + Send + Sync>;

pub struct ActionHandlerRegistry<A: ActionDef = CoreActionDef> {
    handlers: HashMap<String, ActionHandler<A>>,
}

impl<A: ActionDef> Default for ActionHandlerRegistry<A> {
    fn default() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
}

unsafe impl<A: ActionDef> Send for ActionHandlerRegistry<A> {}
unsafe impl<A: ActionDef> Sync for ActionHandlerRegistry<A> {}

impl<A: ActionDef> Resource for ActionHandlerRegistry<A> {}

impl<A: ActionDef> ActionHandlerRegistry<A> {
    pub fn register<F>(&mut self, action_type: &str, handler: F)
    where
        F: Fn(&A, &crate::LayeredFactDatabase, &mut Commands) + Send + Sync + 'static,
    {
        self.handlers
            .insert(action_type.to_string(), Box::new(handler));
    }

    pub fn has_handler(&self, action_type: &str) -> bool {
        self.handlers.contains_key(action_type)
    }

    pub fn execute(&self, action: &A, db: &crate::LayeredFactDatabase, commands: &mut Commands) {
        let action_type = action.action_type();

        if let Some(handler) = self.handlers.get(action_type) {
            handler(action, db, commands);
        } else {
            warn!(
                "FRE: No handler registered for action type '{}'",
                action_type
            );
        }
    }
}
