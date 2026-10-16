//! A reusable native application adapter around the caller-owned Spellcraft Player.
//! Only explicit typed requests mutate equipment, cartridge custody, selection, aim or admission.
pub mod config;
pub mod layout;
mod render;
pub mod revision;
pub mod session;
pub mod view;
pub mod visuals;
use atelico_component::{
    mlua::{self, Lua, LuaSerdeExt},
    register, Component,
};
use atelico_store::{Scope, Store};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::{Mutex, OnceLock},
};
register!(Component {
    name: "spell-player",
    lua: install,
    inspect,
    ..Component::NONE
});
fn registry() -> &'static Mutex<session::Registry> {
    static REGISTRY: OnceLock<Mutex<session::Registry>> = OnceLock::new();
    REGISTRY.get_or_init(Default::default)
}
// Render attachments follow Lua view lifetime; the funded Player session has a separate lifetime.
// Legacy callers share their old hook owner; scoped presentations each own a lease. Counts here are diagnostics only.
fn attachments() -> &'static Mutex<BTreeMap<(String, String), usize>> {
    static ATTACHMENTS: OnceLock<Mutex<BTreeMap<(String, String), usize>>> = OnceLock::new();
    ATTACHMENTS.get_or_init(Default::default)
}
#[derive(Default)]
struct ViewAttachments {
    legacy: BTreeSet<(String, String)>,
    owned: BTreeMap<String, VisualAttachment>,
    seen: BTreeSet<String>,
}
struct VisualAttachment {
    key: (String, String),
    presentation: visuals::Presentation,
}
impl Drop for VisualAttachment {
    fn drop(&mut self) {
        let mut counts = attachments().lock().unwrap();
        if let Some(count) = counts.get_mut(&self.key) {
            *count -= 1;
            if *count == 0 {
                counts.remove(&self.key);
            }
        }
    }
}
impl ViewAttachments {
    fn attach(&mut self, world: &str, session: &str) {
        let key = (world.to_owned(), session.to_owned());
        if self.legacy.insert(key.clone()) {
            *attachments().lock().unwrap().entry(key).or_default() += 1;
        }
    }
    fn clear(&mut self) {
        self.owned.clear();
        self.seen.clear();
        let mut counts = attachments().lock().unwrap();
        for key in std::mem::take(&mut self.legacy) {
            if let Some(count) = counts.get_mut(&key) {
                *count -= 1;
                if *count == 0 {
                    counts.remove(&key);
                    render::publish(&key.0, &key.1, vec![], "");
                }
            }
        }
    }
}
impl Drop for ViewAttachments {
    fn drop(&mut self) {
        self.clear();
    }
}
fn inspect() -> serde_json::Value {
    serde_json::json!({ "render_views": attachments().lock().unwrap().iter().map(|((world, session), views)| {
        serde_json::json!({ "world": world, "session": session, "views": views })
    }).collect::<Vec<_>>() })
}
fn install(lua: &Lua) -> mlua::Result<()> {
    lua.set_app_data(ViewAttachments::default());
    lua.set_app_data(layout::ViewLayouts::default());
    lua.set_app_data(atelico_component::input::KeyEdges::default());
    let api = lua.create_table()?;
    api.set(
        "layout",
        lua.create_function(
            |lua,
             (node, world, width, height, pane, warning): (
                String,
                String,
                f32,
                f32,
                mlua::Value,
                bool,
            )| {
                let pane: layout::PlayerPane = lua.from_value(pane)?;
                let store = lua
                    .app_data_ref::<Store>()
                    .ok_or_else(|| mlua::Error::runtime("Player layout requires the host Store"))?
                    .clone();
                let view = *lua
                    .app_data_ref::<atelico_component::AppView>()
                    .ok_or_else(|| {
                        mlua::Error::runtime("Player layout requires the host AppView")
                    })?;
                lua.app_data_mut::<layout::ViewLayouts>()
                    .expect("installed layout")
                    .publish(&store, view, &node, &world, width, height, pane, warning)
                    .map_err(mlua::Error::runtime)
            },
        )?,
    )?;
    api.set(
        "frame",
        lua.create_function(|lua, (config, name, world): (String, String, String)| {
            player_frame(lua, config, name, world, None).map(|(path, _)| path)
        })?,
    )?;
    api.set(
        "frame_visuals",
        lua.create_function(
            |lua,
             (config, name, world, node, visuals): (
                String,
                String,
                String,
                String,
                mlua::Value,
            )| {
                let visuals: visuals::Visuals = lua.from_value(visuals)?;
                player_frame(lua, config, name, world, Some((node, visuals)))
            },
        )?,
    )?;
    api.set(
        "begin_view",
        lua.create_function(|lua, ()| {
            lua.app_data_mut::<layout::ViewLayouts>()
                .expect("installed layout")
                .begin();
            lua.app_data_mut::<ViewAttachments>()
                .expect("view attachments")
                .seen
                .clear();
            Ok(())
        })?,
    )?;
    api.set(
        "end_view",
        lua.create_function(|lua, ()| {
            lua.app_data_mut::<layout::ViewLayouts>()
                .expect("installed layout")
                .end();
            let mut views = lua
                .app_data_mut::<ViewAttachments>()
                .expect("view attachments");
            let seen = views.seen.clone();
            views.owned.retain(|node, _| seen.contains(node));
            Ok(())
        })?,
    )?;
    api.set(
        "release_view",
        lua.create_function(|lua, ()| {
            if let Some(mut bindings) = lua.app_data_mut::<ViewAttachments>() {
                bindings.clear();
            }
            if let Some(mut layouts) = lua.app_data_mut::<layout::ViewLayouts>() {
                layouts.clear();
            }
            Ok(())
        })?,
    )?;
    lua.globals().set("native_spell_player", api)
}
fn player_frame(
    lua: &Lua,
    config: String,
    name: String,
    world: String,
    visual: Option<(String, visuals::Visuals)>,
) -> mlua::Result<(String, String)> {
    let store = lua
        .app_data_ref::<Store>()
        .ok_or_else(|| mlua::Error::runtime("spell-player requires the host Store"))?
        .clone();
    let _scope = lua
        .app_data_ref::<Scope>()
        .ok_or_else(|| mlua::Error::runtime("spell-player requires a host Scope"))?
        .clone();
    let input = lua
        .app_data_ref::<atelico_component::input::KeyboardInput>()
        .map(|v| v.clone())
        .unwrap_or_default();
    let pressed = lua
        .app_data_mut::<atelico_component::input::KeyEdges>()
        .expect("installed key edges")
        .presses(&input);
    let mut reg = registry()
        .lock()
        .map_err(|_| mlua::Error::runtime("player session lock poisoned"))?;
    let (key, path) = reg
        .frame_input(
            &store,
            Path::new(&config),
            &name,
            atelico_clock::now(),
            &pressed,
        )
        .map_err(mlua::Error::runtime)?;
    let session = &reg.sessions[&key];
    let bodies = session.player.world().bodies();
    let aimed = session.aimed_target();
    let mut views = lua
        .app_data_mut::<ViewAttachments>()
        .expect("installed view attachments");
    let warning = if let Some((node, visuals)) = visual {
        let view = *lua
            .app_data_ref::<atelico_component::AppView>()
            .ok_or_else(|| {
                mlua::Error::runtime("body visuals require the host AppView identity")
            })?;
        if node.trim().is_empty() {
            return Err(mlua::Error::runtime(
                "visual presentation needs its node path",
            ));
        }
        views.seen.insert(node.clone());
        if views
            .owned
            .get(&node)
            .is_some_and(|a| !a.presentation.matches(&world, &key))
        {
            views.owned.remove(&node);
        }
        if !views.owned.contains_key(&node) {
            let presentation = visuals::Presentation::new(view, &world, &key, &node)
                .map_err(mlua::Error::runtime)?;
            let attached = (world.clone(), key.clone());
            *attachments()
                .lock()
                .unwrap()
                .entry(attached.clone())
                .or_default() += 1;
            views.owned.insert(
                node.clone(),
                VisualAttachment {
                    key: attached,
                    presentation,
                },
            );
        }
        views.owned.get_mut(&node).unwrap().presentation.refresh(
            visuals,
            &session.visual_body_ids(),
            &bodies,
            aimed,
            session.player.world().tick() as f32 * session.player.world().tick_seconds(),
        )
    } else {
        views.attach(&world, &key);
        render::publish(&world, &key, bodies, aimed);
        String::new()
    };
    Ok((path, warning))
}
#[cfg(test)]
mod tests;
