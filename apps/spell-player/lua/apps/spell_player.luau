local nodes = require("nodes")
local hud = require("spell_player.hud")
local inventory = require("inventory_grid")
local revision = require("spell_player.revision")
local app = { name = "Spell player", version = "1", framed = false, routes = {}, nodes = {} }
function app.render(self, env) return nodes.render(app, self, env) end
function app.routes.spell_player_command(self, r) hud.command(r) end
function app.routes.spell_player_toggle(self, r) hud.toggle(r) end
function app.routes.spell_player_inventory(self, r) inventory.request(r) end
function app.routes.spell_player_sources(self, r) revision.toggle(r) end
function app.routes.spell_player_revision(self, r) revision.request(r) end
function app.unmount() native_spell_player.release_view() end
return app
