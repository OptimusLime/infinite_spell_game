local nodes = require("nodes")
local hud = require("spell_player.hud")
local capacity = require("spell_player.capacity")
local inventory = require("inventory_grid")
local revision = require("spell_player.revision")
local app = { name = "Spell player", version = "1", framed = false, routes = {}, nodes = {} }
local function visit(tree, f)
    f(tree)
    for _, child in tree.children or {} do visit(child, f) end
end
-- The wire tree contains presentation only. Geometry remains in native typed view records.
local function reserve_field(tree)
    local layouts, worlds = {}, {}
    visit(tree, function(node)
        if node.player_layout then table.insert(layouts, data.object(node.player_layout)) end
        if node.kind == "scene_3d" then table.insert(worlds, node) end
    end)
    if #layouts == 0 then return tree end -- older native producer: unchanged legacy HUD
    local layout = layouts[1]
    local matches = {}
    for _, world in worlds do if world.id == layout.world_id then table.insert(matches, world) end end
    if #layouts ~= 1 or #matches ~= 1 then
        table.insert(tree.children,hud.layout_error("Battlefield layout unavailable: expected one SpellPlayer and one matching World. Controls remain available; check the sibling World ID.",layout.battlefield))
        return tree
    end
    local r = data.object(layout.battlefield)
    local style = matches[1].style
    style.absolute, style.left, style.top = true, r.x, r.y
    style.width, style.height = r.width, r.height
    style.right, style.bottom, style.grow, style.min_height = nil, nil, 0, 0
    style.clip = true
    return tree
end
function app.render(self, env)
    if native_spell_player.begin_view then native_spell_player.begin_view() end
    local tree = nodes.render(app, self, env)
    if native_spell_player.end_view then native_spell_player.end_view() end
    return reserve_field(tree)
end
function app.routes.spell_player_capacity_activate(self, r) capacity.activate(r) end
function app.routes.spell_player_command(self, r) hud.command(r) end
function app.routes.spell_player_toggle(self, r) hud.toggle(r) end
function app.routes.spell_player_inventory(self, r) inventory.request(r) end
function app.routes.spell_player_sources(self, r) revision.toggle(r) end
function app.routes.spell_player_revision(self, r) revision.request(r) end
function app.unmount() native_spell_player.release_view() end
return app
