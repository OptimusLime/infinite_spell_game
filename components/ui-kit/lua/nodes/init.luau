-- Builds a running app's UI from its scene: every node under the scene's App node is data (its props a data
-- object of the node's type) read through typed proxies each frame, so a change from the Inspector, a file,
-- MCP or Claude shows on the next frame without touching the app's state. Node types come from packages: each
-- package's data modules (component.toml `data`) call `nodes.register(type, fields, build)`, which defines the
-- type's data (the store checks every write against it, the Inspector lists it) and how it is drawn. The app
-- brings behaviour: `routes` the buttons call, `values(self, env, ctx)` (named values nodes show with {name}
-- and test with `show`), and `nodes[name](self, env, widget, props, ctx)` hooks for what data cannot say (a
-- face, words lit as they are read, an animated layout). Every widget showing a prop carries a `binding` to its
-- store path, so the verify harness checks it in the tree and in the pixels.
local ui = require("ui")
local data = (_G :: any).data
local store = (_G :: any).store

-- `gathers[type]`: a node type that reads the nodes under it itself (a World gathers its 3D items): the nodes under it
-- are not built one by one (their builders draw nothing), which for a town of 800 sprites is most of a frame's reads.
local nodes = { builders = {}, types = {}, gathers = {} }
local build

local function interp(s, values)
	return (tostring(s or ""):gsub("{([%w_]+)}", function(k)
		local v = values[k]
		return if v == nil then "{" .. k .. "}" else tostring(v)
	end))
end

local function shown(show, values)
	if show == nil or show == "" then return true end
	local negate = show:sub(1, 1) == "!"
	local v = values[if negate then show:sub(2) else show]
	local on = v ~= nil and v ~= false
	return if negate then not on else on
end

local function last(path)
	return path:match("[^/]+$") or path
end

-- A color prop, else the theme's: the value and the store path it comes from.
local function color(ctx, path, p, prop, theme_key)
	local v = p[prop]
	if v then return v, `{ctx.base}.{path}.props.{prop}` end
	return ctx.colors[theme_key], `{ctx.theme}.{theme_key}`
end

local function bound(node, path, view)
	node.binding = { path = path, view = view }
	return node
end

-- A text widget showing prop `prop` of node `path` (bound when it shows the prop as written).
local function text(ctx, path, p, prop, id, style)
	local raw = p[prop] or ""
	local shown_text = interp(raw, ctx.values)
	local node = ui.text(shown_text, { id = id, style = style })
	if shown_text == raw and raw ~= "" then bound(node, `{ctx.base}.{path}.props.{prop}`, "text") end
	return node
end

nodes.interp, nodes.shown, nodes.last, nodes.color, nodes.bound, nodes.text = interp, shown, last, color, bound, text

-- A node type: `fields` its data definition (see data.define), `build(ctx, path, props, children)` its widget.
function nodes.register(name, fields, build)
	local unfit = data.define(name, fields)
	for _, e in unfit or {} do print(`nodes: {e}`) end
	nodes.types[name] = fields
	nodes.builders[name] = build
end

-- Loads every package's node types (the modules the project lists at project.data).
local loaded = false
local function load_packages()
	if loaded then return end
	loaded = true
	for _, m in store.get("project.data") or {} do
		local ok, err = pcall(require, m)
		if not ok then print(`nodes: {m}: {err}`) end
	end
end

-- Node types the app does not draw: the editor draws Labels over the scene.
local SKIP = { Label = true, Applet = true, App = true }

function build(ctx, path)
	local kind = store.get(ctx.base, path, "type")
	if SKIP[kind] then return nil end
	local p = data.object(ctx.base, path, "props")
	local b = nodes.builders[kind]
	if not p or not b then return ui.text(`{path}: no node type {kind}`, { style = { color = "#c0392b", font_size = 14 } }) end
	if not shown(p.show, ctx.values) then return nil end
	local kids = {}
	if not nodes.gathers[kind] then
		for _, child in store.get(ctx.base, path, "children") or {} do
			local w = build(ctx, child)
			if w then table.insert(kids, w) end
		end
	end
	local w = b(ctx, path, p, kids)
	local hook = ctx.hooks[last(path)]
	if hook then w = hook(ctx.self, ctx.env, w, p, ctx) end
	return w
end

-- The props proxy of the node named `name` (its last path segment) in the app's scene.
function nodes.props(ctx, name)
	local path = ctx.paths[name]
	return path and data.object(ctx.base, path, "props")
end

-- The app's UI: the App node's header and its nodes, in the App's theme.
function nodes.render(app, self, env)
	local scene = env.scene
	if not scene or not scene.id then return ui.text("This app runs in a scene: scenes/<name>.scene.luau with an App node.") end
	load_packages()
	local base = `scene.{scene.id}.nodes`
	local root = scene.root
	local a = data.object(base, root, "props")
	if not a then return ui.text(`{root}: not an App node`) end
	local colors = a.colors or {}
	local ctx = {
		base = base, theme = `{base}.{root}.props.colors`, colors = colors, font = a.font, font_size = a.font_size,
		env = env, self = self, hooks = app.nodes or {}, paths = {},
	}
	-- (the scene's node paths, listed again only when a node is added, removed or renamed)
	local paths = unit(`nodes.paths:{base}`, function()
		local t = {}
		for _, path in store.list(base) do t[last(path)] = path end
		return t
	end)
	for name, path in paths do ctx.paths[name] = path end
	-- code that draws with a theme module (the chat transcript, the grid) sees the App's palette
	if a.theme and a.theme ~= "" then
		local ok, t = pcall(require, a.theme)
		if ok and type(t) == "table" and t.colors then
			for k, v in colors do t.colors[k] = v end
		end
	end
	-- the app's named values, asked for again when one is missing: a value may come from a node built earlier in
	-- this same render (the World writes the hour its clock shows as it is built), so the first frame, an edit
	-- preview's only one, fills it too
	-- (asked again at most once a render for each missing name)
	local known = if app.values then app.values(self, env, ctx) or {} else {}
	local asked = {}
	ctx.values = setmetatable({}, { __index = function(_, k)
		if known[k] == nil and app.values and not asked[k] then
			asked[k] = true
			known = app.values(self, env, ctx) or known
		end
		return known[k]
	end })
	local kids = {}
	for _, child in store.get(base, root, "children") or {} do
		local w = build(ctx, child)
		if w then table.insert(kids, w) end
	end
	local body = ui.column({ id = "content", style = { width = "100%", grow = 1, min_height = 0, padding = a.padding, gap = a.spacing, align = a.align, scroll = true } }, kids)
	local page = { body }
	if a.header then
		page = {
			ui.row({ id = "header", style = { width = "100%", height = 64, shrink = 0, padding_x = a.padding, align = "center" } }, {
				text(ctx, root, a, "title", "title", { font = a.font, font_size = a.title_size, color = colors.text, text_wrap = "none" }),
			}),
			ui.node({ style = { width = "100%", height = 4, shrink = 0, background = colors.text } }),
			body,
		}
	end
	return bound(ui.column({ id = "app", style = { width = "100%", height = "100%", background = colors.background } }, page), `{ctx.theme}.background`, "fill")
end

return nodes
