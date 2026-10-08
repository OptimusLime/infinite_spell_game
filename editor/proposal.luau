-- The proposal: what Claude wants to change, before anything changes (runtime.proposal,
-- atelico_control::conversation), shown in the Claude card (chat.luau), grouped by what each change is: an edit by
-- what it does, a package by its picture and name. The person answers with one click: Apply changes (Install for packages) or Discard, which throws
-- the proposal away (runtime.proposal.state = "declined": nothing changes).
local ui = require("ui")
local kit = require("kit")

local proposal = { routes = {} }

local function answer(state)
	return function(self, r, ctx)
		local p = store.get("runtime.proposal") or {}
		if p.n == r.n and p.state == "open" then store.set("runtime.proposal.state", state) end
	end
end
proposal.routes.proposal_approve = answer("approved")
proposal.routes.proposal_decline = answer("declined")

-- Whether the proposal shows: while it waits for an answer, and for the answer's flash after (the pressed button
-- flashes where it was pressed before the card goes).
function proposal.showing(p, ids)
	if p.state == "open" then return true end
	if p.state ~= "approved" and p.state ~= "declined" then return false end
	for _, id in ids or { "proposal-approve", "proposal-decline" } do
		if kit.flashing(id) then return true end
	end
	return false
end

-- The editor in a recipe's look, small: a toolbar, a side panel, the scene in the middle, a heading in its face.
local function workspace(ws, w, h)
	local hh = math.floor(h * 0.18)
	return ui.node({ style = { width = w, height = h, background = ws.background, radius = math.min(ws.radius or 4, 10), clip = true, shrink = 0 } }, {
		ui.row({ style = { absolute = true, left = 0, top = 0, width = w, height = hh, background = ws.header, align = "center", padding_x = kit.u(8), gap = kit.u(6) } }, {
			ui.node({ style = { width = hh * 0.5, height = hh * 0.5, radius = hh * 0.25, background = ws.accent } }),
			ui.text("Aa", { style = { font = ws.display, font_size = hh * 0.62, color = ws.text, text_wrap = "none", shrink = 0 } }),
		}),
		ui.node({ style = { absolute = true, left = kit.u(6), top = hh + 6, width = math.floor(w * 0.24), height = h - hh - 12, radius = math.min(ws.radius or 4, 6), background = ws.panel } }),
		ui.node({ style = { absolute = true, left = math.floor(w * 0.24) + 12, top = hh + 6, right = kit.u(6), height = h - hh - 12, radius = math.min(ws.radius or 4, 6), background = ws.panel, border = 2, border_color = ws.accent } }),
	})
end

proposal.workspace = workspace

-- One change as a picture with a caption under it: the package's capture large, or the workspace it makes; one line.
local TILE_W, TILE_H = 232, 131
local function change(ch, c)
	-- a picture where there is one (a package's, the edited scene's preview); an edit without one is its line alone,
	-- never an empty box
	local picture = if ch.workspace then workspace(ch.workspace, TILE_W, TILE_H)
		elseif ch.image and ch.image ~= "" then ui.image({ texture = ch.image, style = { width = TILE_W, height = TILE_H, radius = kit.u(8) } })
		else nil
	-- a package by its name, a recipe as the editor it makes; an edit by the document's own title only where the
	-- project has one (a document Claude is about to make has none yet: never its file name)
	local titled = (store.get("project.titles") or {})[ch.name]
	local name = if ch.action == "apply" then "Your editor" elseif ch.action == "edit" then titled else ch.name
	local kids = {}
	if picture then table.insert(kids, ui.node({ style = { border = 2, border_color = c.border, radius = kit.u(10), shrink = 0 } }, { picture })) end
	-- the name on one line, whole: as large as fits the tile (a long package name comes down a little)
	if name and name ~= "" then
		table.insert(kids, kit.text(name, { bold = true, size = math.min(kit.u(19), math.floor(kit.u(19) * TILE_W / math.max(1, (select(2, typeset.lines(kit.face.bold, kit.u(19), 1e6, name)) or 0) + 2))), style = { width = TILE_W } }))
	end
	-- (an edit's line is what it does: it reads large when it stands alone)
	table.insert(kids, kit.text(ch.line, { size = if picture then kit.u(15) else kit.m.font_size, tone = if picture then "muted" else "text", wrap = true, style = { width = TILE_W } }))
	return ui.column({ id = "proposal-change-" .. ch.name, style = { width = TILE_W + 4, gap = kit.u(8), shrink = 0 } }, kids)
end

-- The proposal (it takes its container's width): nil unless one waits for an answer. Two parts: its changes, groups
-- one under another, each a heading over its changes (they scroll when there are more than fit), and its answer,
-- Discard and Apply changes (or Install), to be pinned under them.
function proposal.card()
	local p = store.get("runtime.proposal") or {}
	if not proposal.showing(p) then return nil end
	local c = kit.c
	local body = kit.m.font_size
	local groups, order, edits = {}, {}, false
	for _, ch in p.changes or {} do
		local g = ch.group or ""
		if not groups[g] then groups[g] = {} table.insert(order, g) end
		table.insert(groups[g], ch)
		if ch.action == "edit" then edits = true end
	end
	local rows = {
		kit.text(if p.title and p.title ~= "" then p.title else "These changes", { bold = true, size = body, wrap = true, style = { width = "100%", shrink = 0 } }),
	}
	for _, g in order do
		local kids = {}
		if g ~= "" then table.insert(kids, kit.text(g, { id = "proposal-group-" .. g, bold = true, size = body, color = c.accent, style = { shrink = 0 } })) end
		for _, ch in groups[g] do
			-- a package or a recipe by its picture and name; an edit by what it does
			if ch.action ~= "edit" and (ch.image or "") ~= "" or ch.workspace then
				table.insert(kids, change(ch, c))
			else
				table.insert(kids, kit.text(ch.line, { size = body, wrap = true, style = { width = "100%", shrink = 0 } }))
			end
		end
		table.insert(rows, ui.column({ id = "proposal-group:" .. g, style = { width = "100%", gap = kit.u(4), shrink = 0 } }, kids))
	end
	-- the changes scroll when there are more than fit; the answer stays pinned under them, always in view
	local answer = ui.row({ id = "proposal-answer", style = { width = "100%", gap = kit.u(12), align = "center", justify = "end", padding_top = kit.u(8), shrink = 0, border_top = 1, border_color = c.border } }, {
		kit.button("Discard", kit.route("proposal_decline", { n = p.n }), { id = "proposal-decline", height = kit.line(body) + 12, size = body }),
		-- a change to the game is applied; packages are installed (both the confirm green)
		kit.button(if edits then "Apply changes" else "Install", kit.route("proposal_approve", { n = p.n }), { id = "proposal-approve", tone = "confirm", bold = true, height = kit.line(body) + 12, size = body, style = { padding_x = kit.u(20) } }),
	})
	return ui.column({ id = "proposal-card", ui = "proposal", style = {
		width = "100%", shrink = 1, min_height = 0, gap = kit.u(10), scroll = true, padding_bottom = kit.u(6),
	} }, rows), answer
end

return proposal
