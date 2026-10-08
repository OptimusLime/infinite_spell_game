-- Flat Rust records own all readiness, energy, equipment and spell state. These tables are UI nodes only.
local ui = require("ui")
local inventory = require("inventory_grid")
local revision = require("spell_player.revision")
local M = {}
local P = { font_size = 20, text = "#ecf1fa", muted = "#b2bed1", accent = "#a78df7", border = "#465064",
    panel = "#131c2dee", button = "#28374d", item = "#35485d", selected = "#355d73", cell = "#202630", success = "#77cb97", danger = "#ff9585" }
local function object(ref) return if ref and ref ~= "" then data.object(ref) else nil end
local function text(value, size, color)
    return ui.text(value, {style = {font_size = size or 20, color = color or P.text, text_wrap = "word", width = "100%", shrink = 0}})
end
local function button(label, id, route, args, selected)
    return ui.button({ id = id, on_press = ui.action("route", {name = route,args = args}), style = {
        background = if selected then "#59458a" else P.button, border = 1, border_color = if selected then P.accent else P.border,
        padding_x = 12, padding_y = 8, radius = 6, shrink = 0, min_height = 38 } }, {text(label, 20)})
end
local function cmd(label,id,d,command,extra,selected)
    local args = extra or {};args.request,args.command,args.expected_token = d.request,command,d.token
    for _, ref in d.keys do
        local key = object(ref)
        if key.command == command and key.item_id == (args.item_id or "") and key.spell_id == (args.spell_id or "") and key.target_id == (args.target_id or "") then
            label = label .. " [" .. key.key .. "]"
        end
    end
    return button(label,id,"spell_player_command",args,selected)
end
function M.command(r)
    local request = object(r.request);if not request then return end
    request.command = r.command
    request.item_id, request.spell_id, request.target_id = r.item_id or "",r.spell_id or "",r.target_id or ""
    request.expected_token = r.expected_token
    request.sequence = request.sequence + 1
end
function M.toggle(r)
    local state = object(r.ui);if not state then return end
    if r.pane == "inventory" then state.inventory_open = not state.inventory_open
    elseif r.pane == "equipment" then state.equipment_open = not state.equipment_open end
end
local function inline(value, size, color)
    return ui.text(value,{style={font_size=size or 18,color=color or P.text,text_wrap="none",shrink=0}})
end
local function row(children)
    return ui.row({style = {width = "100%",gap = 8,wrap = true,align = "center",shrink = 0}},children)
end
function M.error(message)
    return ui.column({style = {crisp = true,absolute = true,left = 18,right = 18,bottom = 18,padding = 16,background = P.panel}}, {text("Spell player",24),text(message,18,P.danger)})
end
function M.render(ref,width,height)
    local d = object(ref);if not d then return M.error("Reading player session…") end
    local supply, panes = object(d.supply),object(d.ui)
    if not supply or not panes then return M.error("Reading player bindings…") end
    local loaded,wand = object(supply.loaded),object(supply.wand)
    local spells, targets = {},{}
    for _, path in d.spells do local s=object(path);table.insert(spells,cmd(s.title .. " · " .. s.allocation,"spell-select:"..s.id,d,"select",{spell_id=s.id},s.selected)) end
    for _, path in d.targets do local t=object(path);table.insert(targets,cmd("Aim "..t.id.." · "..string.format("%.0f",t.health).." HP","spell-aim:"..t.id,d,"aim",{target_id=t.id},d.aimed_target==t.id)) end
    table.insert(targets,cmd("Fire selected spell","spell-fire",d,"fire",nil,true))
    local status = if d.error ~= "" then d.error elseif not supply.can_cast then supply.reason elseif not d.has_aim then "Choose an aim target." else d.message
    local bottom = {
        row({inline(string.format("Cartridge energy  %.0f total",supply.total),24,"#97d7e7"),
            inline((if loaded then string.format("Loaded %s: %.0f / %.0f",loaded.item_id,loaded.remaining,loaded.capacity) else "No cartridge loaded") .. "   ·   " .. (if wand then "Equipped: "..wand.definition else "No wand equipped"),17,P.muted)}),
        row(spells),row(targets),
        text(status,16,if d.error~="" then P.danger else P.muted),
    }
    local toggles = {
        cmd(if panes.equipment_open then "Close equipment" else "Equipment / cartridges","spell-equipment-toggle",d,"toggle_equipment"),
        cmd(if panes.inventory_open then "Close Inventory" else "Inventory","spell-inventory-toggle",d,"toggle_inventory"),
    }
    if revision.available() then
        table.insert(toggles,button(if panes.sources_open then "Close sources" else "Authored sources","spell-sources-toggle","spell_player_sources",{ui=d.ui}))
    end
    local header = {text(d.title,24),row(toggles)}
    if d.source_error~="" then table.insert(header,text(if revision.available() then "Edited sources pending — open Authored sources. Current finite state is preserved." else d.source_error,16,P.danger)) end
    local pane_top = if d.source_error~="" then 132 else 108
    local pane_height = math.max(160,height-pane_top-226)
    local overlays = {
        ui.column({style={absolute=true,left=16,right=16,top=14,gap=8,background=P.panel,radius=8}},header),
        ui.column({id="spell-player-bottom",style={absolute=true,left=16,right=16,bottom=14,padding=14,gap=8,background=P.panel,radius=10}},bottom),
    }
    if panes.equipment_open then
        local details={text("Equipment",23)}
        local eq=object(d.equipment)
        for _, path in eq.slots do local s=object(path);table.insert(details,text(s.title..": "..(if s.occupied then s.item_id else "empty"),18,P.muted)) end
        local equip={}
        for _, path in d.wands do local item=object(path);table.insert(equip,cmd("Equip "..item.definition,"spell-equip:"..item.id,d,"equip",{item_id=item.id})) end
        table.insert(details,row(equip))
        table.insert(details,text("Physical cartridges",21))
        for _, path in supply.cartridges do local c=object(path);table.insert(details,cmd((if c.loaded then "Loaded " else "Load / replace ")..c.item_id..string.format(" · %.0f / %.0f",c.remaining,c.capacity),"spell-load:"..c.item_id,d,"load",{item_id=c.item_id},c.loaded)) end
        table.insert(details,cmd("Unload to Inventory","spell-unload",d,"unload"))
        table.insert(overlays,ui.column({id="spell-equipment",style={absolute=true,left=16,top=pane_top,width=math.min(460,width-32),height=pane_height,scroll=true,clip=true,background=P.panel,padding=14,gap=9,radius=8}},details))
    end
    if panes.inventory_open then
        local w=math.min(700,width-32)
        table.insert(overlays,ui.column({id="spell-inventory",style={absolute=true,right=16,top=pane_top,width=w,height=pane_height,scroll=true,clip=true,background=P.panel,padding=14,radius=8}}, {
            inventory.render(d.inventory,d.inventory_interaction,d.inventory_request,"spell_player_inventory",w-28,P),
        }))
    end
    if revision.available() and panes.sources_open then
        table.insert(overlays,revision.render(d,width,pane_top,pane_height,P))
    end
    return ui.node({id="spell-player-hud",style={crisp=true,absolute=true,left=0,right=0,top=0,bottom=0}},overlays)
end
return M
