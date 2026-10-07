-- The Store: find packages by describing what you want, see what each does (its capture, its proof), install it.
-- Like every editor panel it holds no state: it reads registry.* from the store and writes requests there
-- (registry.request = {op, query | name, kind, n}); the host runs `app store-request` for each new n, which
-- answers into registry.status / registry.results / registry.log. Script packages install live; native ones
-- rebuild the host and restart it.
local ui = require("ui")
local kit = require("kit")

local store_panel = { routes = {} }
local QUERY = "store:query"
local WIDTH = 460

local function request(fields)
	local n = (store.get("registry.request.n") or 0) + 1
	fields.n = n
	store.set("registry.request", fields)
end

function store_panel.routes.store_toggle(self, r, ctx)
	store.set("registry.open", not store.get("registry.open"))
end

function store_panel.routes.store_search(self, r, ctx)
	local query = tostring(r[QUERY] or store.get("registry.query") or "")
	if query == "" then return end
	ctx.emit("input.blur", {})
	store.set("registry.query", query)
	request({ op = "search", query = query })
end

function store_panel.routes.store_install(self, r, ctx)
	request({ op = "install", name = r.name, kind = r.kind })
end

function store_panel.routes.store_apply(self, r, ctx)
	request({ op = "apply", name = r.name })
end

function store_panel.open()
	return store.get("registry.open") == true
end

-- The toolbar's Store button (in the toolbar, left of the Claude button).
function store_panel.toggle()
	return kit.button("Store", kit.route("store_toggle"), { id = "store-toggle", active = store_panel.open(), height = kit.u(30) })
end

local function pill(text, tone)
	return kit.pill(text, if tone then kit.c[tone] else nil)
end

local function card(r)
	local c, m = kit.c, kit.m
	local busy = (store.get("registry.status") or {}).busy
	local heading = {
		kit.text(r.name, { bold = true, size = m.font_size + 1 }),
		pill(r.kind),
		pill(if r.native then "native" else "script", if r.native then "danger" else "play"),
	}
	-- a capture that stands in until the package's real look lands
	if r.capture_stub then table.insert(heading, pill("stand-in look")) end
	table.insert(heading, kit.spacer())
	table.insert(heading, kit.text(string.format("%.2f", r.score or 0), { mono = true, size = m.small, tone = "muted" }))
	local title = ui.row({ style = { width = "100%", gap = kit.u(6), align = "center" } }, heading)
	local info = { title, kit.text(r.description or "", { wrap = true, size = m.small + 1, style = { width = "100%" } }) }
	local adds = {}
	for _, a in r.adds or {} do if a ~= r.name then table.insert(adds, a) end end
	if #adds > 0 then
		table.insert(info, kit.text("also installs " .. table.concat(adds, ", "), { size = m.small, tone = "muted", wrap = true, style = { width = "100%" } }))
	elseif r.requires and #r.requires > 0 then
		table.insert(info, kit.text("needs " .. table.concat(r.requires, ", ") .. " (installed)", { size = m.small, tone = "muted", wrap = true, style = { width = "100%" } }))
	end
	if r.kind == "recipe" then
		local lineage = {}
		if r.parent then table.insert(lineage, "fork of " .. r.parent) end
		if r.accepted and #r.accepted > 0 then table.insert(lineage, "accepted " .. table.concat(r.accepted, ", ")) end
		if (r.proposals or 0) > 0 then table.insert(lineage, `{r.proposals} open suggestion{if r.proposals == 1 then "" else "s"}`) end
		if #lineage > 0 then
			table.insert(info, kit.text(table.concat(lineage, " · "), { id = `store-lineage-{r.name}`, size = m.small, color = c.accent, wrap = true, style = { width = "100%" } }))
		end
	end
	local action
	if r.kind == "recipe" then
		action = if r.applied then kit.pill("Applied", c.play) else kit.button("Apply recipe", kit.route("store_apply", { name = r.name }), {
			id = `store-apply-{r.name}`, tone = "confirm", height = kit.u(24), size = m.small + 1, bold = true, disabled = busy,
		})
	elseif r.installed then
		action = kit.pill("Installed", c.play)
	else
		action = kit.button(if r.rebuild then "Install · rebuilds host" else "Install · live", kit.route("store_install", { name = r.name, kind = r.kind }), {
			id = `store-install-{r.kind}-{r.name}`, tone = "confirm", height = kit.u(24), size = m.small + 1, bold = true, disabled = busy,
		})
	end
	table.insert(info, ui.row({ style = { width = "100%", gap = kit.u(8), align = "center", padding_top = 2 } }, {
		action,
		kit.spacer(),
		kit.text(if r.video then "proof: " .. r.video else "no proof video", { size = m.small, tone = "muted" }),
	}))
	-- the package's capture (else its proof still), at its own aspect inside a 120 x 76 frame
	local w, h = 120, 76
	if r.still_size then
		local sw, sh = r.still_size[1], r.still_size[2]
		if sw / sh > 120 / 76 then h = math.floor(120 * sh / sw) else w = math.floor(76 * sw / sh) end
	end
	local proof = if r.still
		then ui.node({ style = { width = kit.u(120), height = kit.u(76), radius = kit.u(4), shrink = 0, background = c.background, align = "center", justify = "center", border = 1, border_color = c.border } }, {
			ui.image({ texture = r.still, style = { width = w, height = h } }),
		})
		else ui.node({ style = { width = kit.u(120), height = kit.u(76), radius = kit.u(4), shrink = 0, background = c.background, align = "center", justify = "center", border = 1, border_color = c.border } }, {
			kit.text("no proof", { size = m.small, tone = "muted" }),
		})
	return ui.row({ id = `store-result-{r.kind}-{r.name}`, style = { width = "100%", gap = kit.u(10), padding = kit.u(10), radius = m.radius + 2, background = c.raised, shrink = 0, align = "start" } }, {
		proof,
		ui.column({ style = { grow = 1, width = 0, gap = kit.u(5) } }, info),
	})
end

function store_panel.panel(style, focus, inputs)
	local c, m = kit.c, kit.m
	local editing = focus == QUERY
	local query = store.get("registry.query") or ""
	local status = store.get("registry.status") or {}
	local search = ui.row({ style = { width = "100%", gap = kit.u(8), shrink = 0 } }, {
		ui.input({
			id = "store-query", bind = QUERY, value = if editing then (inputs[QUERY] or "") else query,
			placeholder = "Describe what you want…",
			on_submit = kit.route("store_search", {}), focus_border = c.accent,
			style = { grow = 1, width = 0, height = kit.u(30), padding = kit.u(7), border = 1, radius = m.radius, border_color = c.border, background = c.background, color = c.text, font_size = m.small + 2, text_wrap = "none" },
		}),
		kit.button("Search", kit.route("store_search", {}), { id = "store-search", tone = "accent", height = kit.u(30), bold = true }),
	})
	local rows = {}
	for _, r in store.get("registry.results") or {} do table.insert(rows, card(r)) end
	if #rows == 0 then
		table.insert(rows, kit.text("Packages are published to a registry (an index and a blob store), not copied from a folder. Search by what you want them to do.", { wrap = true, tone = "muted", size = m.small + 1, style = { width = "100%" } }))
	end
	local log = {}
	for _, l in store.get("registry.log") or {} do table.insert(log, kit.text(l, { mono = true, size = m.small, tone = "muted", wrap = true, style = { width = "100%" } })) end
	local body = ui.column({ style = { width = "100%", grow = 1, min_height = 0, padding = kit.u(12), gap = kit.u(10) } }, {
		search,
		kit.text(status.text or "", { id = "store-status", bold = status.busy == true, size = m.small + 1, color = if status.busy then c.accent else c.muted, wrap = true, style = { width = "100%" } }),
		ui.column({ id = "store-results", style = { width = "100%", grow = 1, min_height = 0, gap = kit.u(8), scroll = true } }, rows),
		ui.column({ style = { width = "100%", gap = 2, shrink = 0 } }, log),
	})
	return kit.panel("store", "Store", {
		kit.text(store.get("registry.mode") or "", { tone = "muted", size = m.small }),
		kit.spacer(),
		kit.icon_button("×", kit.route("store_toggle"), { id = "store-close", height = kit.u(26), size = m.small + 1, tip = "Close the store", tip_below = true }),
	}, body, ui.merge({ width = WIDTH, height = "100%", shrink = 0 }, style))
end

store_panel.WIDTH = WIDTH
return store_panel
