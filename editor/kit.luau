-- The editor's widgets. Every color and metric is read from the store (theme.editor, loaded from
-- editor/theme.toml) at the moment it is used, so the render unit using it depends on exactly that path.
-- Widgets that show a store value carry `binding = {path, view}`, which `app verify` checks.
local ui = require("ui")

local kit = {}
local BOLD = "fonts/Inter-SemiBold.ttf"
local MONO = "fonts/FiraMono-Medium.ttf"
local THEME = "theme.editor"

-- The window's look (editor.ui.look): the editor's own colors, or one of the theme's other looks.
local function look()
	local name = store.get("editor.ui.look")
	return if name and name ~= "" then store.get(`{THEME}.looks.{name}`) else nil
end

function kit.color_path(key)
	local name = store.get("editor.ui.look")
	return if name and name ~= "" then `{THEME}.looks.{name}.colors.{key}` else THEME .. ".colors." .. key
end
kit.c = setmetatable({}, { __index = function(_, k) return store.get(kit.color_path(k)) end })
-- the faces the look sets text in (nil: the layout's own); `display` sets headings (panel titles, the name)
kit.face = setmetatable({}, { __index = function(_, k)
	local l = look()
	local bold = if l and l.bold and l.bold ~= "" then l.bold else BOLD
	if k == "bold" then return bold end
	if k == "display" then return if l and l.display and l.display ~= "" then l.display else bold end
	return if l and l.text ~= "" then l.text else nil
end })
-- metrics: the theme's, with a look's own corner radius and text sizes over them
kit.m = setmetatable({}, { __index = function(_, k)
	local l = look()
	if l and (k == "radius" or k == "font_size" or k == "small") and l[k] then return l[k] end
	return store.get(THEME .. ".metrics." .. k)
end })
-- a look's shape beyond its colors: outline width of buttons and cards, a hard shadow's offset (0: none)
kit.shape = setmetatable({}, { __index = function(_, k)
	local l = look()
	return if l and l[k] then l[k] else 0
end })

-- A length in kit units: `n` px at the type scale's body size (18 px), scaled with it, so a look or UI scale that
-- sets larger text makes the room around it larger too. Whole pixels; 1–2 px hairlines stay raw.
function kit.u(n)
	return math.floor(n * (kit.m.font_size or 18) / 18 + 0.5)
end

-- The height a line of text takes at `size` (the layout's: 1.25 of it, whole pixels).
function kit.line(size)
	return math.ceil(size * 1.25)
end

-- The editor's bars, as tall as their text needs: the toolbar (a button's height and a margin), the status bar, the
-- document tabs, and a panel's header (so a collapsed panel is its header).
function kit.bars()
	local m = kit.m
	local button = kit.line(m.font_size) + kit.u(8)
	return { toolbar = button + kit.u(12), status = kit.line(m.small + 1) + kit.u(8), tabs = kit.line(m.font_size) + kit.u(10), head = kit.line(m.small) + kit.u(12) }
end

function kit.route(name, args)
	return ui.action("route", { name = name, args = args })
end

-- The action that writes `value` at `path` (the only way the editor changes anything).
function kit.set(path, value)
	return kit.route("set", { path = path, value = value })
end

function kit.bound(node, path, view, at)
	node.binding = { path = path, view = view, at = at }
	return node
end

-- Text: `tone` is a color key (default text), `size` defaults to the theme's, `bold`/`mono` pick the face.
-- `bind = path` marks it as showing that store value.
function kit.text(value, o)
	o = o or {}
	local node = ui.text(value, {
		id = o.id,
		style = ui.merge({
			font = if o.display then kit.face.display elseif o.bold then kit.face.bold elseif o.mono then MONO else kit.face.text,
			font_size = o.size or kit.m.font_size,
			color = o.color or kit.c[o.tone or "text"],
			text_wrap = if o.wrap then "word" else "none",
			-- a single line too long for its place ends in "…" instead of spilling out
			text_overflow = if o.wrap then nil else "ellipsis",
			-- a single line gives way to its neighbours and ends in "…"; wrapped text keeps its lines
			shrink = if o.wrap then 0 else 1,
			min_width = if o.wrap then nil else 0,
			max_width = if o.wrap then nil else "100%",
		}, o.style),
	})
	if o.bind and o.id then kit.bound(node, o.bind, "text") end
	return node
end

-- A section's label (SCENE, VIEWER, TIMELINE): small capitals in the text's bold face. (The display face is for large
-- headings: set this small, a brand display face such as Atelico Type reads as outlined pixels.)
-- A section's label: small, bold, muted, in its own case (never shouted in capitals).
function kit.label(value)
	return kit.text(value, { bold = true, size = kit.m.small, tone = "muted" })
end

function kit.rule(vertical)
	local c = kit.c.border
	return ui.node({ style = if vertical then { width = 2, height = "100%", shrink = 0, background = c } else { width = "100%", height = 2, shrink = 0, background = c } })
end

function kit.spacer()
	return ui.node({ style = { grow = 1, width = 0 } })
end

-- Buttons, in four kinds, every color from the theme's semantic ones (theme.toml [colors], and each look's):
--   primary    filled accent (a selected segment, Search, Ask Claude)
--   confirm    filled green (`confirm`): the button that makes a change happen (Apply changes, Install)
--   secondary  filled neutral (`button`): clearly a button, never a hairline outline alone
--   icon       a glyph on the neutral fill, with a tooltip on hover (kit.icon_button)
-- (and `play` and `danger`, filled in their own colors). A pressed button flashes for `metrics.flash` seconds on the
-- engine's clock: a confirm button a brighter green (`confirm_pressed`), any other the `pressed` color. The script
-- runtime keeps the last press (env.pressed, kit.pressed); the editor mirrors it into runtime.pressed while it is
-- recent, so the render units drawing the button draw it again when its flash starts and ends.
local KINDS = {
	primary = { fill = "accent", ink = "on_accent", pressed = "pressed" },
	confirm = { fill = "confirm", ink = "on_confirm", pressed = "confirm_pressed" },
	secondary = { fill = "button", ink = "text", pressed = "pressed" },
	icon = { fill = "button", ink = "text", pressed = "pressed" },
	play = { fill = "play", ink = "on_accent", pressed = "pressed" },
	danger = { fill = "danger", ink = "on_accent", pressed = "pressed" },
}
-- the older tones, as kinds
local TONES = { accent = "primary", raised = "secondary", confirm = "confirm", play = "play", danger = "danger", icon = "icon", primary = "primary", secondary = "secondary" }
-- a look made before the button colors (a recipe's) draws them from its nearest older ones
local OLDER = { confirm = "play", on_confirm = "on_accent", confirm_pressed = "play", button = "raised", pressed = "border" }

-- The color key a look draws `key` from: its own, else the nearest older one it has.
function kit.key(key)
	local v = kit.c[key]
	if (v == nil or v == "") and OLDER[key] then return OLDER[key] end
	return key
end

kit.kinds = KINDS

-- Whether the button `id` is flashing its press now.
function kit.flashing(id)
	if not id then return false end
	local p = store.get("runtime.pressed")
	if type(p) ~= "table" or p.id ~= id then return false end
	local since = (kit.now or 0) - (p.t or -math.huge)
	return since >= 0 and since < (kit.m.flash or 0.2)
end

-- Whether the pointer is over the button `id` (runtime.hover, written by the editor): its tooltip shows.
function kit.hovered(id)
	return id ~= nil and store.get("runtime.hover") == id
end

-- A tooltip: a small dark label, placed by `style` (absolute, beside what it names).
function kit.tooltip(text, id, style)
	local c = kit.c
	return ui.node({ id = id, style = ui.merge({ padding_x = kit.u(8), padding_y = kit.u(4), radius = kit.m.radius, background = c.header, border = 1, border_color = c.muted, shrink = 0 }, style) }, {
		kit.text(text, { size = kit.m.small, color = c.text }),
	})
end

-- The button's box, of its kind, around `kids`.
local function button_box(kind, action, o, kids)
	local c, m = kit.c, kit.m
	local k = KINDS[kind] or KINDS.secondary
	local flash = not o.disabled and kit.flashing(o.id)
	local key = kit.key(if flash then k.pressed else k.fill)
	local bg = c[key]
	-- a look with an outline draws every button's edge in its border color; otherwise the edge is the fill's own
	-- color, and hovering rings it
	local outline = kit.shape.outline
	local node = ui.button({
		id = o.id,
		on_press = action,
		disabled = o.disabled,
		-- (a flash keeps the window drawing until it is over)
		ui_state = if flash then { working = true, kind = kind } else { kind = kind },
		hover = { border_color = c.muted },
		style = ui.merge({
			height = o.height or math.max(26, math.ceil((o.size or m.font_size) * 1.25) + 8), shrink = 0, padding_x = kit.u(10), gap = kit.u(6), align = "center", justify = "center",
			radius = m.radius, background = bg, border = if outline > 0 then outline else 2, border_color = if outline > 0 then c.border else bg,
		}, o.style),
	}, kids)
	if o.id and not o.disabled then kit.bound(node, kit.color_path(key), "fill") end
	local shadow = kit.shape.shadow
	if shadow > 0 and kind ~= "secondary" and kind ~= "icon" then
		return ui.node({ style = { shrink = 0, padding_right = shadow, padding_bottom = shadow } }, {
			ui.node({ style = { absolute = true, left = shadow, top = shadow, right = 0, bottom = 0, radius = m.radius, background = c.border } }),
			node,
		})
	end
	return node
end

-- The ink a kind's label or glyph is drawn in.
function kit.ink(kind)
	return kit.c[kit.key((KINDS[kind] or KINDS.secondary).ink)]
end

-- A button. o.tone (or o.kind): "primary" ("accent"), "confirm", "secondary" ("raised", and the default), "play",
-- "danger"; o.active marks a selected segment (primary).
function kit.button(label, action, o)
	o = o or {}
	local kind = if o.active then "primary" else TONES[o.kind or o.tone or "secondary"] or "secondary"
	return button_box(kind, action, o, {
		kit.text(label, { color = if o.disabled then kit.c.muted else kit.ink(kind), size = o.size, bold = o.bold }),
	})
end

-- An icon button: `glyph` (a node, or a character) on the neutral fill, square, with `o.tip` as its tooltip while
-- the pointer is over it (above it; o.tip_outside leaves it to the caller, where a parent clips: kit.hovered).
function kit.icon_button(glyph, action, o)
	o = o or {}
	local side = o.height or math.max(26, math.ceil((o.size or kit.m.font_size) * 1.25) + 8)
	local g = if type(glyph) == "table" then glyph else kit.text(glyph, { color = kit.ink("icon"), size = o.size })
	local kids = { g }
	if o.tip and not o.tip_outside and kit.hovered(o.id) then
		local at = if o.tip_below then { top = side + kit.u(6) } else { bottom = side + kit.u(6) }
		table.insert(kids, kit.tooltip(o.tip, if o.id then o.id .. "-tip" else nil, ui.merge({ absolute = true, overlay = true, right = 0 }, at)))
	end
	return button_box("icon", action, ui.merge(o, { height = side, style = ui.merge({ width = side, padding_x = 0 }, o.style) }), kids)
end

-- A segmented control: options = {{label, value}}, `on(value)` → action.
function kit.segments(options, selected, on, o)
	local c, m = kit.c, kit.m
	-- tall enough for the look's text: a line of it and a little air
	local h = math.max(24, math.ceil((m.small + 1) * 1.25) + 4)
	local items = {}
	for _, opt in options do
		local active = opt[2] == selected
		local id = if o and o.id then o.id .. "-" .. tostring(opt[2]) else nil
		local item = ui.button({
			id = id,
			on_press = on(opt[2]),
			hover = { background = if active then nil else c.raised },
			-- (each as wide as its label at least, the room left shared out: a larger look's labels show whole in a
			-- narrow column, where equal shares cut the longest short)
			style = { grow = 1, height = h, align = "center", justify = "center", radius = m.radius - 1, background = if active then c.accent else nil, padding_x = kit.u(4) },
		}, { kit.text(opt[1], { size = m.small + 1, color = if active then c.on_accent else c.muted, bold = active, style = { width = "100%", text_align = "center" } }) })
		if active and id then kit.bound(item, kit.color_path("accent"), "fill") end
		table.insert(items, item)
	end
	-- (labels that do not fit side by side wrap to a second line of segments rather than being cut short)
	return ui.row({ style = ui.merge({ width = "100%", min_height = h + 6, shrink = 0, wrap = true, padding = 2, gap = 2, radius = m.radius + 1, background = c.background, border = 1, border_color = c.border }, o and o.style) }, items)
end

-- A panel: header (title + extra header items) over a body. `id` is `panel:<name>` for screenshots.
-- A panel: its title (a document's own, "Cel-shaded island", "Bomb explosion"; or the panel's name, "Inspector")
-- leads its head, in words, then `header`'s items.
function kit.panel(name, title, header, body, style)
	local c = kit.c
	local head = { kit.text(title, { id = "title:" .. name, bold = true, size = kit.m.small + 1, style = { shrink = 1, min_width = 0 } }) }
	for _, item in header or {} do table.insert(head, item) end
	return kit.bound(ui.column({ id = "panel:" .. name, ui = name, style = ui.merge({ background = c.panel, min_width = 0, min_height = 0, clip = true }, style) }, {
		kit.bound(ui.row({ id = "head:" .. name, style = { width = "100%", height = kit.bars().head, shrink = 0, padding_x = kit.u(12), gap = kit.u(8), align = "center", background = c.header } }, head), kit.color_path("header"), "fill"),
		kit.rule(),
		body,
	}), kit.color_path("panel"), "fill", "bottom")
end

-- A property row: muted label column, value on the right.
function kit.prop(label, value)
	-- label beside its value, or above it in a narrow panel
	-- (the value takes the room left beside the label, or the full width under it)
	return ui.row({ style = { width = "100%", gap = kit.u(8), align = "start", shrink = 0, min_height = kit.u(20), breakpoints = { { below = 250, style = { direction = "column", gap = 2, align = "stretch" } } } } }, {
		ui.node({ style = { width = kit.u(84), shrink = 0, padding_top = 2 } }, { kit.text(label, { tone = "muted", size = kit.m.small + 1 }) }),
		ui.column({ style = { grow = 1, shrink = 1, min_width = 0, gap = kit.u(4) } }, { if type(value) == "table" then value else kit.text(tostring(value), { wrap = true }) }),
	})
end

function kit.section(title, children, extra)
	local head = { kit.label(title), kit.spacer() }
	if extra then table.insert(head, extra) end
	local items = { ui.row({ style = { width = "100%", align = "center", shrink = 0, padding_bottom = 2 } }, head) }
	for _, child in children do table.insert(items, child) end
	return ui.column({ style = { width = "100%", gap = kit.m.gap, shrink = 0 } }, items)
end

-- A pill; `id` gives it (and its text, `<id>-text`) an address.
function kit.pill(value, color, id)
	local h = math.max(18, math.ceil(kit.m.small * 1.25) + 2)
	return ui.node({ id = id, style = { padding_x = kit.u(6), height = h, align = "center", radius = h / 2, background = color or kit.c.raised, shrink = 1, min_width = 0 } }, {
		kit.text(value, { id = if id then id .. "-text" else nil, size = kit.m.small, bold = true, color = if color then kit.c.on_accent else kit.c.muted }),
	})
end

-- A color swatch showing the store color at `path`.
function kit.swatch(path, size, id)
	local node = ui.node({ id = id, style = { width = size or 16, height = size or 16, shrink = 0, radius = kit.u(3), background = store.get(path), border = 1, border_color = "#ffffff26" } })
	if id then kit.bound(node, path, "swatch") end
	return node
end

-- A draggable divider between panels.
-- The line between two sections of the editor: a 2 px rule in the look's border color down the middle of the gap.
local function seam(vertical)
	local c = kit.c
	return ui.node({ style = if vertical then { width = 2, height = "100%", shrink = 0, background = c.border } else { width = "100%", height = 2, shrink = 0, background = c.border } })
end

-- A gap between two sections of the editor, `size` px across, with its seam (see `seam`).
function kit.separator(vertical, size)
	local c = kit.c
	return ui.node({ style = ui.merge(if vertical then { width = size, height = "100%" } else { width = "100%", height = size }, { shrink = 0, background = c.background, align = "center", justify = "center" }) }, { seam(vertical) })
end

function kit.splitter(which, vertical, args)
	return ui.node({
		id = "split-" .. which,
		on_pointer = kit.route("split", ui.merge({ which = which }, args)),
		hover = { background = kit.c.accent },
		style = ui.merge(if vertical then { width = kit.u(5), height = "100%" } else { width = "100%", height = kit.u(5) }, { shrink = 0, background = kit.c.background, align = "center", justify = "center" }),
	}, { seam(vertical) })
end

-- The active document (editor.ui.doc) and its kind (project.docs: scene, graph:<catalog>, timeline, theme).
-- The shape a cut is previewed in: the one asked for (editor.ui.preview_aspect), unless only the other has been
-- rendered (a 9:16 clip made only for phones shows as made, not as an empty box).
function kit.preview_shape(view)
	local asked = store.get("editor.ui.preview_aspect") or "landscape"
	local other = if asked == "portrait" then "landscape" else "portrait"
	local has = function(o)
		for _, s in (view and view.shots) or {} do
			if ((s.pictures or {})[o] or {}).media ~= "" and ((s.pictures or {})[o] or {}).media ~= nil then return true end
		end
		return false
	end
	return if not has(asked) and has(other) then other else asked
end

-- What a document is called (project.titles: its own title, else its file's name), never its path.
function kit.title(doc)
	return (store.get("project.titles") or {})[doc] or string.match(doc, "[^/]+$") or doc
end

function kit.active()
	local doc = store.get("editor.ui.doc") or ""
	return doc, (store.get("project.docs") or {})[doc]
end

-- Whether a part declared for `docs` (nil: any) shows for a document of `kind`; "graph" covers "graph:shader".
function kit.allows(docs, kind)
	if docs == nil then return true end
	for _, d in docs do
		if d == "*" or d == kind or (kind ~= nil and string.sub(kind, 1, #d + 1) == d .. ":") then return true end
	end
	return false
end

kit.inspectors = {}
kit.BOLD, kit.MONO = BOLD, MONO
return kit
