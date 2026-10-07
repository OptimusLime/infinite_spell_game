-- Claude over the engine, one element in the bottom right corner, over everything, taking no room from the layout:
--
--   button    at rest: a round button in the accent with the Claude mark and a soft glow. While Claude works a ring
--             of dots spins round it, and each note of what it does (runtime.activity: its own one-line status and
--             the tools it calls, never its replies) pops beside it as a small toast that fades; Done pops with an
--             accent tick and the ring stops.
--   card      summoned (a click, or Cmd+K): the same button stretches leftward (180 ms) into a pill-shaped field, the
--             Claude mark at its left end and the caret in it ("Ask Claude…"); Enter sends and it shrinks back into
--             the button, whose ring and toasts then show the work; Esc shrinks it back unsent. A proposal grows
--             upward out of it, with Apply changes (or Install) and Discard. The terminal icon on it (">_", its
--             tooltip "Open the terminal") opens the terminal.
--   terminal  the live Claude terminal (the engine's session, painted by the host into the `agent` surface): a large
--             floating window, about 70% of the engine; Esc or its collapse goes back to the field, then the button.
--
-- The empty engine's modal is the talk with Claude itself; nothing of this shows over it (main.luau).
local ui = require("ui")
local kit = require("kit")
local proposal = require("proposal")

local chat = { routes = {} }

local REQUEST = "bar:request"
-- how long a toast stays, and how long the button takes to stretch into its field and back (seconds)
local TOAST, MORPH = 3.0, 0.18
-- the most toasts beside the button at once
local TOASTS = 3
-- the button's margin from the window's edges (px at 1080p); the least a word is set at (the layout's readable
-- minimum, verify's)
local MARGIN, READABLE = 24, 21
-- the element's accent edge, px
local BORDER = 3

function chat.routes.bar_request(self, r, ctx)
	local text = string.gsub(string.gsub(tostring(r[REQUEST] or ""), "^%s+", ""), "%s+$", "")
	if text == "" then return end
	ctx.emit("input.set", { key = REQUEST, value = "" })
	ctx.emit("editor.ask", { kind = "request", text = text })
end

-- Claude at another level: `button`, `card` or `terminal` (the editor moves the keys' focus with it).
function chat.routes.claude(self, r, ctx)
	ctx.emit("editor.claude", { level = r.level })
end

-- The accent at an alpha (0..1).
local function accent(alpha)
	return string.format("%s%02x", string.sub(kit.c.accent, 1, 7), math.floor(math.clamp(alpha, 0, 1) * 255 + 0.5))
end

-- A small button on the card's head: its label at the card's size, its padding with it.
local function head_button(label, route, id, z)
	return kit.button(label, route, { id = id, height = kit.line(z.text) + math.floor(8 * z.k + 0.5), size = z.text, style = { padding_x = math.floor(12 * z.k + 0.5), gap = 0 } })
end

-- A terminal, drawn (the faces have no terminal glyph): a small rounded screen with a ">_" prompt in it.
local function terminal_glyph(side, ink)
	local w, h = math.floor(side * 0.66 + 0.5), math.floor(side * 0.52 + 0.5)
	local edge = math.max(2, math.floor(side * 0.045 + 0.5))
	local t = math.max(2, math.floor(side * 0.07 + 0.5))
	local l = math.floor(h * 0.4 + 0.5)
	local dy = l * math.sin(math.rad(40))
	local cx, cy = math.floor(w * 0.32), math.floor(h * 0.5 - edge)
	local bar = function(y, angle)
		return ui.node({ style = { absolute = true, left = cx - l / 2, top = y - t / 2, width = l, height = t, radius = t / 2, background = ink, rotation = angle } })
	end
	-- ">": its upper stroke down to the right, its lower one back down to the left; "_" after it on the baseline
	return ui.node({ id = "claude-expand-glyph", style = { width = w, height = h, shrink = 0, radius = math.max(2, math.floor(side * 0.08)), border = edge, border_color = ink } }, {
		bar(cy - dy / 2, -40),
		bar(cy + dy / 2, 40),
		ui.node({ style = { absolute = true, left = math.floor(cx + l * 0.55), top = math.floor(cy + dy / 2 + l * 0.2), width = math.floor(w * 0.3), height = t, radius = t / 2, background = ink } }),
	})
end

-- The field's terminal icon: opens the terminal; its tooltip (drawn by chat.corner, outside the pill that clips)
-- says so.
local function terminal_button(z)
	local side = kit.line(z.text) + math.floor(8 * z.k + 0.5)
	return kit.icon_button(terminal_glyph(side, kit.ink("icon")), kit.route("claude", { level = "terminal" }), { id = "claude-expand", height = side, tip = "Open the terminal", tip_outside = true, style = { radius = side / 2 } })
end

-- The width a line of `text` takes at `size`, bold or not.
local function wide(text, size, bold)
	return select(2, typeset.lines(if bold then kit.face.bold else (kit.face.text or "fonts/Roobert-Regular.ttf"), size, 1e6, text)) or (#text * size * 0.6)
end

-- Sizes for a window `w` x `h`: the whole surface is one unit, scaled as a 1080p picture is (the button 88 px there,
-- its words 27 px, never under that from 1080 up), so a small window (a cube face shot at 1067x600 and scale 1.8)
-- draws it as a smaller picture of the same thing. The card is never narrower than its head.
local function sizes(w, h)
	local k = math.min(w, h) / 1080
	-- (never under the editor's readable least, 21 px: a small window keeps its words readable)
	local text = math.max(READABLE, math.floor(kit.m.font_size * math.min(k, 1) + 0.5))
	local margin = math.floor(MARGIN * k + 0.5)
	local b = math.floor(88 * k + 0.5)
	-- the field at its least: the mark's circle, "Ask Claude…" and the terminal icon, their gaps and padding
	local least = b + wide("Ask Claude…", text) + kit.line(text) + math.floor(8 * k + 0.5) + 4 * math.floor(12 * k)
	local field = if h > w then math.floor(w * 0.92) else math.floor(560 * k + 0.5)
	return {
		k = k,
		button = b,
		ring = b + math.floor(28 * k),
		margin = margin,
		text = text, -- 27 px at 1080p: every word Claude shows reads in the video
		card = math.min(math.max(field, math.ceil(least)), w - 2 * margin),
		tall = h > w,
	}
end

-- How far the morph between the button and its field has gone (0: the button, 1: the field), at time `t`.
local function stretched(level, t)
	local moved = store.get("runtime.claude_moved") or {}
	local since = t - (moved.at or -math.huge)
	local e = math.clamp(since / MORPH, 0, 1)
	e = e * e * (3 - 2 * e)
	if level == "card" then return if moved.from == "button" then e else 1 end
	if level == "button" and moved.from == "card" then return 1 - e end
	return 0
end

-- The Claude mark: a rounded diamond.
local function mark(size, color, id)
	return ui.node({ id = id, style = { width = size, height = size, radius = math.floor(size * 0.22), shrink = 0, background = color, rotation = 45 } })
end

-- The element: the round button with its glow (and, while Claude works, the ring of dots going round it), or, as far
-- as it has stretched (`e`), the field it becomes: the mark's circle at its left end, the field, the terminal icon.
local function entry(z, working, t, e, focus, inputs)
	local b, ring = z.button, z.ring
	local pad = (ring - b) / 2
	-- (never narrower than what the pill holds at its least: its button, b - 6, a border's width of padding each side
	-- and its border; the morph's last frames in a small window came within 4 px of the button, which spilled)
	local width = math.max(math.floor(b + (z.card - b) * e + 0.5), b + 2 * BORDER)
	local kids = {}
	if e == 0 then
		-- the glow: soft rings of the accent behind it
		table.insert(kids, ui.node({ style = { absolute = true, left = 0, top = 0, width = ring, height = ring, radius = ring / 2, background = accent(0.14) } }))
		table.insert(kids, ui.node({ style = { absolute = true, left = pad - 5, top = pad - 5, width = b + 10, height = b + 10, radius = (b + 10) / 2, background = accent(0.3) } }))
		if working then
			-- twelve dots round the button, the lit one going round a second and a half a turn, the ones behind it
			-- fading
			local n, r, d = 12, ring / 2 - 4 * z.k, math.max(4, math.floor(7 * z.k))
			for i = 0, n - 1 do
				local a = i / n * 2 * math.pi
				local lag = ((t / 1.5) - i / n) % 1
				table.insert(kids, ui.node({ style = {
					absolute = true, left = ring / 2 + math.sin(a) * r - d / 2, top = ring / 2 - math.cos(a) * r - d / 2,
					width = d, height = d, radius = d / 2, background = accent(1 - lag * 0.85),
				} }))
			end
		end
		table.insert(kids, ui.button({
			id = "claude-button", ui = "claude", ui_state = { working = working }, on_press = kit.route("claude", { level = "card" }),
			hover = { background = kit.c.accent }, style = {
				absolute = true, left = pad, top = pad, width = b, height = b, radius = b / 2,
				align = "center", justify = "center", background = kit.c.accent,
			},
		}, { mark(math.floor(b * 0.36), kit.c.on_accent, "claude-button-mark") }))
		return ui.node({ id = "claude-button-ring", style = { width = ring, height = ring, shrink = 0 } }, kids)
	end
	-- stretched: a pill as tall as the button, its right end where the button was, the mark's circle at its left end
	local c = kit.c
	local regular = kit.face.text or "fonts/Roobert-Regular.ttf"
	local gap = math.floor(12 * z.k)
	local row = {
		ui.button({ id = "claude-button", ui = "claude", ui_state = { working = true }, on_press = kit.route("claude", { level = "button" }), style = {
			width = b - 6, height = b - 6, radius = (b - 6) / 2, shrink = 0, align = "center", justify = "center", background = c.accent,
		} }, { mark(math.floor(b * 0.3), c.on_accent, "claude-button-mark") }),
	}
	if e > 0.5 then
		-- (in a box that clips it: typed text longer than the field never runs under the terminal icon)
		table.insert(row, ui.node({ id = "claude-field", style = { grow = 1, width = 0, min_width = 0, clip = true } }, {
			ui.input({
				id = "event-request", ui = "request", bind = REQUEST, value = (inputs or {})[REQUEST] or "",
				placeholder = "Ask Claude…", on_submit = kit.route("bar_request", {}), focus_border = c.panel,
				style = { width = "100%", height = kit.line(z.text) + 2 * math.floor(6 * z.k + 0.5), padding = math.floor(6 * z.k + 0.5), border = 0, background = c.panel, color = c.text,
					font = regular, font_size = z.text, text_wrap = "none" },
			}),
		}))
		table.insert(row, terminal_button(z))
	end
	return ui.row({ id = "request-card", ui = "ask", style = {
		-- (the mark's circle a border's width in from the pill's edge)
		width = width, height = b, shrink = 0, padding_left = BORDER, padding_right = if e > 0.5 then gap else BORDER, gap = gap, align = "center", clip = true,
		radius = b / 2, background = c.panel, border = 3, border_color = c.accent,
	} }, row)
end

-- The toasts: the notes of the last few seconds, one a toast (Done with an accent tick), newest nearest the button.
local function toasts(z, now, t, room)
	local notes, at = now.notes or {}, now.noted_at or {}
	local out = {}
	for i = 1, #notes do
		local age = t - (at[i] or -math.huge)
		if age >= 0 and age < TOAST then
			local fade = if age > TOAST - 0.8 then (TOAST - age) / 0.8 else 1
			local done = notes[i] == "Done"
			local c = kit.c
			local words = {}
			if done then table.insert(words, kit.text("✓", { size = z.text, bold = true, color = c.accent, style = { shrink = 0 } })) end
			table.insert(words, kit.text(notes[i], { id = "claude-toast-text", size = z.text, bold = done, color = if done then c.accent else c.text, wrap = true, style = { shrink = 1, min_width = 0, max_width = room } }))
			table.insert(out, ui.row({ id = "claude-toast", ui = "toast", style = {
				shrink = 0, max_width = room, padding_x = math.floor(18 * z.k), padding_y = math.floor(10 * z.k), gap = math.floor(10 * z.k), align = "center",
				radius = math.floor(18 * z.k), background = c.panel, border = 2, border_color = accent(0.35 + 0.65 * fade), opacity = fade,
			} }, words))
		end
	end
	return out
end

-- The proposal, grown upward out of the field: as wide as it, over it.
local function asking(z)
	local p, answer = proposal.card()
	if not p then return nil end
	local c = kit.c
	return ui.column({ id = "claude-proposal", style = {
		width = z.card, shrink = 1, min_height = 0, padding = math.floor(20 * z.k), gap = math.floor(10 * z.k), clip = true,
		radius = math.floor(20 * z.k), background = c.panel, border = 3, border_color = c.accent,
	} }, {
		ui.row({ style = { width = "100%", gap = math.floor(10 * z.k), align = "center", shrink = 0 } }, {
			mark(math.floor(22 * z.k), c.accent),
			kit.text("Claude proposes", { bold = true, size = z.text, color = c.accent }),
		}),
		p,
		answer,
	})
end

-- The terminal: the live Claude session in a large floating window, with a collapse back to the card.
local function terminal(z, w, h, bottom)
	local c = kit.c
	local tw, th = math.floor(w * 0.7), math.floor(math.min(h * 0.7, h - bottom - kit.bars().toolbar - z.margin))
	local head = ui.row({ id = "claude-terminal-head", style = { width = "100%", height = kit.line(z.text) + math.floor(24 * z.k), shrink = 0, padding_x = math.floor(18 * z.k), gap = math.floor(12 * z.k), align = "center", background = c.header } }, {
		mark(math.floor(22 * z.k), c.accent),
		kit.text("Claude", { bold = true, size = z.text, color = c.accent, style = { shrink = 0 } }),
		-- (no session status: its words, "reattached session 'atelico-agent-…'", are jargon on screen)
		kit.spacer(),
		head_button("Back to card", kit.route("claude", { level = "card" }), "claude-collapse", z),
	})
	return ui.column({ id = "claude-terminal", ui = "terminal", style = {
		absolute = true, overlay = true, right = z.margin, bottom = bottom, width = tw, height = th,
		radius = math.floor(18 * z.k), clip = true, background = c.panel, border = 3, border_color = c.accent,
	} }, {
		head,
		{ kind = "surface", surface = "agent", style = { width = "100%", grow = 1, min_height = 0 } },
	})
end

-- Everything of Claude's over the window, for main.luau's overlay: the button always; toasts beside it while it
-- works; the card or the terminal when summoned (the card whenever a proposal waits).
function chat.corner(env)
	local w, h, t = env.width, env.height, clock.now()
	local z = sizes(w, h)
	local now = store.get("runtime.activity") or {}
	local level = store.get("editor.ui.claude") or "button"
	local proposing = proposal.showing(store.get("runtime.proposal") or {})
	if proposing and level == "button" then level = "card" end
	local e = stretched(level, t)
	local over = {}
	-- at the foot of the window, over the status bar, in both shapes; in a tall window within the strip kept for it
	-- (runtime.claude_reserve: a cut's subtitles stand above it)
	local bottom = kit.bars().status + z.margin
	local element = entry(z, now.working == true, t, e, env.focus, env.inputs)
	-- (the element keeps the window drawing while it stretches or shrinks)
	if e > 0 and e < 1 then element.ui_state = { working = true } end
	-- the toasts (at rest): to the button's left, stacked up from its foot; in a tall window only as many as fit in
	-- Claude's strip (the newest)
	local room = w - z.ring - 3 * z.margin
	local popped = if e == 0 then toasts(z, now, t, room) else {}
	-- three at most (the newest): more would climb over what Claude is changing
	while #popped > TOASTS do table.remove(popped, 1) end
	if z.tall then
		local strip = h * (store.get("runtime.claude_reserve") or 0.16) - bottom
		local each = kit.line(z.text) + 2 * math.floor(10 * z.k) + 4 + math.floor(10 * z.k)
		local fits = math.max(1, math.floor(strip / each))
		while #popped > fits do table.remove(popped, 1) end
	end
	-- a proposal grows up out of the field
	local grown = if e == 1 then asking(z) else nil
	-- (the field sits where the button's disc sits, inside its ring)
	local inset = if e > 0 then (z.ring - z.button) / 2 else 0
	if grown then
		local col = {}
		for _, toast in popped do table.insert(col, toast) end
		if grown then table.insert(col, grown) end
		table.insert(col, element)
		table.insert(over, ui.column({ id = "claude-corner", style = { absolute = true, overlay = true, right = z.margin + inset, bottom = bottom + inset, gap = math.floor(12 * z.k), align = "end",
			max_height = h - bottom - kit.bars().toolbar - z.margin } }, col))
	else
		local stack = ui.column({ style = { gap = math.floor(10 * z.k), align = "end", shrink = 1, min_width = 0 } }, popped)
		table.insert(over, ui.row({ id = "claude-corner", style = { absolute = true, overlay = true, right = z.margin + inset, bottom = bottom + inset, gap = z.margin, align = "center" } }, {
			stack, element,
		}))
	end
	-- the terminal icon's tooltip, over the pill's right end (the pill clips what it holds)
	if e == 1 and kit.hovered("claude-expand") then
		table.insert(over, kit.tooltip("Open the terminal", "claude-expand-tip", { absolute = true, overlay = true, right = z.margin + inset, bottom = bottom + inset + z.button + math.floor(10 * z.k) }))
	end
	if level == "terminal" then
		table.insert(over, terminal(z, w, h, bottom + z.ring + math.floor(12 * z.k)))
	end
	return over
end

return chat
