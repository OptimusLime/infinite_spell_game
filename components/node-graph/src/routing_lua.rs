//! One transport boundary for the typed presentation router; no JSON/domain model.
use crate::routing::{self, Edge, EdgeId, Node, NodeId, Point, Rect, Request};
use atelico_component::mlua::{self, Lua, Table};

fn point(lua: &Lua, p: Point) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("x", p.x)?;
    t.set("y", p.y)?;
    Ok(t)
}
fn rect(lua: &Lua, r: Rect) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("x", r.x)?;
    t.set("y", r.y)?;
    t.set("w", r.w)?;
    t.set("h", r.h)?;
    Ok(t)
}
fn points(lua: &Lua, ps: &[Point]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (i, p) in ps.iter().enumerate() {
        t.raw_set(i + 1, point(lua, *p)?)?;
    }
    Ok(t)
}
pub(crate) fn install(lua: &Lua, graph: &Table) -> mlua::Result<()> {
    let cache = std::cell::RefCell::new(routing::Cache::default());
    graph.set(
        "route",
        lua.create_function(move |lua, input: Table| {
            let mut request = Request {
                nodes: Vec::new(),
                edges: Vec::new(),
            };
            for n in input.get::<Table>("nodes")?.sequence_values::<Table>() {
                let n = n?;
                let r = n.get::<Table>("rect")?;
                request.nodes.push(Node {
                    id: NodeId(n.get("id")?),
                    rect: Rect {
                        x: r.get("x")?,
                        y: r.get("y")?,
                        w: r.get("w")?,
                        h: r.get("h")?,
                    },
                });
            }
            for e in input.get::<Table>("edges")?.sequence_values::<Table>() {
                let e = e?;
                request.edges.push(Edge {
                    id: EdgeId(e.get("id")?),
                    from: NodeId(e.get("from")?),
                    to: NodeId(e.get("to")?),
                });
            }
            let mut cache = cache.borrow_mut();
            let result = cache.layout(request).map_err(mlua::Error::external)?;
            let out = lua.create_table()?;
            out.set("bounds", rect(lua, result.bounds)?)?;
            let routes = lua.create_table()?;
            for (i, r) in result.routes.iter().enumerate() {
                let t = lua.create_table()?;
                t.set("id", r.id.0.as_str())?;
                t.set("points", points(lua, &r.points)?)?;
                t.set("badge", rect(lua, r.badge)?)?;
                t.set("leader", points(lua, &r.leader)?)?;
                routes.raw_set(i + 1, t)?;
            }
            out.set("routes", routes)?;
            Ok(out)
        })?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_geometry_crosses_one_lua_boundary_with_opaque_identity() {
        let lua = Lua::new();
        let graph = lua.create_table().unwrap();
        install(&lua, &graph).unwrap();
        lua.globals().set("graph", graph).unwrap();
        lua.load(r#"
            local input={nodes={{id='runtime.work.groups.a',rect={x=0,y=0,w=100,h=80}},{id='runtime.work.groups.b',rect={x=300,y=0,w=100,h=80}}},edges={{id='stable:prerequisite>dependent',from='runtime.work.groups.a',to='runtime.work.groups.b'}}}
            local result=graph.route(input)
            assert(#result.routes==1 and result.routes[1].id==input.edges[1].id)
            assert(#result.routes[1].points>=2 and result.routes[1].badge.w==48)
            input.edges[1].to='missing';assert(not pcall(graph.route,input))
        "#).exec().unwrap();
    }
}
