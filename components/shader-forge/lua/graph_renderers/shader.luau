-- Node bodies for shader graphs on the node-graph canvas: `preview` draws the node's own output live (the graph
-- compiled up to that node), `ramp` its four colors, `value` its param and the time it gives. `run` compiles
-- the whole graph: the canvas shows each node's status from it.
local ui = require("ui")

local renderers = {}

-- The time shader graph `name` is at: its timeline's playhead, or before it first plays, its poster moment.
function renderers.time(name)
	return store.get(`playback.{name}.t`) or store.get(`timeline.{name}.settings.poster`) or 0
end

-- The graph's value tracks (timeline.<name>.tracks): animated params compile to uniforms.
function renderers.tracks(name)
	return store.get(`timeline.{name}.tracks`)
end

function renderers.run(name, g, cat)
	return shader.compile(name, g, cat, nil, renderers.tracks(name))
end

-- The Look Output of a look graph (a World's post pass as a graph), or nil.
function renderers.look_output(g)
	for _, n in (g and g.nodes) or {} do
		if n.type == "look_output" then return n end
	end
end

-- A live picture of graph `name` (or one of its nodes) at its playhead: a shader_preview node, drawing the
-- last good shader while an edit does not compile. A look graph shows its scene drawn through it (a look_preview:
-- the Look Output's scene and camera, rendered with the graph as its post pass; a node's, through it up to that node).
function renderers.picture(name, g, cat, node, id, style)
	local tracks = renderers.tracks(name)
	local r = shader.compile(name, g, cat, node, tracks)
	local src = if r.ok then r else r.last_good
	if not src then return nil, r end
	local t = renderers.time(name)
	local out = renderers.look_output(g)
	if out then
		-- (a node's own preview: the scene through the graph up to that stage)
		local p = out.params or {}
		-- (the compiled stage's content key: any param upstream of it changes the node, so it is drawn again)
		local stage = r.nodes and r.nodes[r.target or node or ""]
		return {
			kind = "look_preview", id = id, graph = if node then `{name}@{node}` else name, scene = p.scene or "scenes/pixel-world.scene.luau", camera = p.camera or "player", time = t,
			version = (stage and stage.key) or (if r.ok then "ok" else r.error),
			ui_state = { graph = name, node = node, time = t, ok = r.ok }, style = style,
		}, r
	end
	return {
		kind = "shader_preview", id = id, wgsl = src.wgsl, time = t, params = shader.sample(tracks, src.slots or {}, t),
		ui_state = { graph = name, node = node, time = t, ok = r.ok }, style = style,
	}, r
end

renderers.preview = {
	height = 92,
	draw = function(ctx)
		local pic, r = renderers.picture(ctx.graph_name, ctx.graph, ctx.catalog, ctx.id, `{ctx.graph_name}/{ctx.id}`,
			{ width = "100%", height = "100%", radius = 4 * ctx.zoom, clip = true, background = "#00000000" })
		if not pic then
			return ui.text(r.error, { style = { font_size = math.max(7, 10 * ctx.zoom), color = ctx.look.colors.danger, text_wrap = "word" } })
		end
		pic.ui = "preview"
		return pic
	end,
}

-- a folded group (a stage): what its last member makes, large
renderers.group = {
	height = 150,
	draw = function(ctx)
		local last = ctx.members[#ctx.members]
		if not (last and ctx.graph.nodes[last]) then return ui.node({}) end
		local pic, r = renderers.picture(ctx.graph_name, ctx.graph, ctx.catalog, last, `{ctx.graph_name}/{ctx.id}`,
			{ width = "100%", height = "100%", radius = 4 * ctx.zoom, clip = true, background = "#00000000" })
		if not pic then
			return ui.text(r.error, { style = { font_size = math.max(7, 10 * ctx.zoom), color = ctx.look.colors.danger, text_wrap = "word" } })
		end
		pic.ui = "preview"
		return pic
	end,
}

local function stops(p)
	local out = {}
	for _, k in { "c0", "c1", "c2", "c3" } do table.insert(out, (p[k] or "#000000") .. "ff") end
	return out
end

renderers.ramp = {
	height = 26,
	draw = function(ctx)
		local p = ctx.params
		local defaults = {}
		for _, d in ctx.type.params or {} do defaults[d.name] = d.default end
		local full = setmetatable({}, { __index = function(_, k) return p[k] or defaults[k] end })
		return ui.node({ ui = "ramp", style = { width = "100%", height = "100%", radius = 4 * ctx.zoom, gradient = stops(full), gradient_dir = "x", border = 1, border_color = ctx.look.colors.border } })
	end,
}

renderers.value = {
	height = 22,
	draw = function(ctx)
		local t = renderers.time(ctx.graph_name) * (ctx.params.speed or 1)
		local fs = 14 * ctx.zoom
		return ui.row({ style = { width = "100%", height = "100%", align = "center", gap = 6 * ctx.zoom } }, {
			ui.text(string.format("t = %.2f", t), { style = { font_size = fs, color = ctx.look.colors.text, text_wrap = "none", shrink = 0 } }),
			ui.node({ style = { grow = 1, height = 4 * ctx.zoom, radius = 2 * ctx.zoom, background = ctx.look.colors.border } }, {
				ui.node({ style = { width = `{math.floor((t % 1) * 100)}%`, height = "100%", radius = 2 * ctx.zoom, background = ctx.look.colors.selected } }),
			}),
		})
	end,
}

return renderers
