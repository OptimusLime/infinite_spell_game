-- scene-3d's scene node types. Their data (fields, kinds, defaults, docs) is Rust (src/nodes.rs); here is how
-- they draw: a World is a scene_3d paint node carrying its props, its environment at this moment, every 3D node
-- under it (instanced scenes included) and the Camera it names, filling its box. The nodes under a World draw nothing by themselves.
local nodes = require("nodes")

local ITEMS = { "TimeOfDay", "Sky", "Palette", "Look", "Grade", "PixelArt", "Sea", "Terrain", "Plant", "Rock", "Hut", "Dock", "Boat", "Material", "Swatch", "Character", "Campfire", "Grown", "Fire", "Explosion", "ParticleEffect", "Bomb", "Arrow", "Sprite", "Walker", "Tiles", "Meadow", "Street", "Plaza", "House", "Well", "Lantern", "TownSeed", "Weather", "Creatures", "SpellFx", "Founding", "CreatureModel" }
local IS_ITEM = {}
for _, name in ITEMS do IS_ITEM[name] = true end

-- Each type's field names, from its schema.
local fields = {}
local function copy(p, name)
	local list = fields[name]
	if not list then
		list = {}
		local s = data.schema(name)
		for _, f in if s then s.fields else {} do table.insert(list, f.name) end
		fields[name] = list
	end
	local t = {}
	for _, k in list do t[k] = p[k] end
	return t
end

-- Every 3D node under `path`, depth first, and the Cameras by name.
local function gather(ctx, path, items, cameras)
	for _, child in store.get(ctx.base, path, "children") or {} do
		local kind = store.get(ctx.base, child, "type")
		local p = data.object(ctx.base, child, "props")
		if p and kind == "Camera" then
			cameras[store.get(ctx.base, child, "name")] = copy(p, "Camera")
		elseif p and IS_ITEM[kind] then
			local t = copy(p, kind)
			t.type = kind
			-- a Character is found by its node's name (third-person drives the one it is given)
			if kind == "Character" and (t.name == nil or t.name == "") then t.name = store.get(ctx.base, child, "name") end
			table.insert(items, t)
		end
		gather(ctx, child, items, cameras)
	end
end

-- What the environment is made from (world.rs environment_of): only these items are handed to it.
local SETTING = { TimeOfDay = true, Sky = true, Palette = true, Look = true, Grade = true }

-- The World's items and cameras as a render unit: gathered again only when a node under the World changes, so a
-- playing frame (only the time moves) reuses the same tables, and their JSON (bind.rs tree_json) is made once.
-- (the unit's output is the items list itself, the table the node carries, so its JSON is the one kept; its cameras
-- and setting are kept beside it, made when it renders)
local beside = {}
local function gathered(ctx, path)
	local key = `scene_3d.items:{ctx.base}:{path}`
	local items = unit(key, function()
		local items, cameras = {}, {}
		gather(ctx, path, items, cameras)
		local setting = {}
		for _, t in items do if SETTING[t.type] then table.insert(setting, t) end end
		-- (a host that keeps a World's items is handed them here, once each time they change)
		local kept = if scene_3d.keep_items then { key, scene_3d.keep_items(key, items) } else nil
		beside[key] = { cameras = cameras, setting = setting, kept = kept }
		return items
	end)
	local b = beside[key] or { cameras = {}, setting = {} }
	return { items = items, cameras = b.cameras, setting = b.setting, kept = b.kept }
end

nodes.types.World = data.schema("World")
-- (the nodes under a World are its items: it gathers them itself, they are not built one by one)
if nodes.gathers then nodes.gathers.World = true end
nodes.builders.World = function(ctx, path, p)
	local g = gathered(ctx, path)
	local camera = g.cameras[p.camera]
	-- the environment at this moment, derived once here: the World draws with it, and it is written to the store
	-- (runtime.environment.<world>, outside any scene file) where a clock, a HUD or a slide reads the same hour
	local environment = scene_3d.environment(g.setting, ctx.env.time or 0, camera)
	store.set(`runtime.environment.{path}`, environment)
	environment.type = "Environment"
	-- the scene's `pixel`: drawn as pixel art, that many of its pixels to one
	local scene = ctx.env and ctx.env.scene
	local pixel = scene and scene.id and store.get(`scene.{scene.id}.pixel`) or nil
	-- (the items as gathered, the same table every frame they do not change; the environment beside them. A host built
	-- before the paint node took it there reads only `items`: appended to a copy for it)
	local items = g.items
	if g.kept then
		-- (the host has them: the node names them and carries none)
		items = nil
	elseif not scene_3d.environment_beside then
		items = table.clone(g.items)
		table.insert(items, environment)
	end
	return {
		kind = "scene_3d", id = path, world = copy(p, "World"), items = items, kept = g.kept, environment = environment, camera = camera, pixel = pixel,
		style = { width = "100%", grow = 1, min_height = 240 },
	}
end

-- WorldSwitch: "changing screens", the live switch between two Worlds under it, the first (the cosy pixel town) and the
-- second (the frozen ink side): both drawn, the second over the first in a box that grows down from the top as the
-- dusk falls (a wipe with an ink line on its edge). `side`: pixel or ink shown, or auto: ink while the twin moons are
-- aligned (the first World's environment, runtime.environment.<world>); a change plays over `seconds`. In Play its
-- key flips it (one key press: the other side, until pressed again); in the editor, set `side`.
local ui = require("ui")
local switch_state = {}

nodes.register("WorldSwitch", {
	doc = "Changing screens: the live switch between its two Worlds (first: the pixel town, second: the ink side), a wipe falling from the top; auto: ink while the moons are aligned; its key flips it in Play",
	side = { kind = "one_of", options = { "auto", "pixel", "ink" }, default = "auto", doc = "Which side shows: auto (ink while the twin moons are aligned), pixel or ink" },
	seconds = { kind = "number", min = 0, max = 10, default = 1.6, doc = "How long the wipe takes" },
	key = { kind = "text", default = "i", doc = "In Play, the key that flips to the other side" },
	ink = { kind = "color", default = "#140f1f", doc = "The ink line on the wipe's edge" },
	show = { kind = "text", doc = "Shown only when the app's value of this name is true" },
}, function(ctx, path, p, kids)
	local st = switch_state[path] or { shown = nil, from = 0, at = 0, flipped = false, was = false }
	switch_state[path] = st
	local now = (ctx.env and ctx.env.time) or 0
	-- which side it wants: the prop, the moons (auto), and in Play the key's flip on top
	local first = store.get(ctx.base, path, "children")
	local world = first and first[1]
	local env = world and store.get("runtime.environment", world) or {}
	local aligned = (env.aligned or 0) > 0.5 or (env.moons_apart or 180) < 8
	-- (how soon, in seconds, the moons cross the line the switch flips at, 18 degrees apart: from how fast their
	-- distance changes; nil while it does not move toward the line)
	local apart = env.moons_apart
	local eta = nil
	if apart and st.apart and st.apart_t and now > st.apart_t then
		local rate = (apart - st.apart) / (now - st.apart_t)
		if math.abs(rate) > 1e-6 then
			local t = (18 - apart) / rate
			if t > 0 then eta = t end
		end
	end
	if apart and (st.apart_t == nil or now - st.apart_t > 0.25) then st.apart, st.apart_t = apart, now end
	local want = if p.side == "ink" then 1 elseif p.side == "pixel" then 0 else (if aligned then 1 else 0)
	local key = string.lower(p.key or "i")
	local down = false
	for _, k in store.get("editor.input.held") or {} do
		if string.lower(k) == key then down = true end
	end
	if store.get("editor.run.playing") then
		if down and not st.was then st.flipped = not st.flipped end
	else
		st.flipped = false
	end
	st.was = down
	if st.flipped then want = 1 - want end
	-- the wipe: from where it was to where it is wanted, eased, over `seconds`
	local seconds = math.max(p.seconds or 1.6, 0.001)
	local function reveal_at(t)
		if st.shown == nil then return st.target or want end
		local f = math.clamp((t - st.at) / seconds, 0, 1)
		f = f * f * (3 - 2 * f)
		return st.from + ((st.target or want) - st.from) * f
	end
	-- (in the editor a still scene draws only when something changes: there a change shows at once; in Play it wipes)
	if not store.get("editor.run.playing") then
		st.shown, st.target, st.from = nil, want, want
	end
	if st.target ~= want then
		st.outgoing = if reveal_at(now) > 0.5 then "ink" else "pixel"
		st.hold = st.last or now
		st.from = if st.shown == nil then want else reveal_at(now)
		st.at = now
		st.target = want
		st.shown = true
	end
	local reveal = reveal_at(now)
	local pixel, ink = kids[1], kids[2]
	local fill = { absolute = true, left = 0, top = 0, right = 0, bottom = 0 }
	local layers = {}
	-- the hidden side's GPU state warmed before it is needed (a cold first draw is what a wipe frame costs): in Play,
	-- once at the start and again a few seconds before the moons cross the line, drawn under the side that shows, first
	-- small (its meshes, materials and light probes made), then half a second later at full size (its targets)
	local hidden = if reveal >= 0.999 then "pixel" elseif reveal <= 0.001 then "ink" else nil
	if store.get("editor.run.playing") and hidden then
		local soon = eta ~= nil and eta < 4
		if st.warmed ~= hidden or (soon and not st.warmed_soon) then
			-- (the first warm-up a second and a half into Play, clear of Play's own start)
			st.warm_side, st.warm_step, st.warm_t = hidden, 0, if st.warmed == nil and not soon then now + 1.5 else now
			st.warmed, st.warmed_soon = hidden, soon or st.warmed_soon
		end
		if st.warm_side == hidden and st.warm_step < 2 and now >= st.warm_t then
			local node = if hidden == "ink" then ink else pixel
			if node then
				local small = st.warm_step == 0
				local copy = table.clone(node)
				copy.style = ui.merge(node.style, if small then { absolute = true, left = 0, top = 0, width = 160, height = 90, min_height = 0 } else fill)
				if hidden == "ink" then copy.pixel = nil end
				table.insert(layers, copy)
			end
			st.warm_step += 1
			st.warm_t = now + 0.5
		end
	else
		st.warmed, st.warmed_soon = nil, false
	end
	if eta == nil or eta > 6 then st.warmed_soon = false end
	-- (only what shows is drawn: the town under a full ink side is not)
	if pixel and reveal < 0.999 then
		pixel.style = ui.merge(pixel.style, fill)
		table.insert(layers, pixel)
	end
	-- (the ink side is never pixel art, whatever the scene's `pixel` says for the town)
	if ink then ink.pixel = nil end
	-- (while the wipe runs both sides show: the side going out holds the last frame it was drawn at, so its picture is
	-- reused as it is, and only the side coming in is drawn: a wipe costs one World a frame, its first frame too)
	if reveal > 0.001 and reveal < 0.999 then
		if st.outgoing == "ink" and ink then ink.time = st.hold end
		if st.outgoing == "pixel" and pixel then pixel.time = st.hold end
	end
	st.last = now
	if ink and reveal > 0.001 then
		-- the ink World at the switch's full size, inside a box `reveal` of its height from the top (clipped)
		ink.style = ui.merge(ink.style, { absolute = true, left = 0, top = 0, right = 0, height = `{100 / reveal}%`, min_height = 0 })
		table.insert(layers, ui.node({ id = "ink-side", style = { absolute = true, left = 0, top = 0, right = 0, height = `{reveal * 100}%`, clip = true } }, { ink }))
		if reveal < 0.999 then
			table.insert(layers, ui.node({ id = "dusk-line", style = { absolute = true, left = 0, right = 0, top = `{reveal * 100}%`, height = 3, background = p.ink or "#140f1f" } }))
		end
	end
	return ui.node({ id = nodes.last(path), ui_state = { side = if reveal > 0.5 then "ink" else "pixel", reveal = reveal }, style = { width = "100%", grow = 1, min_height = 240 } }, layers)
end)

for _, name in { "Camera", table.unpack(ITEMS) } do
	nodes.types[name] = data.schema(name)
	nodes.builders[name] = function() return nil end
end

return nodes.types.World
