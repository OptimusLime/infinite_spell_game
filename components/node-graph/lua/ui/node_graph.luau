-- ui.node_graph: a node-graph canvas over a graph in the store (graph.<name>, file graphs/<name>.graph.yaml):
-- typed nodes from a catalog (graph_types.<name>), custom per-node renderers, cables routed at right angles and
-- bundled where one output feeds several inputs, collapsible groups (folded to one node listing the outputs wired
-- out of it), pan and zoom, and a layout that stacks its blocks when the frame is tall. It holds no graph state: everything it shows is read from the store and every edit is a
-- store.set on the graph, so a script, MCP, the Inspector and this canvas all edit the same thing.
-- The same code draws the editor's shader graph and a game's skill tree; the caller's `look` and `renderers`
-- are the only difference.
--
--   local ng = require("ui.node_graph")
--   local st = ng.new("graph.fire", "fire_pointer", { catalog = "graph_types.shader", selection = "editor.ui.graph_node" })
--   routes.fire_pointer = function(self, r) ng.pointer(st, r) end
--   ng.view(st, { width, height, measure, look, renderers, run, fit, top, bottom, open })   -- a ui node; fit:
--       laid out for its frame and fitted, between `top` and `bottom`; open: the one group shown open, the others
--       folded; frame: node ids the view is fitted to
--   ng.look(theme.editor.graph, { name, ui, node_width })               -- a look from a GraphTheme
--
-- Without a width and height the canvas fills its container; `measure = <name>` has the host report that box's
-- size (runtime.size.<name>), so the fitted view fits the panel it is in.
-- look = {name, ui, colors = {canvas, node, header, text, muted, border, selected, wire, group, danger, ok},
--   types = {<port type> = color}, families = {<category> = header color}, fonts = {title, body}, radius, outline,
--   node_width, plain, head, row (a node's header and port row heights), title, label (their text sizes)}
-- renderers[type.renderer] = {height = n | fn(ctx), draw = fn(ctx) -> ui node}; ctx = {id, node, type, params,
--   status, zoom, w, inner (the body's content width: what it draws must fit it and h), h, look, graph, outputs}. renderers.group, the same for a collapsed group's body (ctx = {id, group,
--   members, zoom, w, h, look, graph, graph_name}): a group folded to one node still shows what it makes.
-- run = graph.run(...)'s result: each node's status is shown on its header.
local ui = require("ui")

local ng = { VERSION = "0.1.0" }
local HEAD, ROW, PAD, PORT = 28, 22, 8, 10 -- HEAD and ROW: the look's (`head`, `row`), set by each view

-- `path`: the graph's store path, or the graph itself (a table the caller owns; the canvas reads it each frame).
function ng.new(path, route, opts)
	opts = opts or {}
	if type(path) == "table" then
		return { data = path, path = "", name = opts.name or "graph", route = route, catalog = opts.catalog, play = opts.play, on_tap = opts.on_tap }
	end
	-- opts.play: a game's view (fitted, not editable; opts.on_tap(id) on a tapped node)
	return { path = path, name = string.match(path, "[^%.]+$"), route = route, catalog = opts.catalog, selection = opts.selection, play = opts.play, on_tap = opts.on_tap, geo = nil }
end

local function types_of(cat)
	local out = {}
	for _, t in (cat and cat.types) or {} do out[t.name] = t end
	return out
end

local function category_color(cat, name, fallback)
	for _, c in (cat and cat.categories) or {} do
		if c.name == name then return c.color end
	end
	return fallback
end

local function split(port)
	local n, p = string.match(port, "^(.-):(.+)$")
	return n, p
end

-- Every group a node sits in, innermost last; groups may hold groups.
local function parents(groups)
	local parent = {}
	for gid, g in groups do
		for _, m in g.members or {} do parent[m] = gid end
	end
	return parent
end

-- Whether a group is collapsed: it is unless its graph says `collapsed: false` (a graph opens folded to its groups'
-- wired outputs, R-7; opening one is a click, which writes the flag).
local function folded(grp)
	return grp ~= nil and grp.collapsed ~= false
end
ng.folded = folded

-- The outermost collapsed group a node or group is inside, if any.
local function folded_into(parent, groups, id)
	local out, at, seen = nil, parent[id], { [id] = true }
	-- (a group named like a node, or groups holding each other, would otherwise walk up forever)
	while at and not seen[at] do
		seen[at] = true
		if folded(groups[at]) then out = at end
		at = parent[at]
	end
	return out
end

local function body_height(r, ctx)
	if not r then return 0 end
	if type(r.height) == "function" then return r.height(ctx) end
	return r.height or 0
end

-- `#rrggbb` → relative luminance (0 dark .. 1 light).
local function luminance(hex)
	local h = string.gsub(hex or "", "#", "")
	local function c(i)
		local v = (tonumber(string.sub(h, i, i + 1), 16) or 0) / 255
		return if v <= 0.03928 then v / 12.92 else ((v + 0.055) / 1.055) ^ 2.4
	end
	return 0.2126 * c(1) + 0.7152 * c(3) + 0.0722 * c(5)
end

-- A look from a GraphTheme (editor/theme.toml [graph]): sharp corners, ink outlines, family headers, ink cables.
function ng.look(t, o)
	t, o = t or {}, o or {}
	return {
		name = o.name or "graph", ui = o.ui or "graph", radius = t.radius or 2, outline = t.outline or 2, node_width = o.node_width or 190,
		colors = {
			canvas = t.canvas, node = t.node, header = t.header, text = t.ink, muted = t.muted, border = t.ink, selected = t.selected,
			wire = t.ink, group = t.group, danger = t.danger, ok = t.ok, grid = t.grid, grid_major = t.grid_major,
		},
		grid = { cell = t.cell or 24, major = t.major or 5 },
		families = t.families or {}, types = t.types or {}, fonts = o.fonts or {},
	}
end

-- A family's header color, and the text that reads on it.
local function header_of(look, cat, category)
	local c = look.colors
	local bg = (look.families and look.families[category]) or (if look.families then c.header else category_color(cat, category, c.header))
	-- the text on it: the light one of canvas and ink on a dark header, the dark one on a light header (a dark
	-- look's ink is light)
	local a, b = c.canvas or c.node, c.text
	local light, dark = a, b
	if a and b and luminance(a) < luminance(b) then light, dark = b, a end
	return bg, if luminance(bg) < 0.18 then light else dark
end

-- Graph-space geometry of every visible node, collapsed group box, open group frame and port anchor. `aspect`: the
-- frame's width over its height when the graph is laid out for its frame instead of as the file places it: inside
-- each block (a top-level group, or the loose nodes together) the columns wrap into rows, as many to a row as fill
-- the frame best, and the blocks, left to right in the file, run side by side in a wide frame and stack in a tall
-- one. `frame`: node ids the arrangement shows largest.
function ng.layout(g, cat, renderers, look, aspect, frame)
	local tall = aspect
	local types, groups = types_of(cat), g.groups or {}
	local parent = parents(groups)
	-- anchors by `node:port`, inputs and outputs apart (a node may have an input and an output of one name)
	local geo = { nodes = {}, boxes = {}, frames = {}, ins = {}, outs = {} }
	for id, n in g.nodes or {} do
		local t = types[n.type] or { title = n.type, inputs = {}, outputs = {}, params = {} }
		local w = t.width or look.node_width or 180
		-- (inner: inside the padding and the widest border a node is drawn with, a selected one's)
		local ctx = { id = id, node = n, type = t, params = n.params or {}, look = look, graph = g, zoom = 1, w = w, inner = w - 2 * PAD - 3 * math.max(1, look.outline or 1) }
		local r = t.renderer and renderers[t.renderer]
		-- a plain look (a game's) draws no port rows: wires meet the node's sides at mid height
		local rows = if look.plain then 0 else math.max(#(t.inputs or {}), #(t.outputs or {}))
		local bh = body_height(r, ctx)
		local h = HEAD + rows * ROW + (if bh > 0 then bh + PAD else 0) + PAD
		geo.nodes[id] = { id = id, x = n.x or 0, y = n.y or 0, w = w, h = h, t = t, body = bh, renderer = r, rows = rows, into = folded_into(parent, groups, id) }
	end
	-- the outermost group each node sits in: its block
	local function top(id)
		local at, seen, out = parent[id], {}, nil
		while at and not seen[at] do
			seen[at] = true
			out = at
			at = parent[at]
		end
		return out
	end
	-- a collapsed group lists only the outputs wired out of it
	local function outs_of(gid)
		local list, seen = {}, {}
		for to, from in g.edges or {} do
			local a, b = geo.nodes[split(from)], geo.nodes[split(to)]
			if a and b and a.into == gid and b.into ~= gid and not seen[from] then
				seen[from] = true
				table.insert(list, from)
			end
		end
		table.sort(list, function(x, y)
			local nx, ny = geo.nodes[split(x)], geo.nodes[split(y)]
			if nx.y ~= ny.y then return nx.y < ny.y end
			return x < y
		end)
		return list
	end
	if tall then
		local blocks, order = {}, {}
		for id, item in geo.nodes do
			local key = top(id) or ""
			local blk = blocks[key]
			if not blk then
				blk = { key = key, members = {}, folded = folded(groups[key]) }
				blocks[key] = blk
				table.insert(order, blk)
			end
			item.x0, item.y0 = item.x, item.y
			table.insert(blk.members, item)
		end
		local function extent(blk)
			local x0, y0, x1, y1
			for _, n in blk.members do
				x0 = math.min(x0 or n.x, n.x); y0 = math.min(y0 or n.y, n.y)
				x1 = math.max(x1 or n.x + n.w, n.x + n.w); y1 = math.max(y1 or n.y + n.h, n.y + n.h)
			end
			if groups[blk.key] then x0, y0, x1, y1 = x0 - 16, y0 - 40, x1 + 16, y1 + 16 end
			if blk.folded then x1, y1 = x0 + (look.node_width or 180), y0 + HEAD + math.max(1, #outs_of(blk.key)) * ROW + PAD end
			return x0, y0, x1, y1
		end
		-- each open block's columns (nodes at about one x), left to right
		for _, blk in order do
			blk.cols = {}
			if not blk.folded then
				for _, n in blk.members do
					local c
					for _, col in blk.cols do if math.abs(col.x - n.x0) < 60 then c = col end end
					if not c then c = { x = n.x0, w = 0, nodes = {} }; table.insert(blk.cols, c) end
					c.w = math.max(c.w, n.w)
					table.insert(c.nodes, n)
				end
				table.sort(blk.cols, function(a, b) return a.x < b.x end)
			end
		end
		table.sort(order, function(a, b)
			local ax, bx = math.huge, math.huge
			for _, n in a.members do ax = math.min(ax, n.x0) end
			for _, n in b.members do bx = math.min(bx, n.x0) end
			return ax < bx
		end)
		-- columns wrap `k` to a row, each row under the last; the blocks go `per` to a row in a wide frame (a column
		-- in a tall one), the rows under each other. The width and height it takes.
		local function arrange(k, per)
			for _, blk in order do
				for _, n in blk.members do n.x, n.y = n.x0, n.y0 end
				if #blk.cols > k then
					local _, y0, _, y1 = extent(blk)
					local rise = y1 - y0 + 40
					local lefts, x = {}, blk.cols[1].x
					for slot = 1, k do
						lefts[slot] = x
						local w = 0
						for i = slot, #blk.cols, k do w = math.max(w, blk.cols[i].w) end
						x += w + 60
					end
					for i, col in blk.cols do
						local row, slot = (i - 1) // k, (i - 1) % k + 1
						for _, n in col.nodes do
							n.x = lefts[slot] + (n.x0 - col.x)
							n.y = n.y0 + row * rise
						end
					end
				end
				blk.x0, blk.y0, blk.x1, blk.y1 = extent(blk)
			end
			-- the blocks side by side (a wide frame) or stacked (a tall one), `per` to a line, the lines after
			-- each other
			local wide = aspect >= 1
			local width, height, at = 0, 0, 0
			for first = 1, #order, per do
				local across, along = 0, 0
				for i = first, math.min(first + per - 1, #order) do
					local blk = order[i]
					across = math.max(across, if wide then blk.y1 - blk.y0 else blk.x1 - blk.x0)
				end
				for i = first, math.min(first + per - 1, #order) do
					local blk = order[i]
					local dx, dy
					if wide then
						dx, dy = along - blk.x0, at - blk.y0
						along += blk.x1 - blk.x0 + 60
					else
						dx, dy = at + (across - (blk.x1 - blk.x0)) / 2 - blk.x0, along - blk.y0
						along += blk.y1 - blk.y0 + 60
					end
					for _, n in blk.members do n.x += dx; n.y += dy end
				end
				if wide then
					width, height = math.max(width, along - 60), at + across
				else
					width, height = at + across, math.max(height, along - 60)
				end
				at += across + 60
			end
			return width, height
		end
		-- as many columns to a row as fill the frame best (the largest zoom)
		local most = 1
		for _, blk in order do most = math.max(most, #blk.cols) end
		local best, fit = most, -1
		-- (framed on some nodes: the arrangement that shows those largest)
		local function framed_size()
			local x0, y0, x1, y1
			for _, id in frame or {} do
				local n = geo.nodes[id]
				if n then
					x0 = math.min(x0 or n.x, n.x); y0 = math.min(y0 or n.y, n.y)
					x1 = math.max(x1 or n.x + n.w, n.x + n.w); y1 = math.max(y1 or n.y + n.h, n.y + n.h)
				end
			end
			return x0 and (x1 - x0), x0 and (y1 - y0)
		end
		-- (at least one to a line: an empty graph, no blocks, would make a zero step, a loop that never ends)
		local lines = math.max(1, #order)
		for k = 1, most do
			for per = 1, #order do
				local w, h = arrange(k, per)
				local fw, fh = framed_size()
				if fw then w, h = fw, fh end
				local z = math.min(aspect / w, 1 / h)
				if z > fit then best, lines, fit = k, per, z end
			end
		end
		arrange(best, lines)
	end
	-- group bounds from their members (nodes and groups), innermost first
	local function bounds(gid, seen)
		seen = seen or {}
		if seen[gid] then return nil end
		seen[gid] = true
		local x0, y0, x1, y1
		for _, m in groups[gid].members or {} do
			local b = geo.nodes[m] or (groups[m] and bounds(m, seen))
			if b then
				x0 = math.min(x0 or b.x, b.x); y0 = math.min(y0 or b.y, b.y)
				x1 = math.max(x1 or b.x + b.w, b.x + b.w); y1 = math.max(y1 or b.y + b.h, b.y + b.h)
			end
		end
		if not x0 then return nil end
		return { x = x0 - 16, y = y0 - 40, w = x1 - x0 + 32, h = y1 - y0 + 56 }
	end
	for gid, grp in groups do
		local b = bounds(gid)
		if b then
			local into = folded_into(parent, groups, gid)
			if folded(grp) and not into then
				local outs = outs_of(gid)
				local gr = renderers and renderers.group
				local body = if gr then body_height(gr, { id = gid, group = grp, members = grp.members or {}, look = look, graph = g, zoom = 1 }) else 0
				geo.boxes[gid] = { id = gid, x = b.x, y = b.y, w = look.node_width or 180, h = HEAD + math.max(1, #outs) * ROW + PAD + (if body > 0 then body + PAD else 0), outs = outs, body = body }
			elseif not folded(grp) and not into then
				geo.frames[gid] = ui.merge(b, { id = gid })
			end
		end
	end
	-- laid out for a frame, what shows (collapsed groups' boxes, nodes in no folded group) never lies on another: a
	-- folded group's box is a different shape from the members it stands for (a row of three emitters becomes one
	-- narrow, tall box), so each one overlapping one already placed moves clear of it, below it in a tall frame, to
	-- its right in a wide one
	if aspect then
		local shown = {}
		for _, box in geo.boxes do table.insert(shown, box) end
		for _, item in geo.nodes do
			if not item.into then table.insert(shown, item) end
		end
		table.sort(shown, function(a, b) if a.y ~= b.y then return a.y < b.y end return a.x < b.x end)
		local placed = {}
		for _, it in shown do
			local moved = true
			while moved do
				moved = false
				for _, p in placed do
					if it.x < p.x + p.w + 40 and it.x + it.w + 40 > p.x and it.y < p.y + p.h + 40 and it.y + it.h + 40 > p.y then
						if aspect < 1 then it.y = p.y + p.h + 60 else it.x = p.x + p.w + 60 end
						moved = true
					end
				end
			end
			table.insert(placed, it)
		end
	end
	for id, item in geo.nodes do
		if not item.into then
			local function y_of(i) return if look.plain then item.y + item.h / 2 else item.y + HEAD + (i - 0.5) * ROW end
			for i, p in item.t.inputs or {} do geo.ins[id .. ":" .. p.name] = { x = item.x, y = y_of(i), type = p.type, node = id } end
			for i, p in item.t.outputs or {} do geo.outs[id .. ":" .. p.name] = { x = item.x + item.w, y = y_of(i), type = p.type, node = id } end
		end
	end
	-- wires into a collapsed group meet its box's left edge by its title; its listed outputs leave from their rows
	for id, item in geo.nodes do
		local box = item.into and geo.boxes[item.into]
		if box then
			for _, p in item.t.inputs or {} do geo.ins[id .. ":" .. p.name] = { x = box.x, y = box.y + HEAD / 2, type = p.type, folded = true, node = box.id } end
			for _, p in item.t.outputs or {} do geo.outs[id .. ":" .. p.name] = { x = box.x + box.w, y = box.y + HEAD / 2, type = p.type, folded = true, node = box.id } end
		end
	end
	for _, box in geo.boxes do
		for i, from in box.outs do
			local a = geo.outs[from]
			if a then geo.outs[from] = { x = box.x + box.w, y = box.y + HEAD + (i - 0.5) * ROW, type = a.type, node = box.id } end
		end
	end
	return geo
end

local function screen(st, x, y)
	local v = st.v
	return x * v.zoom + v.x, y * v.zoom + v.y
end

-- The view that fits every node in w x h, between `top` and `bottom` (room left for what sits over the canvas).
function ng.fit(g, geo, w, h, top, bottom)
	top, bottom = top or 0, bottom or 0
	local x0, y0, x1, y1
	local function take(n)
		x0 = math.min(x0 or n.x, n.x); y0 = math.min(y0 or n.y, n.y)
		x1 = math.max(x1 or n.x + n.w, n.x + n.w); y1 = math.max(y1 or n.y + n.h, n.y + n.h)
	end
	-- what is drawn: visible nodes, collapsed group boxes, open group frames
	for _, n in geo.nodes do if not n.into then take(n) end end
	for _, b in geo.boxes do take(b) end
	for _, f in geo.frames do take(f) end
	if not x0 then return { x = 0, y = 0, zoom = 1 } end
	local zoom = math.clamp(math.min((w - 60) / (x1 - x0), (h - top - bottom - 60) / (y1 - y0)), 0.2, 1.5)
	return { x = (w - (x1 - x0) * zoom) / 2 - x0 * zoom, y = top + (h - top - bottom - (y1 - y0) * zoom) / 2 - y0 * zoom, zoom = zoom }
end

-- One line of text; too long for its place, it ends in "…" (never drawn past its node).
local function text(value, size, color, font, extra)
	return ui.text(value, { style = ui.merge({ font_size = size, color = color, font = font, text_wrap = "none", text_overflow = "ellipsis", shrink = 1, min_width = 0, max_width = "100%" }, extra) })
end

local function status_of(run, id)
	local n = run and run.nodes and run.nodes[id]
	return n and n.status, n and n.reason
end

-- (blocked: waiting on an input that has no value yet, a consequence shown quietly; what failed says why, in red)
local STATUS = { ran = "ok", cached = "muted", failed = "danger", blocked = "muted", cycle = "danger", running = "selected" }

-- An outline's width at zoom z: the look's, in whole pixels, never under one (a fractional border leaves a box's
-- content a pixel or two off its children once the layout rounds its edges)
local function line(look, z)
	return math.max(1, math.floor((look.outline or 1) * z + 0.5))
end

local function port_dot(look, z, x, y, color, wired, extra)
	local c = look.colors
	local r = PORT * z
	return ui.node(ui.merge({
		style = {
			-- a port straddles its node's edge: declared an overlay, not held inside the node
			absolute = true, overlay = true, left = x - r / 2, top = y - r / 2, width = r, height = r, radius = if look.plain then r / 2 else math.max(0, (look.radius or 0) * z),
			background = if wired then color else c.node, border = line(look, z), border_color = c.border,
		},
	}, extra or {}))
end

-- A gathering node's input (an effect's e1…e8) is named by what feeds it: the wired node's label; nothing on an
-- empty one.
local function fed_by(st, types, key)
	local from = st.g.edges[key]
	if not from then return nil end
	local n = st.g.nodes[(split(from))]
	if not n then return nil end
	local ty = types[n.type]
	return n.label or (ty and ty.title) or n.type
end

-- What lands in a graph after it was first drawn is shown landing, on the engine's clock (clock.now()): a group or a
-- node fades in and settles (LAND seconds), a wire draws from its source to its target (DRAW seconds,
-- once both ends have landed). What was there when the graph was first drawn is simply there.
local LAND, DRAW = 0.35, 0.3
local arrivals = {} -- graph key → { first = true while its first frame is drawn, at = { id → time it landed } }
local function landed(st, id)
	local a = st.arrivals
	if not a then return 1 end
	if a.at[id] == nil then a.at[id] = if a.first then -math.huge else clock.now() end
	local e = math.clamp((clock.now() - a.at[id]) / (LAND + DRAW), 0, 1)
	if e < 1 then st.landing = true end
	e = math.clamp((clock.now() - a.at[id]) / LAND, 0, 1)
	return e * e * (3 - 2 * e)
end
-- a style landing, `e` of the way: faded in and settling from a little below (5% of its height), its size kept so
-- nothing inside it runs past it
local function landing(style, e)
	if e >= 1 then return style end
	style.top = (style.top or 0) + (style.height or 0) * 0.05 * (1 - e)
	style.opacity = e
	return style
end
-- The first `f` of a polyline's length.
local function drawn_part(points, f)
	if f >= 1 or #points < 2 then return points end
	local total = 0
	for i = 2, #points do total += math.sqrt((points[i].x - points[i - 1].x) ^ 2 + (points[i].y - points[i - 1].y) ^ 2) end
	local left, out = total * f, { points[1] }
	for i = 2, #points do
		local a, b = points[i - 1], points[i]
		local d = math.sqrt((b.x - a.x) ^ 2 + (b.y - a.y) ^ 2)
		if d >= left then
			local t = if d > 0 then left / d else 0
			table.insert(out, { x = a.x + (b.x - a.x) * t, y = a.y + (b.y - a.y) * t })
			return out
		end
		left -= d
		table.insert(out, b)
	end
	return out
end

local function node_view(st, o, item, cat, issues)
	local look, z, c = o.look, st.v.zoom, o.look.colors
	local id, t, n = item.id, item.t, st.g.nodes[item.id]
	local types = types_of(cat)
	local sx, sy = screen(st, item.x, item.y)
	local selected = st.selected == id
	local status, reason = status_of(o.run, id)
	local problem = issues[id]
	local edge = line(look, z) * (if selected or problem then 1.5 else 1)
	local head_bg, head_fg = header_of(look, cat, t.category)
	local fs = (look.label or 14) * z
	local radius = (look.radius or 0) * z
	local rows = {}
	for i = 1, item.rows do
		local ip, op = (t.inputs or {})[i], (t.outputs or {})[i]
		local iname = if not ip then nil elseif t.gathers then fed_by(st, types, id .. ":" .. ip.name) else ip.name
		rows[i] = ui.row({ style = { width = "100%", height = ROW * z, shrink = 0, padding_x = 12 * z, align = "center" } }, {
			if iname then text(iname, fs, c.muted, look.fonts.body) else ui.node({}),
			ui.node({ style = { grow = 1 } }),
			if op then text(op.name, fs, c.muted, look.fonts.body) else ui.node({}),
		})
	end
	local body
	if item.renderer and item.body > 0 then
		local ctx = { id = id, node = n, type = t, params = n.params or {}, status = status, reason = reason, zoom = z, w = item.w * z, inner = item.w * z - 2 * PAD * z - 2 * edge, h = item.body * z, look = look, graph = st.g, graph_name = st.name, catalog = st.cat,
			outputs = o.run and o.run.nodes and o.run.nodes[id] and o.run.nodes[id].outputs }
		local ok, drawn = pcall(item.renderer.draw, ctx)
		body = ui.node({ style = { width = "100%", height = item.body * z, shrink = 0, padding_x = PAD * z, margin_top = PAD * z } }, {
			if ok then drawn else text(tostring(drawn), fs, c.danger, look.fonts.body, { text_wrap = "word" }),
		})
	end
	local ports = {}
	local function port(p, side, i)
		local key = id .. ":" .. p.name
		local wired = if side == "in" then st.g.edges[key] ~= nil else st.fed[key] ~= nil
		local x = if side == "in" then 0 else item.w * z
		table.insert(ports, port_dot(look, z, x, (HEAD + (i - 0.5) * ROW) * z, look.types[p.type] or c.wire, wired,
			{ ui = `ports/{side}/{p.name}`, ui_state = { type = p.type, side = side, wired = wired } }))
	end
	if not look.plain then
		for i, p in t.inputs or {} do port(p, "in", i) end
		for i, p in t.outputs or {} do port(p, "out", i) end
	end
	local badge = status and not look.plain and ui.node({ style = { width = 8 * z, height = 8 * z, radius = 4 * z, shrink = 0, border = math.max(1, z), border_color = head_fg, background = c[STATUS[status] or "muted"] } })
	local children = {
		ui.row({ style = { width = "100%", height = HEAD * z, shrink = 0, padding_x = 10 * z, gap = 6 * z, align = "center", background = head_bg, border_bottom = line(look, z), border_color = c.border } }, {
			-- a label too long for its header ends in "…"
			text(n.label or t.title or n.type, (look.title or 16) * z, head_fg, look.fonts.title),
			ui.node({ style = { grow = 1 } }),
			badge,
		}),
	}
	for _, r in rows do table.insert(children, r) end
	if body then table.insert(children, body) end
	for _, p in ports do table.insert(children, p) end
	if problem or (status == "failed" or status == "cycle") then
		-- the problem hangs under its node: declared an overlay
		table.insert(children, ui.node({ style = { absolute = true, overlay = true, left = 0, top = item.h * z + 4 * z, width = item.w * z * 1.4 } }, {
			text(problem or reason or status, 14 * z, c.danger, look.fonts.body, { text_wrap = "word" }),
		}))
	end
	return ui.column({
		ui = "nodes/" .. id,
		ui_state = { type = n.type, x = n.x, y = n.y, selected = selected, status = status, group = item.into },
		style = landing({
			-- (a selected or failing node's wider border grows its box rather than eating into its rows)
			absolute = true, left = sx, top = sy, width = item.w * z, height = item.h * z + 2 * (edge - line(look, z)), clip = false,
			background = c.node, radius = radius, border = edge,
			border_color = if problem then c.danger elseif selected then c.selected else c.border,
		}, landed(st, item.id or "")),
	}, children)
end

local function toggle(st, look, z, gid, sign, bg, fg)
	local c = look.colors
	return ui.button({
		ui = "toggle", on_press = ui.action("route", { name = st.route, args = { toggle = gid } }),
		style = { width = 22 * z, height = 22 * z, radius = (look.radius or 0) * z, align = "center", justify = "center", background = bg, border = line(look, z), border_color = c.border, shrink = 0 },
	}, { text(sign, 14 * z, fg, look.fonts.title) })
end

local function group_frame(st, o, f)
	local look, z, c = o.look, st.v.zoom, o.look.colors
	local grp = st.g.groups[f.id]
	local sx, sy = screen(st, f.x, f.y)
	return ui.node({
		ui = "groups/" .. f.id, ui_state = { label = grp.label, collapsed = false, members = grp.members },
		style = landing({ absolute = true, left = sx, top = sy, width = f.w * z, height = f.h * z, radius = (look.radius or 0) * z, background = grp.color or c.group, border = line(look, z), border_color = c.border }, landed(st, "group:" .. f.id)),
	}, {
		ui.row({ style = { width = "100%", height = 30 * z, padding_x = 10 * z, gap = 8 * z, align = "center" } }, {
			-- a plain look has no editing chrome: the group is only its label
			if look.plain then ui.node({ style = { hidden = true } }) else toggle(st, look, z, f.id, "–", c.node, c.text),
			text(grp.label or f.id, (look.title or 16) * 1.1 * z, c.text, look.fonts.title),
		}),
	})
end

-- A collapsed group: one compact node, its title, and a row and a port for each output wired out of it.
local function group_box(st, o, b)
	local look, z, c = o.look, st.v.zoom, o.look.colors
	local grp = st.g.groups[b.id]
	local sx, sy = screen(st, b.x, b.y)
	local bg, fg = c.header, c.text
	if look.families then bg, fg = c.text, c.canvas or c.node end
	local rows, ports = {}, {}
	local names = {}
	for _, from in b.outs do local _, p = split(from); names[p] = (names[p] or 0) + 1 end
	for i, from in b.outs do
		local nid, p = split(from)
		local n = st.g.nodes[nid] or {}
		local t = types_of(st.cat)[n.type] or {}
		local name = if names[p] > 1 then `{n.label or t.title or nid} · {p}` else p
		local ptype
		for _, q in t.outputs or {} do if q.name == p then ptype = q.type end end
		table.insert(rows, ui.row({ style = { width = "100%", height = ROW * z, shrink = 0, padding_x = 12 * z, align = "center", justify = "end" } }, {
			text(name, (look.label or 14) * z, c.muted, look.fonts.body),
		}))
		table.insert(ports, port_dot(look, z, b.w * z, (HEAD + (i - 0.5) * ROW) * z, look.types[ptype] or c.wire, true, { ui = `ports/out/{p}` }))
	end
	local children = {
		ui.row({ style = { width = "100%", height = HEAD * z, shrink = 0, padding_x = 10 * z, gap = 8 * z, align = "center", background = bg, border_bottom = line(look, z), border_color = c.border } }, {
			if look.plain then ui.node({ style = { hidden = true } }) else toggle(st, look, z, b.id, "+", c.node, c.text),
			text(grp.label or b.id, (look.title or 16) * z, fg, look.fonts.title),
		}),
	}
	for _, r in rows do table.insert(children, r) end
	local gr = o.renderers and o.renderers.group
	if gr and (b.body or 0) > 0 then
		local ctx = { id = b.id, group = grp, members = grp.members or {}, zoom = z, w = b.w * z, h = b.body * z, look = look, graph = st.g, graph_name = st.name, catalog = st.cat }
		local ok, drawn = pcall(gr.draw, ctx)
		table.insert(children, ui.node({ style = { width = "100%", height = b.body * z, shrink = 0, padding_x = PAD * z, margin_top = PAD * z } }, {
			if ok then drawn else text(tostring(drawn), (look.label or 14) * z, c.danger, look.fonts.body, { text_wrap = "word" }),
		}))
	end
	for _, p in ports do table.insert(children, p) end
	return ui.column({
		ui = "groups/" .. b.id, ui_state = { label = grp.label, collapsed = true, members = grp.members, outputs = b.outs },
		style = landing({
			absolute = true, left = sx, top = sy, width = b.w * z, height = b.h * z, radius = (look.radius or 0) * z,
			background = c.node, border = line(look, z) * (if st.selected == "group:" .. b.id then 1.5 else 1), border_color = if st.selected == "group:" .. b.id then c.selected else c.border,
		}, landed(st, "group:" .. b.id)),
	}, children)
end

-- Every cable as a polyline at right angles (screen space). A cable runs out of its output, turns once at a column
-- between its ends and runs into its input; the cables of one output share that column (a bundle, split with a
-- dot where a branch leaves it); a cable that runs backwards goes round below both nodes.
local function route(st, geo, edges, look)
	local z = st.v.zoom
	local gap = 24 * z
	local by_source = {}
	for to, from in edges do
		local a, b = geo.outs[from], geo.ins[to]
		if a and b and not (a.folded and b.folded and a.node == b.node) and a.node ~= b.node then
			by_source[from] = by_source[from] or {}
			table.insert(by_source[from], to)
		end
	end
	-- a node's (or a collapsed group's) top and bottom, graph space
	local function span(key)
		local n = geo.nodes[key] or geo.boxes[key]
		if n then return n.y, n.y + n.h end
		return nil
	end
	local wires, dots = {}, {}
	for from, targets in by_source do
		local a = geo.outs[from]
		local ax, ay = screen(st, a.x, a.y)
		table.sort(targets, function(x, y) return geo.ins[x].y < geo.ins[y].y end)
		local forward, xs = {}, {}
		for _, to in targets do
			local b = geo.ins[to]
			local bx = screen(st, b.x, b.y)
			if bx - ax >= 2 * gap then table.insert(forward, to); table.insert(xs, bx) end
		end
		-- the bundle's column: halfway to the nearest input, never closer to either end than `gap`
		local col
		if #forward > 0 then
			local nearest = math.huge
			for _, x in xs do nearest = math.min(nearest, x) end
			col = if #forward > 1 then ax + math.max(gap, math.min((nearest - ax) / 2, 3 * gap)) else (ax + nearest) / 2
		end
		local lo, hi = ay, ay
		for _, to in targets do
			local b = geo.ins[to]
			local bx, by = screen(st, b.x, b.y)
			local hot = st.selected and (split(from) == st.selected or split(to) == st.selected)
			local pts
			if bx - ax >= 2 * gap then
				pts = { { x = ax, y = ay }, { x = col, y = ay }, { x = col, y = by }, { x = bx, y = by } }
				lo, hi = math.min(lo, by), math.max(hi, by)
			elseif bx - ax > 4 then
				-- only just forward: one turn halfway
				local mx = (ax + bx) / 2
				pts = { { x = ax, y = ay }, { x = mx, y = ay }, { x = mx, y = by }, { x = bx, y = by } }
			else
				-- backwards: out, along the gap between the two nodes (or under both), and in
				local at, ab = span(a.node)
				local bt, bb = span(b.node)
				local lane
				if at and bt and ab + 16 < bt then lane = (ab + bt) / 2
				elseif at and bt and bb + 16 < at then lane = (bb + at) / 2
				else lane = math.max(ab or b.y, bb or b.y) + 20 end
				local _, ly = screen(st, 0, lane)
				pts = { { x = ax, y = ay }, { x = ax + gap, y = ay }, { x = ax + gap, y = ly }, { x = bx - gap, y = ly }, { x = bx - gap, y = by }, { x = bx, y = by } }
			end
			table.insert(wires, { points = pts, from = pts[1], to = pts[#pts], color = if hot then look.colors.selected else look.colors.wire, width = (look.outline or 2) * (if hot then 1.5 else 1), key = to, source = from })
		end
		-- a split: where a branch leaves a bundle that runs on
		if #forward > 1 then
			for _, to in forward do
				local _, by = screen(st, 0, geo.ins[to].y)
				if (by > lo and by < hi) or by == ay then table.insert(dots, { x = col, y = by }) end
			end
			if ay > lo and ay < hi then table.insert(dots, { x = col, y = ay }) end
		end
	end
	return wires, dots
end

-- The point halfway along a polyline.
local function halfway(pts)
	local total = 0
	for i = 2, #pts do total += math.abs(pts[i].x - pts[i - 1].x) + math.abs(pts[i].y - pts[i - 1].y) end
	local left = total / 2
	for i = 2, #pts do
		local d = math.abs(pts[i].x - pts[i - 1].x) + math.abs(pts[i].y - pts[i - 1].y)
		if d >= left and d > 0 then
			local k = left / d
			return pts[i - 1].x + (pts[i].x - pts[i - 1].x) * k, pts[i - 1].y + (pts[i].y - pts[i - 1].y) * k
		end
		left -= d
	end
	return pts[1].x, pts[1].y
end

function ng.view(st, o)
	local look, c = o.look, o.look.colors
	-- a graph in the store (edited through it), or one the caller holds (a game's own data)
	local g = st.data or store.get(st.path) or {}
	g.nodes, g.edges, g.groups = g.nodes or {}, g.edges or {}, g.groups or {}
	local cat = if type(st.catalog) == "string" then store.get(st.catalog) else st.catalog
	st.g, st.cat = g, cat
	local box = (o.measure and store.get("runtime.size." .. o.measure)) or st.size or {}
	local w, h = o.width or box.w or 900, o.height or box.h or 520
	st.selected = st.selection and store.get(st.selection)
	-- node sizes and text from the look (a slide's is larger than the editor's)
	HEAD, ROW = look.head or 28, look.row or 22
	-- one group open (`o.open`): every other top-level group is folded to its outputs, in this view only
	local laid = g
	if o.open and o.open ~= "" and g.groups[o.open] then
		local inner = parents(g.groups)
		laid = table.clone(g)
		laid.groups = {}
		for gid, grp in g.groups do
			laid.groups[gid] = if inner[gid] then grp else ui.merge(grp, { collapsed = gid ~= o.open })
		end
	end
	local tall = h > w * 1.1
	-- the editor keeps the file's layout while it is wide; a tall frame and a fitted view are laid out for their frame
	local geo = ng.layout(laid, cat, o.renderers or {}, look, if tall or o.fit then w / h else nil, o.frame)
	st.geo = geo
	-- the graph's saved view is the editor's (in its wide layout); a game's, a slide's and a tall one are fitted, to
	-- the nodes `o.frame` names when it names some
	local framed
	for _, id in o.frame or {} do
		local n = geo.nodes[id]
		if n and not n.into then
			framed = framed or { nodes = {}, boxes = {}, frames = {} }
			table.insert(framed.nodes, n)
		end
	end
	st.v = if g.view and g.view.zoom and not st.play and not o.fit and not tall then g.view else ng.fit(g, framed or geo, w, h, o.top, o.bottom)
	st.g = laid
	st.fed = {}
	for to, from in g.edges do st.fed[from] = to end
	local issues = {}
	if cat and graph then
		local ok, list = pcall(graph.check, g, cat)
		for _, i in (if ok then list else {}) do issues[i.at] = issues[i.at] or i.message end
	end
	st.issues = issues

	-- what lands after the graph's first frame is shown landing (see `landed`)
	local key = o.measure or o.id or "graph"
	local a = arrivals[key]
	if not a then
		a = { first = true, at = {} }
		arrivals[key] = a
	end
	st.arrivals = a
	local children = {}
	for _, f in geo.frames do table.insert(children, group_frame(st, o, f)) end
	-- cables, then a handle on each one's midpoint (its address; drag it away to disconnect)
	local wires, dots = route(st, geo, g.edges, look)
	-- a wire that lands draws from its source to its target, once the ends it joins have landed
	for _, wire in wires do
		if wire.points and st.arrivals then
			local key = "wire:" .. (wire.key or "")
			local e = landed(st, key)
			local at = st.arrivals.at[key]
			if at and at > -math.huge then
				local f = math.clamp((clock.now() - at - LAND * 0.5) / DRAW, 0, 1)
				wire.points = drawn_part(wire.points, f)
			end
			local _ = e
		end
	end
	local handles = {}
	st.mids = {}
	for _, wire in wires do
		local mx, my = halfway(wire.points)
		local r = 4
		st.mids[wire.key] = { x = mx, y = my }
		table.insert(handles, ui.node({
			ui = "wires/" .. wire.key, ui_state = { from = wire.source, to = wire.key, type = (geo.outs[wire.source] or {}).type },
			style = { absolute = true, left = mx - r, top = my - r, width = 2 * r, height = 2 * r, radius = r, background = "#00000000" },
		}))
	end
	local d = st.drag
	if d and d.kind == "wire" and d.at then
		local a = geo.outs[d.from]
		if a then
			local ax, ay = screen(st, a.x, a.y)
			local mx = (ax + d.at.x) / 2
			table.insert(wires, { points = { { x = ax, y = ay }, { x = mx, y = ay }, { x = mx, y = d.at.y }, { x = d.at.x, y = d.at.y } }, color = if d.bad then c.danger else c.selected, width = look.outline or 2, dash = true })
		end
	end
	if o.measure then
		table.insert(children, 1, { kind = "measure", name = o.measure, style = { absolute = true, left = 0, top = 0, width = "100%", height = "100%" } })
	end
	-- the grid under everything, in graph space: it pans and zooms with the nodes
	if c.grid and look.grid then
		table.insert(children, 1, { kind = "graph_grid", x = st.v.x, y = st.v.y, zoom = st.v.zoom, cell = look.grid.cell, major = look.grid.major, minor_color = c.grid, major_color = c.grid_major or c.grid,
			style = { absolute = true, left = 0, top = 0, width = "100%", height = "100%" } })
	end
	table.insert(children, { kind = "graph_wires", zoom = st.v.zoom, wires = wires, dots = dots, corner = 8, style = { absolute = true, left = 0, top = 0, width = "100%", height = "100%" } })
	for _, item in geo.nodes do
		if not item.into then table.insert(children, node_view(st, o, item, cat, issues)) end
	end
	for _, b in geo.boxes do table.insert(children, group_box(st, o, b)) end
	for _, hd in handles do table.insert(children, hd) end
	local note = st.note and os.clock() < st.note.until_ and st.note.text
	if not o.fit then
		table.insert(children, ui.row({ style = { absolute = true, left = 10, bottom = 8, gap = 10, align = "center" } }, {
			text(`{math.floor(st.v.zoom * 100 + 0.5)}%`, 11, c.muted, look.fonts.body),
			if note then text(note, 11, c.danger, look.fonts.body) else ui.node({}),
		}))
	end
	-- the first frame drawn; while anything is still landing, the next frame is asked for (runtime.graph_landing)
	a.first = false
	if st.landing then
		st.landing = false
		store.set("runtime.graph_landing", clock.now())
	end
	return ui.node({
		ui = look.ui or "graph",
		ui_state = { graph = st.path, zoom = st.v.zoom, x = st.v.x, y = st.v.y, tall = tall },
		component = { source = "node-graph/lua/ui/node_graph.luau", version = ng.VERSION, look = look.name, bind = if st.data then "own data: " .. st.name else st.path },
		on_pointer = ui.action("route", { name = st.route }),
		-- given a size, that size; else the space its container leaves it
		-- zoom: its content is drawn at the view's zoom, as a map is; its text is as large as the look sets it
		style = { width = o.width or "100%", height = o.height, grow = if o.height then 0 else 1, shrink = if o.height then 0 else 1, min_height = 0, clip = true, background = c.canvas, zoom = st.v.zoom },
	}, children)
end

-- Canvas-local pointer position.
local function local_at(st, r)
	if r.rect then st.origin = { x = r.rect[1], y = r.rect[2] }; st.size = { w = r.rect[3], h = r.rect[4] } end
	return r.px - st.origin.x, r.py - st.origin.y
end

local function graph_at(st, x, y)
	return (x - st.v.x) / st.v.zoom, (y - st.v.y) / st.v.zoom
end

-- What is under canvas point (x, y), topmost first: a port, a wire handle, a collapsed group, a node, a group frame.
local function hit(st, x, y)
	local z = st.v.zoom
	local best, side, bd = nil, nil, (PORT * z) ^ 2 * 2.5
	for s, anchors in { ["in"] = st.geo.ins, out = st.geo.outs } do
		for key, a in anchors do
			if not a.folded then
				local ax, ay = screen(st, a.x, a.y)
				local dd = (ax - x) ^ 2 + (ay - y) ^ 2
				if dd < bd then best, side, bd = key, s, dd end
			end
		end
	end
	if best then return { kind = "port", key = best, side = side } end
	for to, m in st.mids or {} do
		if (m.x - x) ^ 2 + (m.y - y) ^ 2 < 81 then return { kind = "handle", key = to } end
	end
	local gx, gy = graph_at(st, x, y)
	local inside = function(b) return gx >= b.x and gx <= b.x + b.w and gy >= b.y and gy <= b.y + b.h end
	for gid, b in st.geo.boxes do if inside(b) then return { kind = "group", id = gid } end end
	local top
	for id, n in st.geo.nodes do if not n.into and inside(n) then top = id end end
	if top then return { kind = "node", id = top } end
	for gid, f in st.geo.frames do
		if inside(f) and gy <= f.y + 30 then return { kind = "group", id = gid } end
	end
	return { kind = "canvas" }
end

-- Every node a group holds, through nested groups.
local function members(g, gid, out)
	out = out or {}
	for _, m in (g.groups[gid] and g.groups[gid].members) or {} do
		if g.nodes[m] then table.insert(out, m) elseif g.groups[m] then members(g, m, out) end
	end
	return out
end

function ng.set(st, rel, value)
	store.set(st.path .. "." .. rel, value)
end

function ng.connect(st, from, to)
	local ok, why = graph.can_connect(st.g, st.cat, from, to)
	if ok then ng.set(st, "edges." .. to, from) else st.note = { text = why, until_ = os.clock() + 3 } end
	return ok, why
end

local function select(st, id)
	if st.selection then store.set(st.selection, id) end
end

-- The canvas's on_pointer route (and its group toggles, which pass `toggle`).
function ng.pointer(st, r)
	if not st.geo then return end
	if r.toggle then
		local grp = st.g.groups[r.toggle]
		if grp then ng.set(st, "groups." .. r.toggle .. ".collapsed", not folded(grp)) end
		return
	end
	local x, y = local_at(st, r)
	local v = st.v
	-- play mode (a game's view of the graph): no editing, a tap on a node is the only action
	if st.play then
		if r.phase == "down" then
			local h = hit(st, x, y)
			st.drag = if h.kind == "node" then { kind = "tap", id = h.id, x0 = x, y0 = y } else nil
		elseif r.phase == "up" and st.drag then
			local d = st.drag
			st.drag = nil
			if math.abs(x - d.x0) + math.abs(y - d.y0) < 6 and st.on_tap then st.on_tap(d.id) end
		end
		return
	end
	if r.phase == "wheel" then
		local zoom = math.clamp(v.zoom * math.exp(-(r.dy or 0) * 0.002), 0.15, 3)
		local gx, gy = graph_at(st, x, y)
		ng.set(st, "view", { x = x - gx * zoom, y = y - gy * zoom, zoom = zoom })
	elseif r.phase == "down" then
		local h = hit(st, x, y)
		st.drag = { kind = h.kind, hit = h, x0 = x, y0 = y, v0 = { x = v.x, y = v.y, zoom = v.zoom } }
		if h.kind == "port" and h.side == "out" then
			st.drag = { kind = "wire", from = h.key, at = { x = x, y = y } }
		elseif h.kind == "port" and st.g.edges[h.key] then
			-- pick the wire up off this input, keeping its source end
			local from = st.g.edges[h.key]
			ng.set(st, "edges." .. h.key, nil)
			st.drag = { kind = "wire", from = from, at = { x = x, y = y } }
		elseif h.kind == "node" then
			select(st, h.id)
			local n = st.g.nodes[h.id]
			st.drag.start = { [h.id] = { x = n.x or 0, y = n.y or 0 } }
		elseif h.kind == "group" then
			select(st, "group:" .. h.id)
			st.drag.start = {}
			for _, m in members(st.g, h.id) do
				local n = st.g.nodes[m]
				st.drag.start[m] = { x = n.x or 0, y = n.y or 0 }
			end
		end
	elseif r.phase == "move" and st.drag then
		local d = st.drag
		if d.kind == "wire" then
			d.at = { x = x, y = y }
			local h = hit(st, x, y)
			d.bad = h.kind == "port" and h.side == "in" and not graph.can_connect(st.g, st.cat, d.from, h.key)
			-- no store write: redraw so the loose end follows
			store.set("runtime.graph_drag", { from = d.from, x = x, y = y })
		elseif d.start then
			local dx, dy = (x - d.x0) / v.zoom, (y - d.y0) / v.zoom
			for id, p in d.start do
				ng.set(st, "nodes." .. id .. ".x", math.floor(p.x + dx + 0.5))
				ng.set(st, "nodes." .. id .. ".y", math.floor(p.y + dy + 0.5))
			end
		elseif d.kind == "canvas" then
			ng.set(st, "view", { x = d.v0.x + x - d.x0, y = d.v0.y + y - d.y0, zoom = d.v0.zoom })
		end
	elseif r.phase == "up" and st.drag then
		local d = st.drag
		st.drag = nil
		if d.kind == "wire" then
			local h = hit(st, x, y)
			if h.kind == "port" and h.side == "in" then ng.connect(st, d.from, h.key) end
			store.set("runtime.graph_drag", nil)
		elseif d.kind == "handle" then
			if math.abs(x - d.x0) + math.abs(y - d.y0) > 30 then ng.set(st, "edges." .. d.hit.key, nil) end
		elseif d.kind == "canvas" and math.abs(x - d.x0) + math.abs(y - d.y0) < 3 then
			select(st, nil)
		end
	end
end

return ng
