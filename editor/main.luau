-- The Atelico editor: a Luau app drawn with the engine's own components (layout, ui-kit). It holds no
-- state: everything it shows is read from the store (store.get, tracked per render unit) and everything
-- it changes is a store.set. The host composites the scene and the agent into the `surface` nodes here.
--
-- This file is the shell: a window with the Store and Claude floating over it. Everything else is a part — a
-- module `editor_parts.<name>` ({name, slot, order, size, width, docs, edits, render, routes}) from an installed
-- `editor-<name>` package — placed by slot (toolbar, left, center, bottom, right, status) or by
-- `editor.ui.layout`, which a recipe sets. Installing a part package (live) adds its panel; with none installed,
-- an engine checkout shows its own parts (editor/editor_parts) and a fresh project shows an empty editor.
--
-- The editor follows its active document (editor.ui.doc, a project file; its kind from project.docs): a part
-- shows only for the kinds it declares (`docs`; none: any), the center is the part that `edits` that kind, the
-- other open documents are tabs. Sizes and what is hidden come from runtime.layout (the kind's workspace with the
-- document's own preference over it) over the parts' declared `width`s. A part with `inspects = <node type>` is
-- not a panel: the Inspector draws it for a selected node of that type.
local ui = require("ui")
local kit = require("kit")
local shell = require("shell")
local store_panel = require("store")
local welcome = require("welcome")
local chat = require("chat")
local proposal = require("proposal")

local editor = { name = "Editor", routes = {} }
local SPLIT = 5
local MIN_ZOOM, MIN_CENTER = 0.42, 360 -- under these the center sheds side panels (verify's floor is 0.4)
local BUILTIN = { "toolbar", "project", "scene_tree", "node_graph", "scene_view", "timeline_editor", "theme_editor", "slides", "console", "shader_preview", "inspector", "status" }

for name, f in store_panel.routes do editor.routes[name] = f end
for name, f in welcome.routes do editor.routes[name] = f end
for name, f in chat.routes do editor.routes[name] = f end
for name, f in proposal.routes do editor.routes[name] = f end

-- The installed parts: from editor-* packages, and this checkout's own.
local function parts()
	local list = {}
	local packages = store.get("project.packages") or {}
	for _, p in packages.components or {} do
		local part = string.match(p.name, "^editor%-(.+)$")
		if part and part ~= "panels" then
			local ok, def = pcall(require, "editor_parts." .. string.gsub(part, "-", "_"))
			if ok and type(def) == "table" then table.insert(list, def) end
		end
	end
	-- the checkout's own parts (editor/editor_parts), where it has them and no package provides one of that name
	for _, name in BUILTIN do
		local ok, def = pcall(require, "editor_parts." .. name)
		if ok and type(def) == "table" then
			local dup = false
			for _, d in list do if d.name == def.name then dup = true end end
			if not dup then table.insert(list, def) end
		end
	end
	kit.inspectors = {}
	for _, def in list do
		for name, f in def.routes or {} do editor.routes[name] = f end
		if def.inspects then kit.inspectors[def.inspects] = def end
	end
	return list
end

-- The window's mode (editor.ui.mode: game, video, slides) as editor/modes.toml describes it.
local function mode()
	return store.get("editor.modes." .. (store.get("editor.ui.mode") or "game")) or {}
end

-- Parts by slot: the mode's slots and `editor.ui.layout` (a recipe's or a form's; slot → names, in order) first,
-- the mode's where both name a slot (not in a form), else each part's own slot and order.
local function slots(list)
	local layout = table.clone(store.get("editor.ui.layout") or {})
	-- (a window in a form is arranged by it alone; else the mode's slots win)
	if (store.get("editor.ui.form") or "") == "" then
		for slot, names in mode().slots or {} do layout[slot] = names end
	end
	local out, placed = { toolbar = {}, left = {}, center = {}, bottom = {}, right = {}, status = {} }, {}
	for slot, names in layout do
		for _, n in names do
			for _, def in list do
				if def.name == n and out[slot] then table.insert(out[slot], def); placed[def] = true end
			end
		end
	end
	-- parts the layout does not mention go to their own slot (a newly installed part always shows up), unless the
	-- window is in a form: it shows only the parts it names
	local rest = {}
	local form = store.get("editor.ui.form") or ""
	for _, def in list do if not placed[def] and form == "" then table.insert(rest, def) end end
	table.sort(rest, function(a, b) return (a.order or 0) < (b.order or 0) end)
	-- (a part without a slot of its own, such as the On Device strip, shows only where a layout puts it)
	for _, def in rest do
		local slot = def.slot and out[def.slot]
		if slot then table.insert(slot, def) end
	end
	-- a center with nothing for this document takes the part that edits it from the side (in video mode the node
	-- graph sits left of a shader's preview, but a video graph has no preview: the graph is the center)
	if #out.center == 0 then
		for _, side in { "left", "right" } do
			for i = #out[side], 1, -1 do
				if out[side][i].edits then table.insert(out.center, table.remove(out[side], i)) end
			end
		end
	end
	return out
end

-- The parts shown for the active document: meant for its kind, not hidden by its layout, and in Focus only its
-- editor and the Inspector.
local function shown(list, kind, layout)
	local hide = {}
	for _, n in layout.hide or {} do hide[n] = true end
	for _, n in mode().hide or {} do hide[n] = true end
	local focused = store.get("editor.ui.focused") == true or layout.focused == true
	local out = {}
	for _, def in list do
		local chrome = def.slot == "toolbar" or def.slot == "status"
		local keep = not def.inspects and (chrome or (kit.allows(def.docs, kind) and not hide[def.name]))
		if keep and focused and not chrome then keep = def.edits == true or def.name == "inspector" end
		if keep then table.insert(out, def) end
	end
	return out
end

-- A column's width: the layout's, else the widest its parts declare.
local function width(defs, set, fallback)
	if set then return set end
	local w
	for _, def in defs do if def.width then w = math.max(w or 0, def.width) end end
	return w or fallback
end

-- The open scene's design size, when the active document is a scene.
local function design(kind)
	if kind ~= "scene" then return nil end
	local id = store.get("editor.ui.scene")
	for _, s in store.get("project.scenes") or {} do
		if s.id == id then return s.width, s.height end
	end
end

-- Parts stacked in a column: sized ones keep their size, the last grows. Each panel is stamped with what it is
-- for (`docs`, `edits`, `collapsed`), which verify checks against the active document.
local function stack(defs, env, collapsed, lead)
	local out = {}
	for i, def in defs do
		if i > 1 then table.insert(out, kit.rule()) end
		-- a sized part keeps its size while there is room; every part gives way, to its header and a little
		-- more (60), when the window is short
		-- (a part's size may be a function: the On Device strip is one row or two, by the width it measured; the
		-- form may give its first part a share of the height, `lead`)
		local size = if i == 1 and lead then `{math.floor(lead * 100)}%` elseif type(def.size) == "function" then def.size() else def.size
		local s = if size and i < #defs then { width = "100%", height = size, shrink = 1, min_height = kit.u(60) } else { width = "100%", grow = 1, min_height = if collapsed then 0 else 60 }
		-- a part that animates (`clock`) gets the time, and so draws again every frame
		local node = unit(def.key or def.name, def.render, s, env.focus, env.inputs, if def.clock then env.time else nil)
		node.docs, node.edits, node.collapsed = def.docs or { "*" }, def.edits == true, collapsed == true
		table.insert(out, node)
	end
	return out
end

-- The open documents as tabs over the center; the active one highlighted.
local function tabs(doc, room)
	local c, m = kit.c, kit.m
	local items = {}
	local all = store.get("editor.ui.docs") or {}
	-- as many tabs as fit `room` with their names whole (a name's width estimated from its letters; the active one's
	-- with its ×), the active one always among them, then the others nearest it, one side and the other in turn
	local size = m.small + 1
	local function wide(d)
		return kit.u(16) + #kit.title(d) * size * 0.62 + (if d == doc and #all > 1 then kit.u(34) else 0)
	end
	local at = table.find(all, doc) or 1
	local lo, hi = at, at
	local used = if all[at] then wide(all[at]) else 0
	while true do
		local grew = false
		for _, k in { hi + 1, lo - 1 } do
			if all[k] and used + wide(all[k]) <= (room or math.huge) then
				used += wide(all[k])
				if k > hi then hi = k else lo = k end
				grew = true
			end
		end
		if not grew then break end
	end
	local docs = if #all > 0 then table.move(all, lo, hi, 1, {}) else all
	for _, d in docs do
		local active = d == doc
		local name = kit.title(d)
		-- only the active tab has its × (as tall as a line of its type and a little air, whatever the look's type)
		local closes = active and #all > 1
		local tab = ui.row({ id = "doc:" .. d, selected = active, style = {
			height = kit.bars().tabs, shrink = 1, min_width = kit.u(if closes then 84 else 44), max_width = kit.u(220), padding_left = kit.u(12), padding_right = kit.u(4), gap = kit.u(6), align = "center",
			background = if active then c.panel else nil, border_top = if active then 2 else 0, border_color = c.accent,
		} }, {
			ui.button({ id = "open:" .. d, on_press = kit.route("open", { doc = d }), style = { shrink = 1, min_width = 0, height = "100%", align = "center" } }, {
				kit.text(name, { bold = active, tone = if active then "text" else "muted", size = m.small + 1 }),
			}),
			if closes then kit.icon_button("×", kit.route("close_doc", { doc = d }), { id = "close:" .. d, height = math.max(kit.u(20), kit.line(m.small) + kit.u(4)), size = m.small, tip = "Close this tab", tip_below = true }) else nil,
		})
		if active then kit.bound(tab, kit.color_path("panel"), "fill", "bottom") end
		table.insert(items, tab)
	end
	return ui.row({ id = "tabs", style = { width = "100%", height = kit.bars().tabs, shrink = 0, background = c.header, align = "end", clip = true } }, items)
end

-- Writes a layout field where it applies: the document's own preference when it sets that field, else its kind's
-- workspace (layout.<doc>: the doc's dots spelled ":", as atelico_control::layout_path has it).
local function layout_set(field, value)
	local doc, kind = kit.active()
	local own = "layout." .. string.gsub(doc, "%.", ":")
	local at = if (store.get(own) or {})[field] ~= nil then own else `editor.layouts.{kind}`
	-- (a level not there yet is written whole, so it is created as a Layout)
	if store.get(at) == nil then store.set(at, { [field] = value }) else store.set(`{at}.{field}`, value) end
end

function editor.routes.set(self, r, ctx)
	local ok, err = store.set(r.path, r.value)
	if not ok then ctx.emit_error(err) end
end


function editor.routes.open(self, r, ctx) store.set("editor.ui.doc", r.doc) end

-- A tab closed: the next one (else the previous) becomes active; the last open document stays.
function editor.routes.close_doc(self, r, ctx)
	local docs, out, at = store.get("editor.ui.docs") or {}, {}, 1
	for i, d in docs do if d == r.doc then at = i else table.insert(out, d) end end
	if #out == 0 then return end
	if store.get("editor.ui.doc") == r.doc then store.set("editor.ui.doc", out[math.min(at, #out)]) end
	store.set("editor.ui.docs", out)
end

function editor.routes.focus_toggle(self, r, ctx) store.set("editor.ui.focused", not store.get("editor.ui.focused")) end

-- The title's form menu: opened by a click on the form's name; a pick turns the window into that form (or opens
-- the Store) and closes it.
function editor.routes.form_menu(self, r, ctx) store.set("editor.ui.form_menu", store.get("editor.ui.form_menu") ~= true) end

function editor.routes.form_pick(self, r, ctx)
	store.set("editor.ui.form_menu", false)
	if r.form then store.set("editor.ui.form", r.form) end
	if r.store then store_panel.routes.store_toggle(self, r, ctx) end
	-- the Inspector, where the form leaves it out: added to the right column, or taken away again
	if r.inspector then
		local layout = store.get("editor.ui.layout") or {}
		local right, had = {}, false
		for _, p in layout.right or {} do
			if p == "inspector" then had = true else table.insert(right, p) end
		end
		if not had then table.insert(right, 1, "inspector") end
		store.set("editor.ui.layout.right", right)
	end
end

function editor.routes.console_toggle(self, r, ctx) layout_set("console", not ((store.get("runtime.layout") or {}).console == true)) end

-- Tidy with Claude: its terminal opens with a request to simplify this document's layout (the host adds the
-- document, its panels and their sizes, and the ui_layout tool to apply it with).
function editor.routes.tidy(self, r, ctx)
	ctx.emit("editor.ask", { text = "Tidy the Atelico editor's layout for the document I have open." })
end

function editor.routes.expand(self, r, ctx)
	local out, found = {}, false
	for _, d in store.get("editor.ui.expanded") or {} do
		if d == r.dir then found = true else table.insert(out, d) end
	end
	if not found then table.insert(out, r.dir) end
	store.set("editor.ui.expanded", out)
end

function editor.routes.node(self, r, ctx)
	store.set("editor.ui.node", r.path)
	store.set("editor.ui.tab", "node")
end

function editor.routes.resolution(self, r, ctx)
	store.set(`scene.{r.scene}.width`, r.width)
	store.set(`scene.{r.scene}.height`, r.height)
end

-- A property field submitted: typed text, parsed as the property's type.
function editor.routes.prop(self, r, ctx)
	local text = tostring(r[r.key] or "")
	ctx.emit("input.set", { key = r.key, value = "" })
	ctx.emit("input.blur", {})
	local value = text
	if r.kind == "number" then
		value = tonumber(text)
	elseif r.kind == "boolean" then
		value = text == "true"
	elseif r.kind == "vector" then
		value = {}
		for part in string.gmatch(text, "[^,%s]+") do
			local n = tonumber(part)
			if n == nil then return end
			table.insert(value, n)
		end
	end
	if value == nil then return end
	local ok, err = store.set(r.path, value)
	if not ok then ctx.emit_error(err) end
end

-- Panel splitters: drags arrive with px/py in window coordinates; the size goes into the layout.
function editor.routes.split(self, r, ctx)
	if r.which == "left" then
		layout_set("left", r.px)
	elseif r.which == "right" then
		layout_set("right", r.width - r.px)
	elseif r.which == "bottom" then
		layout_set("bottom", r.height - kit.bars().status - r.py)
	end
end

-- A bare mode (slides): the active document's editor alone, the whole window.
local function bare(env, s)
	local center = if #s.center > 0 then stack(s.center, env) else { unit("empty", shell.empty) }
	return ui.column({ style = { width = "100%", height = "100%", background = kit.c.viewport } }, center)
end

-- Over the window, in the bottom right corner: the request card (talking to Claude), and above it, opened from it, a
-- proposal waiting for an answer. Under the title, when its form name was clicked: the form menu.
-- Over the window: Claude in the bottom right corner (its button always; the card or the terminal when summoned,
-- a proposal in the card; note toasts beside the button while it works), and under the title, when its form name
-- was clicked, the form menu.
local function with_cards(window, env, welcomed)
	-- the empty engine's modal is the talk with Claude itself: nothing goes over it
	if welcomed then return window end
	local over = chat.corner(env)
	if store.get("editor.ui.form_menu") == true then
		-- required here, not at the top: an empty engine has no panels package installed until Claude installs it
		local panels = require("panels")
		table.insert(over, panels.form_menu(panels.form_options(), store.get("editor.ui.form") or ""))
	end
	-- (`cards` is painted again over the scene the host composites, so it stays on top)
	return ui.node({ style = { width = "100%", height = "100%" } }, {
		window,
		ui.node({ id = "cards", style = { absolute = true, left = 0, top = 0, width = "100%", height = "100%" } }, over),
	})
end

function editor.render(self, env)
	-- (for a button to show its press: the last one pressed and the time now)
	kit.pressed, kit.now = env.pressed, env.time or 0
	local c = kit.c
	local ui_state = store.get("editor.ui")
	-- (a tall window stacks what a wide one puts side by side: the Store under the document, a form's right column
	-- over it)
	local tall = env.height > env.width
	local playing = store.get("editor.run.playing")
	local doc, kind = kit.active()
	local layout = store.get("runtime.layout") or {}
	local s = slots(shown(parts(), kind, layout))
	if mode().bare then return bare(env, s) end
	-- the empty engine: the welcome view alone, full window
	if ui_state.welcome then
		return with_cards(ui.column({ style = { width = "100%", height = "100%", background = kit.c.background } }, { unit("welcome", welcome.render, env.focus, env.inputs, env.time) }), env, true)
	end
	-- sizes: the document's layout, else the window's form's, else the parts' own
	local form = if (ui_state.form or "") ~= "" then store.get("form." .. ui_state.form) or {} else {}
	-- (the recipe's own sizes, editor.ui.left/right, after the document's layout and the form's)
	local left = width(s.left, layout.left or form.left or ui_state.left, 240)
	local right = width(s.right, layout.right or form.right or ui_state.right, 320)
	-- a form may put the right column above the center in a tall window (full width, a share of the height)
	local above = if tall and (form.right_above or 0) > 0 and #s.right > 0 then math.floor((env.height - kit.bars().toolbar - kit.bars().status - 4) * form.right_above) else 0
	local above_defs = if above > 0 then s.right else nil
	if above > 0 then s.right = {} end
	local console = layout.console == true
	local bars = kit.bars()
	-- the console, collapsed, is its header and the rule over it
	local collapsed = bars.head + 2
	local bottom = if console then (layout.bottom or form.bottom or kit.u(150)) else collapsed
	-- never a tiny center: the right column's panels move under the left column's, then the console, the left and the
	-- right columns go, until the center is readable (Claude floats over the window and takes none of it)
	local sw, sh = design(kind)
	local function room()
		local w = env.width - (if #s.left > 0 then left + SPLIT else 0) - (if #s.right > 0 then right + SPLIT else 0)
		local h = env.height - bars.toolbar - 2 - bars.status - 2 - bars.tabs - collapsed - (if #s.bottom > 0 then bottom + SPLIT else 0) - (if store_panel.open() and tall then env.height * 0.36 else 0) - (if above > 0 then above + SPLIT else 0)
		return w, if sw then math.min((w - 56) / sw, (h - 56) / sh) else 1
	end
	for _, step in { "stack", "bottom", "left", "right" } do
		local w, zoom = room()
		if w >= MIN_CENTER and zoom >= MIN_ZOOM then break end
		if step == "stack" and #s.left > 0 and #s.right > 0 then
			for _, def in s.right do table.insert(s.left, def) end
			s.right = {}
		elseif step == "bottom" and console then
			console, bottom = false, collapsed
		elseif step == "left" or step == "right" then
			s[step] = {}
		end
	end
	local body = {}
	if #s.left > 0 then
		table.insert(body, ui.column({ style = { width = left, height = "100%", shrink = 0 } }, stack(s.left, env)))
		table.insert(body, kit.splitter("left", true))
	end
	-- nothing edits what is open (a new project: nothing is): the welcome view asks what you are building
	local center = if #s.center > 0 then stack(s.center, env, nil, form.lead) else { unit("welcome", welcome.render, env.focus, env.inputs, env.time) }
	-- (one open document needs no tab: its panel names it)
	if doc ~= "" and kind and #(store.get("editor.ui.docs") or {}) > 1 then table.insert(center, 1, unit("tabs", tabs, doc, (room()))) end
	if #s.bottom > 0 then
		if console then table.insert(center, kit.splitter("bottom", false, { height = env.height })) else table.insert(center, kit.rule()) end
		table.insert(center, ui.column({ style = { width = "100%", height = bottom, shrink = 0, clip = true } }, stack(s.bottom, env, not console)))
	end
	table.insert(body, ui.column({ style = { grow = 1, width = 0, min_width = 0, height = "100%" } }, center))
	-- the Store beside the document, or under it in a tall window
	local store_under = store_panel.open() and tall
	if store_panel.open() and not tall then
		table.insert(body, kit.separator(true, SPLIT))
		table.insert(body, unit("store", store_panel.panel, {}, env.focus, env.inputs))
	end
	if #s.right > 0 then
		table.insert(body, kit.splitter("right", true, { width = env.width }))
		table.insert(body, ui.column({ style = { width = right, height = "100%", shrink = 0 } }, stack(s.right, env)))
	end
	-- (the toolbar gets the window's width: its own measure is a frame old after a resize)
	local toolbar = if #s.toolbar > 0 then unit("toolbar", s.toolbar[1].render, env.width) else unit("toolbar", shell.toolbar)
	local status = if #s.status > 0 then unit("status", s.status[1].render) else unit("status", shell.status)
	local column = {
		toolbar,
		ui.node({ style = { width = "100%", height = 2, shrink = 0, background = if playing then c.play else c.border } }),
	}
	if above_defs then
		table.insert(column, ui.column({ style = { width = "100%", height = above, shrink = 0 } }, stack(above_defs, env)))
		table.insert(column, kit.separator(false, SPLIT))
	end
	table.insert(column, ui.row({ style = { width = "100%", grow = 1, min_height = 0 } }, body))
	-- in a tall window the Store sits under the document
	if store_under then
		table.insert(column, kit.separator(false, SPLIT))
		table.insert(column, unit("store", store_panel.panel, { width = "100%", height = math.floor(env.height * 0.36), shrink = 0 }, env.focus, env.inputs))
	end
	table.insert(column, kit.rule())
	table.insert(column, status)
	local window = ui.column({ style = { width = "100%", height = "100%", background = c.background } }, column)
	return with_cards(window, env, #s.center == 0)
end

return editor
