-- The empty engine: a blank screen in the window's look (Atelico dark for a new project) and one modal on it, which
-- is the whole conversation that turns it into an engine (docs/launch-video/sources.md §6):
--   1. it asks "What kind of engine do you need today?", the answer box inside it;
--   2. the answer sent (to Claude's session, editor.ask kind "build"), the same modal is the talk with Claude: what
--      was asked, and Claude's short notes (its `say` lines on runtime.chat, and what it is doing, runtime.activity),
--      never its replies;
--   3. what Claude proposes (runtime.proposal), each package pictured by its capture, and Install;
--   4. "Installing…" as each package lands (registry.landed), then "Restarting…" while they take effect
--      (registry.phase), then the engine is the new one and the window turns into it.
-- The answer is kept at editor.ui.brief.
local ui = require("ui")
local kit = require("kit")
local proposal = require("proposal")

local welcome = { routes = {} }
local BRIEF = "welcome:brief"
local QUESTION = "What kind of engine do you need today?"
-- the modal's width at most, and its margin from the window's edges (kit units)
local MAX_W, MARGIN = 820, 32

function welcome.routes.welcome_build(self, r, ctx)
	local text = string.gsub(tostring(r[BRIEF] or ""), "^%s+", "")
	text = string.gsub(text, "%s+$", "")
	if text == "" then return end
	ctx.emit("input.blur", {})
	store.set("editor.ui.brief", text)
	ctx.emit("editor.ask", { kind = "build", text = text })
end

-- Claude's short lines to the person (its `say`s), newest last.
local function claude_lines()
	local out = {}
	for _, m in (store.get("runtime.chat") or {}).messages or {} do
		if m.from == "claude" then table.insert(out, m.text) end
	end
	return out
end

-- A sentence whole, its first one (the rest stays in the store and the terminal).
local function first_sentence(text)
	return string.match(text, "^(.-[%.!?])%s") or text
end

-- A heading in the bold text face, as written (the brand face is for the opening and the reveal only).
local function heading(text, size, o)
	o = o or {}
	return kit.text(text, { id = o.id, bold = true, size = size, wrap = true, color = o.color, style = o.style or { width = "100%" } })
end

local function line(text, o)
	o = o or {}
	return kit.text(text, { id = o.id, bold = o.bold, size = o.size or kit.m.font_size, tone = o.tone, color = o.color, wrap = true, style = { width = "100%" } })
end

-- Claude's mark, and its note beside it.
local function said(text, id, c)
	return ui.row({ id = id, style = { width = "100%", gap = kit.u(12), align = "center", shrink = 0 } }, {
		ui.node({ style = { width = kit.u(30), height = kit.u(30), radius = kit.u(15), background = c.accent, align = "center", justify = "center", shrink = 0 } }, {
			kit.text("C", { bold = true, size = kit.u(15), color = c.on_accent }),
		}),
		kit.text(first_sentence(text), { bold = true, size = kit.m.font_size, wrap = true, style = { grow = 1, width = 0 } }),
	})
end

-- What the person asked, as a quote.
local function asked(brief, c)
	return ui.node({ id = "welcome-said", style = { width = "100%", padding_x = kit.u(14), padding_y = kit.u(10), radius = kit.u(12), background = c.raised, shrink = 0 } }, {
		kit.text("“" .. brief .. "”", { id = "welcome-brief-said", size = kit.m.small + 1, wrap = true, style = { width = "100%" } }),
	})
end

-- A tile's caption: one short line, whole. The line as Claude wrote it when it fits, else its first part (to a comma or
-- "and") when that says something whole, else none.
local function caption(text, w)
	local size = kit.m.small
	local face = kit.face.text or "fonts/Roobert-Regular.ttf"
	local fits = function(t) return (select(2, typeset.lines(face, size, 1e6, t)) or math.huge) <= w end
	local first = string.match(text, "^(.-),") or string.match(text, "^(.-) and ") or text
	-- (a first part of three words or more says something whole; "a round" does not)
	local whole = select(2, string.gsub(first, "%S+", "")) >= 3
	local shown = if fits(text) then text elseif whole and fits(first) then first else nil
	if not shown then return ui.node({ style = { height = 0 } }) end
	return kit.text(shown, { size = size, tone = "muted", style = { width = w } })
end

-- A package as a picture (its capture) with its name and one line under it; `done`: it has landed.
local function tile(ch, w, c, done)
	local h = math.floor(w * 9 / 16)
	local picture = if ch.workspace then proposal.workspace(ch.workspace, w, h)
		elseif ch.image and ch.image ~= "" then ui.image({ texture = ch.image, style = { width = w, height = h, radius = kit.u(8) } })
		else ui.node({ style = { width = w, height = h, radius = kit.u(8), background = c.raised } })
	-- what people call it (its title), never its package name
	local name = if ch.action == "apply" then "Your editor" elseif ch.title and ch.title ~= "" then ch.title else ch.name
	local mark = if done == nil then {} else {
		ui.node({ id = `welcome-mark-{ch.name}`, style = { absolute = true, right = kit.u(6), top = kit.u(6), width = kit.u(22), height = kit.u(22), radius = kit.u(11), align = "center", justify = "center",
			background = if done then c.play else c.raised, border = 2, border_color = c.border } }, {
			kit.text(if done then "✓" else "…", { bold = true, size = kit.u(13), color = if done then c.on_accent else c.muted }),
		}),
	}
	local framed = { picture }
	for _, m in mark do table.insert(framed, m) end
	return ui.column({ id = "welcome-change-" .. ch.name, style = { width = w + 4, gap = kit.u(6), shrink = 0 } }, {
		ui.node({ style = { border = 2, border_color = c.border, radius = kit.u(10), shrink = 0 } }, framed),
		kit.text(name, { bold = true, size = kit.m.small + 1 }),
		caption(ch.line or "", w),
	})
end

-- The approved packages that have landed, by name.
local function landed_names()
	local out = {}
	for _, p in store.get("registry.landed") or {} do out[p.name] = p end
	return out
end

-- The modal's content for where the talk is.
local function content(c, inputs, inner)
	local brief = store.get("editor.ui.brief") or ""
	if brief == "" then
		-- 1. the question, and the box to answer in
		return {
			heading(QUESTION, kit.u(34), { id = "welcome-question" }),
			line("Say it in your own words. Claude finds the packages that make it, and this engine becomes yours.", { tone = "muted", size = kit.m.small + 1 }),
			ui.input({
				id = "welcome-brief", bind = BRIEF, value = inputs[BRIEF] or "",
				placeholder = "A small island to explore, seen over the shoulder of a kid with a satchel…",
				on_submit = kit.route("welcome_build", {}), focus_border = c.accent,
				style = { width = "100%", height = kit.u(112), padding = kit.u(10), radius = kit.m.radius, border = 2, border_color = c.border,
					background = c.background, color = c.text, font = kit.face.text, font_size = kit.m.font_size, text_wrap = "word" },
			}),
			ui.row({ style = { width = "100%", gap = kit.u(12), align = "center", justify = "end", shrink = 0 } }, {
				kit.text("Enter to send", { size = kit.m.small, tone = "muted" }),
				kit.button("Ask Claude", kit.route("welcome_build", {}), { id = "welcome-build", tone = "accent", bold = true, height = kit.u(40), style = { padding_x = kit.u(20) } }),
			}),
		}
	end
	local p = store.get("runtime.proposal") or {}
	local phase = store.get("registry.phase") or ""
	-- (what was asked shows while Claude works it out; the proposal and the install have the room after)
	local items = if p.state == "open" or p.state == "approved" then {} else { asked(brief, c) }
	-- the packages and the editor are pictured; an edit to a document of the project (the game scene Claude puts
	-- together) is not a package: it is said in a line under them, in Claude's words
	local packages, edits = {}, {}
	for _, ch in p.changes or {} do table.insert(if ch.action == "edit" then edits else packages, ch) end
	local function then_line()
		local said = {}
		for _, ch in edits do table.insert(said, string.match(ch.line or "", ":%s*(.+)$") or ch.line or "") end
		if #said == 0 then return nil end
		return kit.text("Then: " .. table.concat(said, "; "), { id = "welcome-then", size = kit.m.small + 1, tone = "muted", wrap = true, style = { width = "100%" } })
	end
	local cols = math.max(2, math.min(#packages, if inner > kit.u(560) then 4 else 2))
	local tw = math.floor((inner - (cols - 1) * kit.u(14)) / cols) - 4
	if proposal.showing(p, { "welcome-install", "welcome-decline" }) then
		-- 3. what Claude proposes, pictured, and Install
		table.insert(items, heading(if p.title and p.title ~= "" then p.title else "Claude proposes", kit.u(26), { id = "welcome-proposal-title" }))
		local tiles = {}
		for _, ch in packages do table.insert(tiles, tile(ch, tw, c, nil)) end
		table.insert(items, ui.row({ id = "welcome-proposal", style = { width = "100%", gap = kit.u(14), wrap = true } }, tiles))
		local line = then_line()
		if line then table.insert(items, line) end
		table.insert(items, ui.row({ style = { width = "100%", gap = kit.u(12), align = "center", justify = "end", shrink = 0 } }, {
			kit.button("Discard", kit.route("proposal_decline", { n = p.n }), { id = "welcome-decline", height = kit.u(40) }),
			kit.button("Install", kit.route("proposal_approve", { n = p.n }), { id = "welcome-install", tone = "confirm", bold = true, height = kit.u(40), style = { padding_x = kit.u(28) } }),
		}))
		return items
	end
	if p.state == "approved" then
		-- 4. installing, each package marked as it lands; then restarting; then ready
		local got = landed_names()
		local done, total = 0, 0
		local tiles = {}
		for _, ch in packages do
			-- a package once it has landed; the recipe once it is the editor's
			local recipe = store.get("editor.ui.recipe") or ""
			local here = if ch.action == "install" then got[ch.name] ~= nil else string.sub(recipe, 1, #ch.name + 1) == ch.name .. " "
			if ch.action == "install" then
				total += 1
				if here then done += 1 end
			end
			table.insert(tiles, tile(ch, tw, c, here))
		end
		local title = if phase == "restarting" then "Restarting…"
			elseif phase == "ready" and done == total then "Your engine is ready"
			else "Installing…"
		-- (the tiles' marks show how far it is: no counter)
		table.insert(items, heading(title, kit.u(26), { id = "welcome-phase" }))
		table.insert(items, ui.row({ id = "welcome-installing", style = { width = "100%", gap = kit.u(14), wrap = true } }, tiles))
	end
	-- 2. (and under 4) Claude's newest short note, and what it is doing now
	local lines = claude_lines()
	if #lines > 0 then table.insert(items, said(lines[#lines], "welcome-claude", c)) end
	local now = store.get("runtime.activity") or {}
	local notes = now.notes or {}
	local doing = notes[#notes]
	-- while the Store installs, what it is doing now (its own status: "Installing cel-shading: fetched …", "Applying
	-- recipe …"), the step before its colon; while the packages take effect, that
	if p.state == "approved" then
		local st = (store.get("registry.status") or {}).text or ""
		local step = string.gsub(string.match(st, "^([^:]+)") or st, "…$", "")
		-- (the Store says ready after each install: the engine opens only once every package has landed)
		local got, all = landed_names(), true
		for _, ch in p.changes or {} do
			if ch.action == "install" and got[ch.name] == nil then all = false end
		end
		if phase == "restarting" then doing = "Restarting the engine with the new packages"
		elseif phase == "ready" and all then doing = "Opening your engine"
		elseif phase == "ready" then doing = "Installing the next package"
		elseif step ~= "" then doing = step end
	end
	table.insert(items, ui.row({ id = "welcome-activity", style = { width = "100%", gap = kit.u(10), align = "center", shrink = 0 } }, {
		ui.node({ style = { width = kit.u(10), height = kit.u(10), radius = kit.u(5), shrink = 0, background = if now.working then c.accent else c.muted } }),
		kit.text(doing or "Claude is reading what you asked", { id = "welcome-note", size = kit.m.small + 1, tone = "muted", wrap = true, style = { grow = 1, width = 0 } }),
	}))
	return items
end

function welcome.render(focus, inputs, time)
	local c = kit.c
	local size = store.get("runtime.size.welcome") or {}
	local w = size.w or 1200
	local mw = math.min(kit.u(MAX_W), w - 2 * kit.u(MARGIN))
	local pad = kit.u(28)
	local modal = ui.column({ id = "welcome-modal", ui = "modal", style = {
		width = mw, padding = pad, gap = kit.u(16), radius = kit.u(16), background = c.panel, border = 2, border_color = c.accent, shrink = 0,
	} }, content(c, inputs, mw - 2 * pad - 4))
	return ui.column({ id = "panel:welcome", ui = "welcome", style = { width = "100%", grow = 1, min_height = 0, background = c.background, clip = true } }, {
		{ kind = "measure", name = "welcome", style = { absolute = true, left = 0, top = 0, width = "100%", height = "100%" } },
		ui.column({ style = { width = "100%", grow = 1, min_height = 0, align = "center", justify = "center", scroll = true, padding_y = kit.u(MARGIN) } }, { modal }),
	})
end

return welcome
