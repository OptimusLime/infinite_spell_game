-- The editor's panels, each a render unit (see main.luau): it reads the store paths it shows and renders
-- again only when one of them changes. Toolbar, Project (file tree), Scene Tree, Scene view, Inspector
-- (Scene, Node, Theme), Console, Agent, status bar.
local ui = require("ui")
local kit = require("kit")
local picker = require("picker")

local panels = {}
local EDITOR_COLORS = { "background", "panel", "header", "raised", "border", "text", "muted", "accent", "on_accent", "play", "danger", "viewport" }
local RESOLUTIONS = { { "Phone", 390, 844 }, { "Tablet", 820, 1180 }, { "Desktop", 1280, 800 } }

-- The selected scene's entry in project.scenes ({id, name, width, height, file, applet}).
function panels.scene()
	local id = store.get("editor.ui.scene")
	for _, s in store.get("project.scenes") or {} do
		if s.id == id then return s end
	end
	return nil
end

local function top_scenes()
	local out = {}
	for _, s in store.get("project.scenes") or {} do
		if not s.id:find("/") then table.insert(out, s) end
	end
	return out
end

function panels.toolbar()
	local c = kit.c
	local playing = store.get("editor.run.playing")
	local options = {}
	for _, s in top_scenes() do table.insert(options, { s.name, s.id }) end
	local logo = kit.bound(ui.node({ id = "logo", style = { width = 18, height = 18, radius = 5, background = c.accent, align = "center", justify = "center" } }, {
		ui.node({ style = { width = 6, height = 6, radius = 3, background = c.on_accent } }),
	}), kit.color_path("accent"), "fill")
	local left = ui.row({ style = { grow = 1, width = 0, gap = 14, align = "center" } }, {
		ui.row({ style = { gap = 8, align = "center", shrink = 0 } }, { logo, kit.text("Atelico", { bold = true, size = kit.m.font_size + 1 }) }),
		kit.rule(true),
		if #options > 0 then kit.segments(options, store.get("editor.ui.scene"), function(v) return kit.set("editor.ui.scene", v) end, { id = "scene-tab", style = { width = 96 * #options + 6, height = 30 } })
			else kit.text("No scenes: add scenes/<name>.scene.luau", { tone = "muted" }),
	})
	local play = kit.button(if playing then "■  Stop" else "▶  Play", kit.set("editor.run.playing", not playing), {
		id = "play", tone = if playing then "play" else "raised", bold = true, height = 30, disabled = #options == 0, style = { width = 96 },
	})
	local right = ui.row({ style = { grow = 1, width = 0, gap = 10, align = "center", justify = "end" } }, {
		kit.button(`Claude  {store.get("runtime.shortcut")}J`, kit.route("drawer"), { id = "agent-toggle", active = store.get("editor.ui.drawer"), height = 30 }),
	})
	return kit.bound(ui.row({ id = "panel:toolbar", style = { width = "100%", height = 44, shrink = 0, padding_x = 12, gap = 12, align = "center", background = c.header } }, {
		left, play, right,
	}), kit.color_path("header"), "fill")
end

local function row(id, depth, children, action, active)
	local c = kit.c
	return ui.button({
		id = id,
		on_press = action,
		hover = { background = if active then nil else c.raised },
		style = { width = "100%", height = 24, shrink = 0, padding_left = 10 + depth * 14, padding_right = 10, gap = 7, align = "center", background = if active then c.raised else nil },
	}, children)
end

-- A file's mark: a scene is a small frame in the accent color, anything else a muted dot.
local function mark(scene)
	local c = kit.c
	return ui.node({ style = { width = 12, height = 12, shrink = 0, align = "center", justify = "center" } }, {
		if scene then ui.node({ style = { width = 8, height = 10, radius = 2, border = 1.5, border_color = c.accent } })
			else ui.node({ style = { width = 4, height = 4, radius = 2, background = c.border } }),
	})
end

-- Project: the project directory as a tree, from the directory service (project.files), live.
function panels.project(style)
	local c = kit.c
	local open = {}
	for _, d in store.get("editor.ui.expanded") or {} do open[d] = true end
	local current = store.get("editor.ui.scene")
	local tree = { dirs = {}, files = {} }
	for _, f in store.get("project.files") or {} do
		local node = tree
		local parts = f:split("/")
		for i = 1, #parts - 1 do
			node.dirs[parts[i]] = node.dirs[parts[i]] or { dirs = {}, files = {} }
			node = node.dirs[parts[i]]
		end
		table.insert(node.files, f)
	end
	local rows = {}
	local function walk(node, prefix, depth)
		local names = {}
		for name in node.dirs do table.insert(names, name) end
		table.sort(names)
		for _, name in names do
			local path = if prefix == "" then name else prefix .. "/" .. name
			local expanded = open[path] == true
			table.insert(rows, row("dir:" .. path, depth, {
				kit.text(if expanded then "−" else "+", { tone = "muted", size = kit.m.small + 1, style = { width = 12 } }),
				kit.text(name, { bold = depth == 0, style = { grow = 1, width = 0 } }),
			}, kit.route("expand", { dir = path }), false))
			if expanded then walk(node.dirs[name], path, depth + 1) end
		end
		table.sort(node.files)
		for _, f in node.files do
			local name = f:match("[^/]+$")
			local scene = f:match("^scenes/(.+)%.scene%.luau$")
			local active = scene ~= nil and scene == current
			table.insert(rows, row("file:" .. f, depth, {
				mark(scene ~= nil),
				kit.text(name, { bold = active, tone = if scene then "text" else "muted", style = { grow = 1, width = 0 } }),
			}, if scene then kit.set("editor.ui.scene", scene) else nil, active))
		end
	end
	walk(tree, "", 0)
	return kit.panel("project", "Project", { kit.spacer(), kit.text(`{#(store.get("project.files") or {})} files`, { tone = "muted", size = kit.m.small }) },
		ui.column({ style = { width = "100%", grow = 1, min_height = 0, padding_y = 4, scroll = true } }, rows), style)
end

-- Scene Tree: the open scene's node hierarchy; instanced nodes are marked with the file they come from.
function panels.tree(style)
	local c = kit.c
	local s = panels.scene()
	local rows = {}
	if s then
		local nodes = store.get("scene", s.id, "nodes") or {}
		local selected = store.get("editor.ui.node")
		local function walk(path, depth)
			local n = nodes[path]
			if not n then return end
			local active = path == selected
			table.insert(rows, row("node:" .. path, depth, {
				ui.node({ style = { width = 2, height = 14, radius = 1, shrink = 0, background = if active then c.accent else nil } }),
				kit.text(n.name, { bold = active, style = { grow = 1, width = 0 } }),
				if n.instance then kit.text((n.instance:match("[^/]+$"):gsub("%.scene%.luau$", "")), { tone = "accent", size = kit.m.small }) else ui.node({}),
				kit.text(n.type, { tone = "muted", size = kit.m.small }),
			}, kit.route("node", { path = path }), active))
			for _, child in n.children or {} do walk(child, depth + 1) end
		end
		walk(store.get("scene", s.id, "root"), 0)
	end
	return kit.panel("tree", "Scene Tree", { kit.spacer(), kit.text(if s then s.name else "", { tone = "muted", size = kit.m.small }) },
		ui.column({ style = { width = "100%", grow = 1, min_height = 0, padding_y = 4, scroll = true } }, rows), style)
end

function panels.scene_view(style)
	local c = kit.c
	local s = panels.scene()
	local info = {}
	local playing = store.get("editor.run.playing")
	if s then
		local zoom = store.get("runtime.scene.zoom")
		table.insert(info, kit.text(s.name, { bold = true }))
		table.insert(info, kit.text(`{s.width} × {s.height}{if zoom then `  ·  {math.floor(zoom * 100 + 0.5)}%` else ""}`, { tone = "muted", size = kit.m.small + 1 }))
		if not playing and store.get("scene", s.id, "preview") == "off" then
			table.insert(info, kit.text("preview off: press Play", { tone = "muted", size = kit.m.small + 1 }))
		end
	else
		table.insert(info, kit.text("No scene: add scenes/<name>.scene.luau", { tone = "muted", size = kit.m.small + 1 }))
	end
	table.insert(info, kit.spacer())
	if playing then
		table.insert(info, kit.pill(`PLAYING  {store.get("runtime.scene.fps") or 0} fps`, c.play))
	else
		table.insert(info, kit.pill("EDIT"))
	end
	local p = kit.panel("scene", "Scene", info, { kind = "surface", surface = "scene", style = { width = "100%", grow = 1, min_height = 0 } }, style)
	p.binding = nil -- its body is the scene view, not panel color
	return p
end

-- A text field bound to a store value: shows the value unless it has focus (then what is being typed).
local function field(id, path, value, focus, inputs, kind)
	local c, m = kit.c, kit.m
	local bind = "prop:" .. path
	local editing = focus == bind
	local node = ui.input({
		id = id, bind = bind, value = if editing then (inputs[bind] or "") else tostring(value), placeholder = tostring(value),
		on_submit = kit.route("prop", { path = path, kind = kind, key = bind }), focus_border = c.accent,
		style = { width = "100%", height = 26, padding = 5, border = 1, radius = m.radius, border_color = c.border, background = c.background, color = c.text, font_size = m.small + 1, text_wrap = "none" },
	})
	if not editing then kit.bound(node, path, "input") end
	return node
end

local function is_color(v)
	return type(v) == "string" and v:match("^#%x%x%x%x%x%x$") ~= nil
end

-- Rows of colors under `base` (a store path to a {key = "#rrggbb"} table); a click picks one for the picker.
local function color_rows(base, keys, picked)
	local c = kit.c
	local rows = {}
	for _, key in keys do
		local path = base .. "." .. key
		local active = path == picked
		table.insert(rows, ui.button({
			id = "swatch-" .. key,
			on_press = kit.set("editor.ui.color", path),
			hover = { background = if active then nil else c.raised },
			style = { width = "100%", height = 26, shrink = 0, padding_x = 6, gap = 8, align = "center", radius = kit.m.radius, background = if active then c.raised else nil, border = if active then 1 else 0, border_color = c.accent },
		}, {
			kit.swatch(path, 16, "sw-" .. key),
			kit.text(key, { style = { grow = 1, width = 0 } }),
			kit.text(store.get(path) or "", { id = "hex-" .. key, bind = path, mono = true, tone = "muted", size = kit.m.small + 1 }),
		}))
	end
	return ui.column({ style = { width = "100%", gap = 2, shrink = 0 } }, rows)
end

local function scene_tab(s)
	if not s then return kit.text("No scene. Add scenes/<name>.scene.luau.", { tone = "muted", wrap = true }) end
	local presets, current = {}, nil
	for _, r in RESOLUTIONS do
		table.insert(presets, { r[1], `{r[2]}x{r[3]}` })
		if r[2] == s.width and r[3] == s.height then current = `{r[2]}x{r[3]}` end
	end
	local base = "scene." .. s.id
	return ui.column({ style = { width = "100%", gap = 18 } }, {
		ui.column({ style = { width = "100%", gap = 4 } }, {
			kit.text(s.name, { id = "scene-name", bind = base .. ".name", bold = true, size = kit.m.font_size + 3 }),
			kit.text(s.file, { tone = "muted", size = kit.m.small + 1 }),
		}),
		kit.section("Resolution", {
			kit.segments(presets, current, function(v)
				local w, h = v:match("(%d+)x(%d+)")
				return kit.route("resolution", { scene = s.id, width = tonumber(w), height = tonumber(h) })
			end, { id = "res" }),
			kit.prop("Design", `{s.width} × {s.height}`),
		}),
		kit.section("Edit preview", {
			kit.segments({ { "First frame", "frame" }, { "Off", "off" } }, store.get(base .. ".preview"), function(v) return kit.set(base .. ".preview", v) end, { id = "preview" }),
			kit.text("Edit mode shows the scene still: mounted, one frame, no input and no engine calls.", { tone = "muted", wrap = true, size = kit.m.small + 1 }),
		}),
	})
end

-- A kind as data.define spells it: "color", {kind = "one_of", options = ...}, ...
local function kind_name(k)
	return if type(k) == "table" then k.kind else k
end

-- The selected node: every field of its type (from the type's data definition, see project.types), with the
-- editor its kind calls for; fields the node does not set show their default and can be set from here.
local function node_tab(s, focus, inputs)
	local path = store.get("editor.ui.node")
	local n = s and path and store.get("scene", s.id, "nodes", path)
	if not n then return kit.text("Select a node in the Scene Tree.", { tone = "muted", wrap = true }) end
	local base = `scene.{s.id}.nodes.{path}.props`
	local picked = store.get("editor.ui.color")
	local def = store.get("project.types", n.type)
	local props = n.props or {}
	local fields = {}
	if def then
		for _, fdef in def.fields do table.insert(fields, fdef) end
	else
		-- an untyped node: its props as they are
		local keys = {}
		for k in props do table.insert(keys, k) end
		table.sort(keys)
		for _, k in keys do table.insert(fields, { name = k, kind = if is_color(props[k]) then "color" elseif type(props[k]) == "table" then "any" else type(props[k]) }) end
	end
	local items = {
		ui.column({ style = { width = "100%", gap = 4 } }, {
			kit.text(n.name, { bold = true, size = kit.m.font_size + 3 }),
			kit.text(`{n.type}{if def and def.package ~= "" then "  ·  " .. def.package else ""}  ·  {n.file}`, { tone = "muted", size = kit.m.small + 1 }),
			if def and def.doc ~= "" then kit.text(def.doc, { tone = "muted", size = kit.m.small + 1 }) else ui.node({ style = { hidden = true } }),
		}),
	}
	local rows = {}
	for _, fd in fields do
		local k, kind = fd.name, kind_name(fd.kind)
		local v = props[k]
		local at = base .. "." .. k
		if kind == "map" and type(v) == "table" then
			local ck = {}
			for key, x in v do if is_color(x) then table.insert(ck, key) end end
			table.sort(ck)
			if #ck > 0 then table.insert(items, kit.section(k, { color_rows(at, ck, picked) })) end
		elseif kind == "color" then
			local swatch = if v ~= nil then kit.swatch(at, 20, "sw-" .. k)
				else ui.node({ style = { width = 20, height = 20, shrink = 0, radius = 3, border = 1, border_color = kit.c.muted } })
			-- picking an unset color starts it at its default (or the App's accent), then edits it
			local start = fd.default or store.get("scene", s.id, "nodes", store.get("scene", s.id, "root"), "props", "colors", "accent") or "#808080"
			table.insert(rows, kit.prop(k, ui.row({ style = { width = "100%", gap = 6, align = "center" } }, {
				ui.button({ id = "pick-" .. k, on_press = kit.route("pick", { path = at, start = start }), style = { shrink = 0 } }, { swatch }),
				if v ~= nil then field("prop-" .. k, at, v, focus, inputs, "string") else kit.text("unset", { tone = "muted", size = kit.m.small + 1 }),
			})))
		elseif kind == "one_of" and type(fd.kind) == "table" then
			local options = {}
			for _, o in fd.kind.options do table.insert(options, { o, o }) end
			table.insert(rows, kit.prop(k, kit.segments(options, v or fd.default, function(o) return kit.set(at, o) end, { id = "seg-" .. k })))
		elseif kind == "bool" then
			table.insert(rows, kit.prop(k, kit.segments({ { "Yes", true }, { "No", false } }, if v == nil then fd.default else v, function(o) return kit.set(at, o) end, { id = "seg-" .. k })))
		elseif kind == "list" or kind == "any" or kind == "map" then
			if v ~= nil then table.insert(rows, kit.prop(k, kit.text(if type(v) == "table" then "{…}" else tostring(v), { tone = "muted" }))) end
		else
			local shown = if v == nil then (fd.default or "") else v
			local input = field("prop-" .. k, at, shown, focus, inputs, if kind == "number" then "number" else "string")
			if v == nil then input.binding = nil end -- it shows the default, not a store value
			table.insert(rows, kit.prop(k, input))
		end
	end
	if #rows > 0 then table.insert(items, 2, kit.section("Properties", rows)) end
	-- the picker for the color picked, above the fields so it stays in view
	if type(picked) == "string" and picked:sub(1, #base) == base and is_color(store.get(picked)) then
		table.insert(items, 2, kit.section(picked:sub(#base + 2), { picker.view(picked, focus, inputs.hex) }))
	end
	return ui.column({ style = { width = "100%", gap = 16 } }, items)
end

local function theme_tab(s, focus, inputs)
	local target = store.get("editor.ui.theme")
	local base, keys, file
	if target == "scene" and s and s.applet then
		base = `scene.{s.id}.nodes.{s.applet}.props.colors`
		keys = store.list(base)
		table.sort(keys)
		file = s.file
	else
		base, keys, file = "theme.editor.colors", EDITOR_COLORS, "editor/theme.toml"
	end
	local picked = store.get("editor.ui.color")
	if type(picked) ~= "string" or picked:sub(1, #base + 1) ~= base .. "." or store.get(picked) == nil then
		picked = base .. "." .. (if table.find(keys, "accent") then "accent" else keys[1] or "")
	end
	return ui.column({ style = { width = "100%", gap = 12 } }, {
		kit.segments({ { "Editor", "editor" }, { "Scene app", "scene" } }, if target == "scene" and s and s.applet then "scene" else "editor", function(v) return kit.set("editor.ui.theme", v) end, { id = "theme-target" }),
		kit.text(file, { tone = "muted", size = kit.m.small + 1, wrap = true }),
		color_rows(base, keys, picked),
		kit.section(picked:match("[^%.]+$"), { picker.view(picked, focus, inputs.hex) }),
	})
end

function panels.inspector(style, focus, inputs)
	local tab = store.get("editor.ui.tab")
	local s = panels.scene()
	local tabs = kit.segments({ { "Scene", "scene" }, { "Node", "node" }, { "Theme", "theme" } }, tab, function(v) return kit.set("editor.ui.tab", v) end, { id = "tab", style = { width = 190, height = 24 } })
	local body = if tab == "theme" then theme_tab(s, focus, inputs) elseif tab == "node" then node_tab(s, focus, inputs) else scene_tab(s)
	return kit.panel("inspector", "Inspector", { kit.spacer(), tabs }, ui.column({ style = { width = "100%", grow = 1, min_height = 0, padding = 14, scroll = true } }, { body }), style)
end

local LEVEL = { error = "danger", trace = "accent", request = "play" }

function panels.console(style)
	local lines = {}
	local all = store.get("runtime.console") or {}
	local requests = 0
	for _, l in all do
		if l.source == "engine" then requests += 1 end
		local t = l.at - math.floor(l.at / 3600) * 3600
		table.insert(lines, ui.row({ style = { width = "100%", gap = 10, shrink = 0, align = "start", padding_x = 12, min_height = 18 } }, {
			kit.text(string.format("%02d:%02d", math.floor(t / 60) % 60, math.floor(t % 60)), { mono = true, size = kit.m.small, tone = "muted", style = { width = 34, padding_top = 1 } }),
			kit.text(l.source, { mono = true, size = kit.m.small, tone = LEVEL[l.level] or "muted", style = { width = 50, padding_top = 1 } }),
			kit.text(l.text, { mono = true, size = kit.m.small + 1, color = if l.level == "error" then kit.c.danger else nil, style = { grow = 1, width = 0 } }),
		}))
	end
	return kit.panel("console", "Console", {
		kit.text(`{#all} lines`, { tone = "muted", size = kit.m.small }),
		kit.spacer(),
		kit.pill(if requests == 1 then "1 engine request" else `{requests} engine requests`, if requests > 0 then kit.c.play else nil),
	}, ui.column({ id = "console-lines", style = { width = "100%", grow = 1, min_height = 0, padding_y = 6, gap = 1, scroll = true, scroll_y = #lines * 100 } }, lines), style)
end

function panels.agent(style)
	local p = kit.panel("agent", "Claude", {
		kit.text(store.get("runtime.agent.status") or "", { tone = "muted", size = kit.m.small }),
		kit.spacer(),
		kit.button(`{store.get("runtime.shortcut")}J`, kit.route("drawer"), { id = "agent-close", height = 22, size = kit.m.small }),
	}, { kind = "surface", surface = "agent", style = { width = "100%", grow = 1, min_height = 0 } }, style)
	p.binding = nil -- its body is the terminal
	return p
end

function panels.status()
	local c = kit.c
	local playing = store.get("editor.run.playing")
	local s = panels.scene()
	local on = if playing then c.on_accent else nil
	local bar = ui.row({ id = "panel:status", style = { width = "100%", height = 26, shrink = 0, padding_x = 12, gap = 14, align = "center", background = if playing then c.play else c.header } }, {
		kit.text(if playing then "Playing" else "Edit mode", { bold = true, size = kit.m.small + 1, color = on or c.muted }),
		kit.text(if s then s.name else "", { size = kit.m.small + 1, color = on or c.muted }),
		kit.spacer(),
		kit.text(store.get("runtime.caption") or "", { id = "caption", bold = true, size = kit.m.small + 1, color = on or c.accent }),
		kit.spacer(),
		kit.text(`MCP :{store.get("runtime.mcp")}  ·  editor/theme.toml`, { size = kit.m.small + 1, color = on or c.muted }),
	})
	return kit.bound(bar, kit.color_path(if playing then "play" else "header"), "fill")
end

return panels
