//! A reusable native application adapter around the caller-owned Spellcraft Player.
//! Only explicit typed requests mutate equipment, cartridge custody, selection, aim or admission.
pub mod config;
mod render;
pub mod revision;
pub mod session;
pub mod view;
use atelico_component::{
    mlua::{self, Lua},
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
// Two views of the same session share one hook owner, so closing one does not remove the other's bodies.
fn attachments() -> &'static Mutex<BTreeMap<(String, String), usize>> {
    static ATTACHMENTS: OnceLock<Mutex<BTreeMap<(String, String), usize>>> = OnceLock::new();
    ATTACHMENTS.get_or_init(Default::default)
}
#[derive(Default)]
struct ViewAttachments(BTreeSet<(String, String)>);
impl ViewAttachments {
    fn attach(&mut self, world: &str, session: &str) {
        let key = (world.to_owned(), session.to_owned());
        if self.0.insert(key.clone()) {
            *attachments().lock().unwrap().entry(key).or_default() += 1;
        }
    }
    fn clear(&mut self) {
        let mut counts = attachments().lock().unwrap();
        for key in std::mem::take(&mut self.0) {
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
    lua.set_app_data(atelico_component::input::KeyEdges::default());
    let api = lua.create_table()?;
    api.set(
        "frame",
        lua.create_function(|lua, (config, name, world): (String, String, String)| {
            // The app binds its existing Store after component installation, so resolve context at invocation.
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
            lua.app_data_mut::<ViewAttachments>()
                .expect("installed view attachments")
                .attach(&world, &key);
            render::publish(
                &world,
                &key,
                reg.sessions[&key].player.world().bodies(),
                reg.sessions[&key].aimed_target(),
            );
            Ok(path)
        })?,
    )?;
    api.set(
        "release_view",
        lua.create_function(|lua, ()| {
            if let Some(mut bindings) = lua.app_data_mut::<ViewAttachments>() {
                bindings.clear();
            }
            Ok(())
        })?,
    )?;
    lua.globals().set("native_spell_player", api)
}
#[cfg(test)]
mod tests;
