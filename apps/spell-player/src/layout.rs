//! Per-view presentation geometry. No camera policy, simulation, inventory or funded state.
use atelico_component::AppView;
use atelico_data::{Data, DataKind};
use atelico_store::{Origin, Store};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, DataKind, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlayerPane {
    #[default]
    None,
    Equipment,
    Inventory,
    Sources,
    Capacity,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Data, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerLayout {
    pub world_id: String,
    pub battlefield: String,
    pub dock: String,
    pub toolbar: String,
    pub actions: String,
    pub status: String,
    pub drawer: String,
    pub compact: bool,
    #[data(choice)]
    pub pane: PlayerPane,
}
#[derive(Debug, Clone)]
pub struct Geometry {
    pub battlefield: PlayerRect,
    pub dock: PlayerRect,
    pub toolbar: PlayerRect,
    pub actions: PlayerRect,
    pub status: PlayerRect,
    pub drawer: PlayerRect,
    pub compact: bool,
}
impl Geometry {
    pub fn at(width: f32, height: f32, pane: PlayerPane, warning: bool) -> Result<Self, String> {
        if !width.is_finite() || !height.is_finite() || width <= 0. || height <= 0. {
            return Err("Player layout requires a finite positive viewport".into());
        }
        let compact = width < 1440.;
        let toolbar_h: f32 = 36.;
        let actions_h: f32 = if compact { 80. } else { 38. };
        let status_h: f32 = if warning { 48. } else { 24. };
        let dock_h = (16. + toolbar_h + actions_h + status_h + 8.).min(height);
        let field_h = height - dock_h;
        let drawer_w = match pane {
            PlayerPane::None => 0.,
            PlayerPane::Equipment => 440_f32.min(width * 0.46),
            PlayerPane::Inventory => 660_f32.min(width * 0.52),
            PlayerPane::Sources | PlayerPane::Capacity => 620_f32.min(width * 0.52),
        };
        let r = |x, y, width, height| PlayerRect {
            x,
            y,
            width,
            height,
        };
        let toolbar = r(
            8.,
            field_h + 8.,
            (width - 16.).max(0.),
            toolbar_h.min((dock_h - 8.).max(0.)),
        );
        let actions_y = toolbar.y + toolbar.height + 4.;
        let actions = r(
            8.,
            actions_y.min(height),
            (width - 16.).max(0.),
            actions_h.min((height - actions_y).max(0.)),
        );
        let status_y = actions.y + actions.height + 4.;
        let status = r(
            8.,
            status_y.min(height),
            (width - 16.).max(0.),
            status_h.min((height - status_y - 8.).max(0.)),
        );
        Ok(Self {
            battlefield: r(drawer_w, 0., width - drawer_w, field_h),
            dock: r(0., field_h, width, dock_h),
            toolbar,
            actions,
            status,
            drawer: r(0., 0., drawer_w, field_h),
            compact,
        })
    }
}

#[derive(Default)]
pub(crate) struct ViewLayouts {
    store: Option<Store>,
    roots: BTreeSet<String>,
    seen: BTreeSet<String>,
}
impl ViewLayouts {
    pub fn begin(&mut self) {
        self.seen.clear();
    }
    pub fn end(&mut self) {
        if let Some(store) = &self.store {
            self.roots.retain(|root| {
                if self.seen.contains(root) {
                    return true;
                }
                let _ = store.remove(root, Origin::Host);
                false
            });
        }
    }
    pub fn publish(
        &mut self,
        store: &Store,
        view: AppView,
        node: &str,
        world: &str,
        width: f32,
        height: f32,
        pane: PlayerPane,
        warning: bool,
    ) -> Result<String, String> {
        if node.is_empty() || world.is_empty() {
            return Err("Player layout requires exact node and World IDs".into());
        }
        let g = Geometry::at(width, height, pane, warning)?;
        let encoded: String = node.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
        let root = format!("runtime.player_layouts.v{}.n{encoded}", view.id);
        let path = format!("{root}.layout");
        if store.type_of(&path).as_deref() != Some("PlayerLayout") {
            let mut errors = store.define(PlayerLayout::schema());
            errors.extend(store.define(PlayerRect::schema()));
            errors.extend(store.bind(&path, "PlayerLayout"));
            errors.extend(store.bind(&format!("{root}.rects.*"), "PlayerRect"));
            if !errors.is_empty() {
                return Err(errors.join("; "));
            }
        }
        let rect = |name| format!("{root}.rects.{name}");
        let row = PlayerLayout {
            world_id: world.into(),
            battlefield: rect("battlefield"),
            dock: rect("dock"),
            toolbar: rect("toolbar"),
            actions: rect("actions"),
            status: rect("status"),
            drawer: rect("drawer"),
            compact: g.compact,
            pane,
        };
        let mut ops = Vec::new();
        let value = serde_json::to_value(row).map_err(|e| e.to_string())?;
        ops.extend(atelico_store::diff(
            &path,
            store.get(&path).as_ref(),
            &value,
        ));
        for (name, r) in [
            ("battlefield", g.battlefield),
            ("dock", g.dock),
            ("toolbar", g.toolbar),
            ("actions", g.actions),
            ("status", g.status),
            ("drawer", g.drawer),
        ] {
            let p = rect(name);
            let value = serde_json::to_value(r).map_err(|e| e.to_string())?;
            ops.extend(atelico_store::diff(&p, store.get(&p).as_ref(), &value));
        }
        store.batch(ops, Origin::Host)?;
        self.store = Some(store.clone());
        self.seen.insert(root.clone());
        self.roots.insert(root);
        Ok(path)
    }
    pub fn clear(&mut self) {
        self.seen.clear();
        if let Some(s) = self.store.take() {
            for root in std::mem::take(&mut self.roots) {
                let _ = s.remove(&root, Origin::Host);
            }
        }
    }
}
impl Drop for ViewLayouts {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn disjoint(a: PlayerRect, b: PlayerRect) -> bool {
        a.x + a.width <= b.x
            || b.x + b.width <= a.x
            || a.y + a.height <= b.y
            || b.y + b.height <= a.y
    }
    #[test]
    fn battlefield_and_drawers_never_overlap_controls_in_both_profiles() {
        for (w, h) in [(1600., 960.), (1100., 800.), (800., 600.)] {
            for pane in [
                PlayerPane::None,
                PlayerPane::Equipment,
                PlayerPane::Inventory,
                PlayerPane::Sources,
                PlayerPane::Capacity,
            ] {
                for warning in [false, true] {
                    let g = Geometry::at(w, h, pane, warning).unwrap();
                    assert!(disjoint(g.battlefield, g.dock) && disjoint(g.battlefield, g.drawer));
                    for r in [
                        g.battlefield,
                        g.dock,
                        g.drawer,
                        g.toolbar,
                        g.actions,
                        g.status,
                    ] {
                        assert!(
                            r.x >= 0.
                                && r.y >= 0.
                                && r.width >= 0.
                                && r.height >= 0.
                                && r.x + r.width <= w
                                && r.y + r.height <= h,
                            "{r:?}"
                        );
                    }
                    assert!(g.battlefield.height > h / 2.);
                }
            }
        }
        assert!(Geometry::at(f32::NAN, 800., PlayerPane::None, false).is_err());
    }
    #[test]
    fn presentation_matches_exact_world_and_refuses_ambiguity_without_restyling_others() {
        use atelico_component::mlua::Lua;
        let lua = Lua::new();
        lua.load(r#"
            data={object=function(ref)
                if ref=='layout' then return {world_id='root/field',battlefield='rect'} end
                if ref=='rect' then return {x=400,y=0,width=700,height=636} end
                error('unexpected binding')
            end}
            native_spell_player={}
            nodes={render=function() return fixture end}
            function require(name)
                if name=='nodes' then return nodes end
                if name=='spell_player.hud' then return {layout_error=function(message)return {diagnostic=message}end} end
                return {}
            end
        "#).exec().unwrap();
        let app: atelico_component::mlua::Table = lua
            .load(include_str!("../lua/apps/spell_player.luau"))
            .eval()
            .unwrap();
        lua.globals().set("app", app).unwrap();
        lua.load(r#"
            local field={kind='scene_3d',id='root/field',style={grow=1,min_height=240}}
            local other={kind='scene_3d',id='other/field',style={grow=1,min_height=240}}
            fixture={children={field,other,{player_layout='layout'}}}
            local got=app.render({}, {})
            assert(got==fixture and field.id=='root/field' and field.style.left==400 and field.style.width==700)
            assert(other.style.absolute==nil and other.style.grow==1)
            field.style={grow=1}
            table.insert(fixture.children,{player_layout='layout'})
            assert(app.render({},{})==fixture and fixture.children[#fixture.children].diagnostic and field.style.absolute==nil)
            fixture={children={field}} -- no new native layout capability
            assert(app.render({},{})==fixture and field.style.absolute==nil)
        "#).exec().unwrap();
    }

    #[test]
    fn two_views_resize_and_release_independently_without_game_state() {
        let store = Store::new();
        store
            .set(
                "runtime.funded.receipts",
                serde_json::json!([{"amount":20}]),
                Origin::Host,
            )
            .unwrap();
        let before = store.get("runtime.funded");
        let (mut a, mut b) = (ViewLayouts::default(), ViewLayouts::default());
        let (av, bv) = (AppView::new(1), AppView::new(1));
        let ap = a
            .publish(
                &store,
                av,
                "root/player",
                "root/world",
                1600.,
                960.,
                PlayerPane::None,
                false,
            )
            .unwrap();
        let bp = b
            .publish(
                &store,
                bv,
                "root/player",
                "root/world",
                1100.,
                800.,
                PlayerPane::Inventory,
                false,
            )
            .unwrap();
        assert_ne!(ap, bp);
        let b_before = store.get(bp.rsplit_once('.').unwrap().0);
        a.publish(
            &store,
            av,
            "root/player",
            "root/world",
            1100.,
            800.,
            PlayerPane::Sources,
            true,
        )
        .unwrap();
        assert_eq!(store.get(bp.rsplit_once('.').unwrap().0), b_before);
        a.begin();
        a.end(); // deleting the last node retires only that view's presentation
        a.clear();
        assert!(store.read::<PlayerLayout>(&ap).is_none());
        assert!(store.read::<PlayerLayout>(&bp).is_some());
        assert_eq!(store.get("runtime.funded"), before);
    }
}
