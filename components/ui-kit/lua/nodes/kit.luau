-- ui-kit's scene node types: the App root, Panel, Text, Button, Input, Image, Choice and Chat, each a data
-- definition (its props) and how it is drawn. Colors left unset come from the App's `colors`.
local ui = require("ui")
local nodes = require("nodes")
local interp, shown, last, color, bound, text = nodes.interp, nodes.shown, nodes.last, nodes.color, nodes.bound, nodes.text
local function f(kind, doc, default) return { kind = kind, doc = doc, default = default } end
local function one_of(options, doc, default) return { kind = "one_of", options = options, doc = doc, default = default } end
local SHOW = f("text", "Shown only when the app's value of this name is true (\"!name\": when false)")
local WIDTH = f("any", "A number of pixels or a percentage (\"100%\")")

local types = {
	App = {
		doc = "An app's root: the installed app whose routes and state drive the nodes below, and its theme",
		app = f("text", "The installed app (its routes, state and values)"),
		theme = f("text", "A Luau module whose colors table follows `colors` (for code that draws with it)"),
		title = f("text", "The header's title"),
		header = f("bool", "Show the header", true),
		title_size = { kind = "number", min = 8, max = 96, default = 30 },
		font = f("text", "The font file", "themes/atelico/roobert.ttf"),
		font_size = { kind = "number", min = 8, max = 96, default = 20 },
		padding = { kind = "number", min = 0, max = 200, default = 16 },
		spacing = { kind = "number", min = 0, max = 200, default = 14, doc = "Space between the nodes" },
		align = one_of({ "start", "center", "end", "stretch" }, "Cross-axis alignment of the nodes", "center"),
		colors = { kind = "map", of = "color", doc = "The theme: background, panel, raised, border, text, muted, accent, call, red" },
	},
	Panel = {
		doc = "A box of nodes, laid out in a row or a column",
		direction = one_of({ "row", "column" }, nil, "column"),
		gap = { kind = "number", min = 0, max = 200 },
		padding = { kind = "number", min = 0, max = 200 },
		align = one_of({ "start", "center", "end", "stretch" }),
		justify = one_of({ "start", "center", "end", "space_between" }),
		background = f("color"),
		border = { kind = "number", min = 0, max = 20 },
		border_color = f("color"),
		radius = { kind = "number", min = 0, max = 200 },
		width = WIDTH,
		height = WIDTH,
		grow = { kind = "number", min = 0, max = 10 },
		wrap = f("bool"),
		show = SHOW,
	},
	Text = {
		doc = "Text; {name} is replaced by the app's value of that name",
		text = f("text"),
		size = { kind = "number", min = 6, max = 200 },
		color = f("color", "Default: the theme's text"),
		font = f("text"),
		align = one_of({ "left", "center", "right" }),
		wrap = f("bool", nil, true),
		width = WIDTH,
		grow = { kind = "number", min = 0, max = 10 },
		show = SHOW,
	},
	Button = {
		doc = "A button that calls one of the app's routes",
		text = f("text", "Its label; {name} is replaced by the app's value of that name"),
		action = f("text", "The app route it calls"),
		args = f("any", "Arguments for the route"),
		tone = one_of({ "call", "plain", "end" }, "Default colors: call is the theme's call color, end its red, plain its raised", "plain"),
		background = f("color", "Default: from its tone"),
		color = f("color", "The label's color (default: the theme's text)"),
		border_color = f("color", "Default: the theme's border"),
		hover = f("color", "Background under the pointer"),
		size = { kind = "number", min = 6, max = 200, doc = "The label's size" },
		height = { kind = "number", min = 16, max = 400, default = 48 },
		radius = { kind = "number", min = 0, max = 200 },
		width = WIDTH,
		grow = { kind = "number", min = 0, max = 10 },
		show = SHOW,
	},
	Input = {
		doc = "A text field; what is typed is the app's input named `bind`",
		bind = f("text", "The input's name (default: the node's name)"),
		label = f("text"),
		placeholder = f("text"),
		action = f("text", "The route Enter calls"),
		size = { kind = "number", min = 6, max = 200, default = 24 },
		height = { kind = "number", min = 16, max = 800, default = 48 },
		background = f("color", "Default: the theme's background"),
		color = f("color"),
		border_color = f("color"),
		show = SHOW,
	},
	Image = {
		doc = "A framed picture (animated: an illustration atlas that plays)",
		texture = f("text"),
		animate = f("bool"),
		size = WIDTH,
		aspect = { kind = "number", min = 0.1, max = 10, default = 1 },
		radius = { kind = "number", min = 0, max = 200, default = 24 },
		border = { kind = "number", min = 0, max = 20, default = 3 },
		border_color = f("color"),
		show = SHOW,
	},
	Choice = {
		doc = "A row of buttons, one per option; the selected one is highlighted",
		options = f("list", "Option values (or the app's value named by `source`)"),
		source = f("text", "The app value listing the options ({label, value, current})"),
		bind = f("text", "The input a press sets (default: the node's name)"),
		action = f("text", "A route a press calls instead, with {[arg] = value}"),
		arg = f("text", nil, "value"),
		size = { kind = "number", min = 6, max = 200, default = 18 },
		selected = f("color", "The selected option's background (default: the theme's call)"),
		width = WIDTH,
		show = SHOW,
	},


	Chat = {
		doc = "A conversation (the app's value named by `source`) with a field to type into",
		source = f("text", nil, "turns"),
		bind = f("text", nil, "message"),
		placeholder = f("text"),
		send = f("text", "The route sending the message", "send"),
		back = f("text", "The route going back (called with {open = false})"),
		size = { kind = "number", min = 6, max = 200, default = 18 },
		send_background = f("color", "Default: the theme's accent"),
		width = WIDTH,
		height = WIDTH,
		show = SHOW,
	},
}


-- The App node is the root the renderer draws itself (header, theme); it has no builder of its own.
nodes.register("App", types.App, nil)


local build = {}
function build.Panel(ctx, path, p, kids)
	local style = {
		gap = p.gap, padding = p.padding, align = p.align, justify = p.justify, border = p.border, radius = p.radius,
		border_color = p.border_color, width = p.width or "100%", height = p.height, grow = p.grow, wrap = p.wrap, shrink = 0, min_width = 0,
	}
	if p.direction ~= "row" then style.direction = "column" end
	local node = ui.node({ id = last(path), style = style }, kids)
	if p.background then
		style.background = p.background
		bound(node, `{ctx.base}.{path}.props.background`, "fill")
	end
	return node
end

function build.Text(ctx, path, p)
	local fg = color(ctx, path, p, "color", "text")
	return text(ctx, path, p, "text", last(path), {
		font = p.font or ctx.font, font_size = p.size or ctx.font_size, color = fg, text_align = p.align,
		text_wrap = if p.wrap == false then "none" else "word", width = p.width, grow = p.grow, shrink = 0,
	})
end

local TONE = { call = "call", plain = "raised", ["end"] = "red" }

function build.Button(ctx, path, p)
	local name = last(path)
	local bg, bg_path = color(ctx, path, p, "background", TONE[p.tone or "plain"] or "raised")
	local fg = color(ctx, path, p, "color", "text")
	local border = color(ctx, path, p, "border_color", "border")
	-- pressed a moment ago (under 200 ms, on the app's clock: env.pressed): it flashes, a call to act (tone call)
	-- in the app's confirm colour, any other in its pressed colour (the colours' own keys, else near ones)
	local pressed = ctx.env and ctx.env.pressed
	local flashing = pressed and pressed.id == name and (ctx.env.time or 0) - (pressed.t or 0) < 0.2
	if flashing then
		bg = if p.tone == "call" then (ctx.colors.confirm or ctx.colors.accent or bg) else (ctx.colors.pressed or ctx.colors.border or bg)
		bg_path = nil
	end
	local label = text(ctx, path, p, "text", name .. "-label", { font = ctx.font, font_size = p.size or ctx.font_size, color = fg, text_wrap = "none" })
	local h = p.height or 48
	local button = ui.button({
		id = name,
		on_press = if p.action then ui.action("route", { name = p.action, args = p.args }) else nil,
		style = { height = h, min_height = h, shrink = 0, padding_x = 12, align = "center", justify = "center", border = 2, border_color = border, background = bg, radius = p.radius, width = p.width, grow = p.grow, min_width = 0, gap = 8 },
		hover = if p.hover then { background = p.hover } else nil,
	}, { label })
	return if bg_path then bound(button, bg_path, "fill") else button
end

function build.Input(ctx, path, p)
	local name = last(path)
	local key = p.bind or name
	local bg, bg_path = color(ctx, path, p, "background", "background")
	local fg = color(ctx, path, p, "color", "text")
	local border = color(ctx, path, p, "border_color", "border")
	local h, size = p.height or 48, p.size or 24
	local field = ui.input({
		id = name, bind = key, value = ctx.env.inputs[key] or "", placeholder = interp(p.placeholder, ctx.values),
		on_submit = if p.action then ui.action("route", { name = p.action }) else nil, focus_border = ctx.colors.accent,
		style = { width = "100%", min_width = 0, height = h, min_height = h, max_height = h, shrink = 0, padding = 10, border = 2, border_color = border, background = bg, color = fg, font = ctx.font, font_size = size, text_wrap = if h > 2 * size + 20 then "word" else "none" },
	})
	bound(field, bg_path, "fill")
	if not p.label then return field end
	return ui.column({ id = name .. "-field", style = { width = "100%", gap = 8, shrink = 0 } }, {
		text(ctx, path, p, "label", name .. "-title", { font = ctx.font, font_size = math.floor(size * 0.75 + 0.5), color = ctx.colors.muted, shrink = 0 }),
		field,
	})
end

function build.Image(ctx, path, p)
	local name = last(path)
	local border = color(ctx, path, p, "border_color", "text")
	local style = { width = "100%", height = "100%", radius = math.max(0, (p.radius or 24) - (p.border or 3)) }
	local picture = if p.animate then require("ui.editorial").image(p.texture or "", ctx.env.time, style) else ui.image({ texture = p.texture, style = style })
	return ui.node({ id = name, style = { width = p.size or "100%", aspect = p.aspect or 1, radius = p.radius or 24, border = p.border or 3, border_color = border, clip = true, shrink = 0 } }, { picture })
end

local function capital(s)
	return s:sub(1, 1):upper() .. s:sub(2)
end

function build.Choice(ctx, path, p)
	local name = last(path)
	local key = p.bind or name
	local options = if p.source then ctx.values[p.source] or {} else p.options or {}
	local selected_bg, selected_path = color(ctx, path, p, "selected", "call")
	local buttons = {}
	for _, o in options do
		local value = if type(o) == "table" then o.value else o
		local label = if type(o) == "table" then o.label else capital(tostring(o))
		local current = if type(o) == "table" then o.current else (ctx.env.inputs[key] or options[1]) == value
		local action = if p.action then ui.action("route", { name = p.action, args = { [p.arg or "value"] = value } }) else ui.action("input", { key = key, value = value })
		local b = ui.button({
			id = `{name}-{string.lower(tostring(label))}`,
			on_press = action,
			style = { grow = 1, height = 44, min_height = 44, shrink = 0, padding_x = 8, align = "center", justify = "center", border = 2, border_color = ctx.colors.border, background = if current then selected_bg else ctx.colors.raised, min_width = 0 },
		}, { ui.text(label, { style = { font = ctx.font, font_size = p.size or 18, color = ctx.colors.text, text_wrap = "none" } }) })
		if current then bound(b, selected_path, "fill") end
		table.insert(buttons, b)
	end
	return ui.row({ id = name, style = { width = p.width or "100%", gap = 8, shrink = 0 } }, buttons)
end

local function icon(ctx, name, action, id, background)
	return ui.button({ id = id, on_press = action, style = { width = 44, height = 44, min_height = 44, shrink = 0, align = "center", justify = "center", border = 2, border_color = ctx.colors.text, background = background or ctx.colors.background } }, {
		ui.image({ texture = `themes/atelico/{name}.png`, style = { width = 26, height = 26 } }),
	})
end

function build.Chat(ctx, path, p)
	local name = last(path)
	local key = p.bind or "message"
	local conversation = require("phone.scenes.conversation")
	local turns = ctx.values[p.source or "turns"] or {}
	local send = ui.action("route", { name = p.send or "send" })
	local send_bg = color(ctx, path, p, "send_background", "accent")
	local row = {}
	if p.back then table.insert(row, icon(ctx, "back", ui.action("route", { name = p.back, args = { open = false } }), "grid-back")) end
	table.insert(row, ui.input({
		id = key, bind = key, value = ctx.env.inputs[key] or "", placeholder = interp(p.placeholder, ctx.values), on_submit = send, focus_border = ctx.colors.accent,
		-- (as tall as a line of its text and its padding, 48 at least)
		style = { width = 0, grow = 1, min_width = 0, height = math.max(48, math.ceil((p.size or 18) * 1.25) + 24), min_height = 48, shrink = 0, padding = 10, border = 2, border_color = ctx.colors.border, background = ctx.colors.background, color = ctx.colors.text, font = ctx.font, font_size = p.size or 18, text_wrap = "none" },
	}))
	table.insert(row, icon(ctx, "send", send, "send", send_bg))
	return ui.column({ id = name, style = { width = p.width or "100%", height = p.height, gap = 8, shrink = 0 } }, {
		ui.column({ id = name .. "-scroll", style = { width = "100%", grow = 1, min_height = 0, scroll = true, scroll_to = "end" } }, {
			conversation.render({ state = { turns = turns, connected = true } }, ctx.env, { name = ctx.values.name or "", size = p.size }),
		}),
		ui.row({ style = { width = "100%", gap = 8, shrink = 0, align = "end" } }, row),
	})
end


for name, fields in types do
	if name ~= "App" then nodes.register(name, fields, build[name]) end
end
return types
