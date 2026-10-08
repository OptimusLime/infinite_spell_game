-- Node bodies for particle graphs (catalog `particles`, packages/particles) on the node-graph canvas, as shader nodes
-- have theirs: `preview` draws the node's own particles at the graph's playhead, framed on where they go, over a
-- subdued dusk backdrop (a `particle_preview`, painted on the CPU: cheap enough for every node); the Explosion
-- node the whole, and a folded group (`group`) all its emitters together.
local ui = require("ui")

local renderers = {}

-- The time graph `name` is at: its timeline's playhead, or before it first plays, its poster moment.
function renderers.time(name)
	return store.get(`playback.{name}.t`) or store.get(`timeline.{name}.settings.poster`) or 0
end

-- The Particle Preview's picture: the whole effect on the particle stage (the island's west beach at dusk), at
-- time `t`, sized `style`; nil when the stage is not loaded.
local STAGE = "scenes/particle-stage.scene.luau"
function renderers.picture(name, node, t, style)
	local ok, world_view = pcall(require, "world_view")
	if not ok then return nil end
	local view = world_view.node(STAGE, "preview", 0, style, t)
	if not view then return nil end
	table.insert(view.items, { type = "ParticleEffect", graph = name, node = node or "", x = -9.7, z = -7.0, y = 0, scale = 1, start = 0, every = 0 })
	if view.camera then view.camera.time = t end
	view.id = `particles/{name}/{node or ""}`
	return view
end

-- The graph run by the core for the canvas: every node gives its output (the emitters are made in Rust, in the
-- particle style, when a preview or the scene draws them), so each shows its status and key.
function renderers.run(name, g, cat)
	return graph.run(name, g, cat, function(e)
		if e.type == "effect" then return { effect = true } end
		return { emitter = true }
	end)
end

-- A node's preview at the playhead; when nothing of it is alive then, its peak, dimmed, saying when.
local function preview(ctx, nodes)
	local t = renderers.time(ctx.graph_name)
	local pic = {
		kind = "particle_preview", id = `particles/{ctx.graph_name}/{ctx.id}`, ui = "preview", graph = ctx.graph_name, nodes = nodes,
		t = t, style = { width = "100%", height = "100%", radius = 4 * ctx.zoom, clip = true },
	}
	local peak = particles and particles.peak(ctx.graph_name, nodes, t)
	if not peak then return pic end
	pic.style = { absolute = true, left = 0, top = 0, width = "100%", height = "100%", radius = 4 * ctx.zoom, clip = true }
	return ui.node({ style = { width = "100%", height = "100%" } }, {
		pic,
		ui.node({ ui = "peak", style = { absolute = true, left = 6 * ctx.zoom, top = 4 * ctx.zoom } }, {
			ui.text(string.format("peak %.1f s", peak), { style = { font_size = math.max(7, 11 * ctx.zoom), color = "#f4ead8", text_wrap = "none" } }),
		}),
	})
end

renderers.preview = {
	height = 118,
	draw = function(ctx)
		return preview(ctx, if ctx.type and ctx.type.name == "effect" then {} else { ctx.id })
	end,
}

-- a folded group: its emitters together
renderers.group = {
	height = 118,
	draw = function(ctx)
		local members = {}
		for _, m in ctx.members do
			if ctx.graph.nodes[m] and ctx.graph.nodes[m].type == "emitter" then table.insert(members, m) end
		end
		return preview(ctx, members)
	end,
}

return renderers
