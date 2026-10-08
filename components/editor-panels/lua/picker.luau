-- A color picker bound to one store path. Everything it shows (SV square, both handles, swatch, hex field)
-- is derived from that one value on every render; it keeps no copy of its own. A drag keeps the hue it
-- started with because the hue travels in the pointer route's args, captured when the drag begins.
local ui = require("ui")
local kit = require("kit")
local color = require("color")

local picker = { routes = {} }

-- Picks `path` for the picker; a color not set yet starts at `start`.
function picker.routes.pick(self, r, ctx)
	if store.get(r.path) == nil then
		local ok, err = store.set(r.path, r.start)
		if not ok then return ctx.emit_error(err) end
	end
	store.set("editor.ui.color", r.path)
end

function picker.view(path, focus, typed)
	local c, m = kit.c, kit.m
	local hex = store.get(path) or "#808080"
	local h, s, v = color.hsv(hex)
	local pure = color.from_hsv(h, 1, 1)
	local sv = kit.bound(ui.node({ id = "picker-sv", on_pointer = kit.route("sv", { path = path, hue = h }), style = { width = "100%", height = 150, shrink = 0, radius = m.radius, clip = true, gradient = { "#ffffff", pure } } }, {
		ui.node({ style = { absolute = true, left = 0, top = 0, width = "100%", height = "100%", gradient = { "#00000000", "#000000" }, gradient_dir = "y" } }),
		kit.bound(ui.node({ id = "picker-sv-handle", style = { absolute = true, left = `{s * 100}%`, top = `{(1 - v) * 100}%`, margin_left = -7, margin_top = -7, width = 14, height = 14, radius = 7, border = 2, border_color = "#ffffff", background = hex } }), path, "sv"),
	}), path, "square")
	local hue = ui.node({ id = "picker-hue", on_pointer = kit.route("hue", { path = path }), style = { width = "100%", height = 14, shrink = 0, radius = 7, gradient = { "#ff0000", "#ffff00", "#00ff00", "#00ffff", "#0000ff", "#ff00ff", "#ff0000" } } }, {
		kit.bound(ui.node({ id = "picker-hue-handle", style = { absolute = true, left = `{h / 360 * 100}%`, top = -2, margin_left = -9, width = 18, height = 18, radius = 9, border = 2, border_color = "#ffffff", background = pure } }), path, "hue"),
	})
	-- The field shows the value, except while it has focus and holds what is being typed.
	local editing = focus == "hex"
	local field = ui.input({
		id = "hex", bind = "hex", value = if editing then (typed or "") else hex, placeholder = hex, on_submit = kit.route("hex", { path = path }), focus_border = c.accent,
		style = { grow = 1, width = 0, height = 28, padding = 6, border = 1, radius = m.radius, border_color = c.border, background = c.background, color = c.text, font = kit.MONO, font_size = 12, text_wrap = "none" },
	})
	if not editing then kit.bound(field, path, "input") end
	return ui.column({ style = { width = "100%", gap = 10, shrink = 0 } }, {
		sv,
		hue,
		ui.row({ style = { width = "100%", gap = 8, align = "center" } }, { kit.swatch(path, 28, "picker-swatch"), field }),
	})
end

function picker.routes.sv(self, r, ctx)
	store.set(r.path, color.from_hsv(r.hue, r.x, 1 - r.y))
end

-- Moving the hue of a grey or black gives it some saturation and value, so the move shows.
function picker.routes.hue(self, r, ctx)
	local _, s, v = color.hsv(store.get(r.path) or "#808080")
	if s < 0.02 then s = 0.75 end
	if v < 0.02 then v = 0.75 end
	store.set(r.path, color.from_hsv(math.clamp(r.x, 0, 0.999) * 360, s, v))
end

function picker.routes.hex(self, r, ctx)
	local hex = tostring(r.hex or ""):gsub("%s", "")
	ctx.emit("input.set", { key = "hex", value = "" })
	ctx.emit("input.blur", {})
	if not color.valid(hex) then return end
	if hex:sub(1, 1) ~= "#" then hex = "#" .. hex end
	store.set(r.path, hex)
end

return picker
