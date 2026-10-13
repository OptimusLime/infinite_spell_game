local nodes = require("nodes")
local hud = require("spell_player.hud")
nodes.types.SpellPlayer = data.schema("SpellPlayer")
-- Source modules can reload while an older native host preserves finite state.
local has_visuals = false
if native_spell_player and type(native_spell_player.frame_visuals) == "function" then
    for _, field in nodes.types.SpellPlayer.fields do
        if field.name == "visuals" then has_visuals = true end
    end
end
nodes.builders.SpellPlayer = function(ctx, path, props)
    local parent = string.match(path, "^(.*)/[^/]+$")
    local world = if parent then parent .. "/" .. props.world else props.world
    local ok, ref, warning
    if has_visuals then
        ok, ref, warning = pcall(native_spell_player.frame_visuals, props.config, props.session, world, path, props.visuals)
    else
        ok, ref = pcall(native_spell_player.frame, props.config, props.session, world)
    end
    if not ok then return hud.error(tostring(ref)) end
    local width, height = ctx.env.width or 1100, ctx.env.height or 720
    local layout_ref
    if type(native_spell_player.layout) == "function" then
        local d = data.object(ref)
        local panes = if d then data.object(d.ui) else nil
        local pane = if panes and panes.equipment_open then "equipment" elseif panes and panes.inventory_open then "inventory" elseif panes and panes.sources_open then "sources" elseif panes and panes.capacity_open then "capacity" else "none"
        local issue = d and (d.error ~= "" or d.source_error ~= "" or (warning and warning ~= "")) or false
        layout_ref = native_spell_player.layout(path, world, width, height, pane, issue)
    end
    local tree = hud.render(ref, width, height, warning, layout_ref)
    -- Presentation references only; the app binds the exact sibling World after both nodes are built.
    if layout_ref then tree.player_layout = layout_ref end
    return tree
end
return nodes.types.SpellPlayer
