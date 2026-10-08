local nodes = require("nodes")
local hud = require("spell_player.hud")
nodes.types.SpellPlayer = data.schema("SpellPlayer")
nodes.builders.SpellPlayer = function(ctx, path, props)
    local parent = string.match(path, "^(.*)/[^/]+$")
    local world = if parent then parent .. "/" .. props.world else props.world
    local ok, ref = pcall(native_spell_player.frame, props.config, props.session, world)
    if not ok then return hud.error(tostring(ref)) end
    return hud.render(ref, ctx.env.width or 1100, ctx.env.height or 720)
end
return nodes.types.SpellPlayer
