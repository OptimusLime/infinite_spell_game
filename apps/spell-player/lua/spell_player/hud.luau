-- Flat Rust records own all readiness, energy, equipment and spell state. These tables are UI nodes only.
local ui = require("ui")
local inventory = require("inventory_grid")
local revision = require("spell_player.revision")
local capacity = require("spell_player.capacity")
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
    elseif r.pane == "equipment" then state.equipment_open = not state.equipment_open
    elseif r.pane == "capacity" then
        state.capacity_open = not state.capacity_open
        state.inventory_open, state.equipment_open, state.sources_open = false, false, false
    end
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
function M.layout_error(message,field_ref)
    local r = object(field_ref)
    return ui.column({id="spell-layout-error",style={crisp=true,absolute=true,left=if r then r.x+8 else 8,top=if r then r.y+8 else 8,width=if r then math.max(0,r.width-16) else nil,right=if r then nil else 8,height=if r then math.min(132,math.max(0,r.height-16)) else 86,padding=12,scroll=true,clip=true,background=P.panel}}, {text(message,18,P.danger)})
end
local function placed(node, r)
    local s = node.style
    s.absolute, s.left, s.top, s.width, s.height = true, r.x, r.y, r.width, r.height
    s.right, s.bottom = nil, nil
    return node
end
local function compact_button(node)
    node.style.padding_y, node.style.padding_x, node.style.min_height = 5, 10, 36
    for _, child in node.children or {} do child.style.font_size = 18 end
    return node
end
function M.render(ref,width,height,visual_warning,layout_ref)
    local d = object(ref);if not d then return M.error("Reading player session…") end
    local supply, panes = object(d.supply),object(d.ui)
    if not supply or not panes then return M.error("Reading player bindings…") end
    local layout = object(layout_ref)
    local loaded,wand = object(supply.loaded),object(supply.wand)
    local spells, targets = {},{}
    for _, path in d.spells do local s=object(path);table.insert(spells,cmd(s.title .. " · " .. s.allocation,"spell-select:"..s.id,d,"select",{spell_id=s.id},s.selected)) end
    for _, path in d.targets do local t=object(path);table.insert(targets,cmd("Aim "..t.id.." · "..string.format("%.0f",t.health).." HP","spell-aim:"..t.id,d,"aim",{target_id=t.id},d.aimed_target==t.id)) end
    table.insert(targets,cmd(if layout then "Fire" else "Fire selected spell","spell-fire",d,"fire",nil,true))
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
    if capacity.available() and d.capacity ~= "" then
        table.insert(toggles,button(if panes.capacity_open then "Close capacity" else "Capacity fixture","spell-capacity-toggle","spell_player_toggle",{ui=d.ui,pane="capacity"}))
    end
    if revision.available() then
        table.insert(toggles,button(if panes.sources_open then "Close sources" else "Authored sources","spell-sources-toggle","spell_player_sources",{ui=d.ui}))
    end
    local header = {text(d.title,24),row(toggles)}
    if visual_warning and visual_warning ~= "" then table.insert(header,text("Visual mapping: "..visual_warning,16,P.danger)) end
    if d.source_error~="" then table.insert(header,text(if revision.available() then "Edited sources pending — open Authored sources. Current finite state is preserved." else d.source_error,16,P.danger)) end
    local pane_top = if d.source_error~="" or (visual_warning and visual_warning~="") then 132 else 108
    local pane_height = math.max(160,height-pane_top-226)
    local overlays = {
        ui.column({style={absolute=true,left=16,right=16,top=14,gap=8,background=P.panel,radius=8}},header),
        ui.column({id="spell-player-bottom",style={absolute=true,left=16,right=16,bottom=14,padding=14,gap=8,background=P.panel,radius=10}},bottom),
    }
    local drawer
    if layout then
        local dock, toolbar, actions, status_rect = object(layout.dock),object(layout.toolbar),object(layout.actions),object(layout.status)
        drawer = object(layout.drawer)
        pane_top, pane_height = drawer.y, drawer.height
        for _, controls in {spells,targets,toggles} do for _, control in controls do compact_button(control) end end
        local toolbar_children = {
            ui.text(d.title,{style={width=if d.capacity ~= "" then 80 else 170,font_size=16,color=P.muted,text_wrap="none",text_overflow="ellipsis",shrink=0}}),
            inline(string.format("Total energy %.0f",supply.total),20,"#97d7e7"),
            inline(if loaded then string.format("%s %.0f / %.0f",loaded.item_id,loaded.remaining,loaded.capacity) else "No cartridge",17,P.muted),
        }
        for _, control in toggles do table.insert(toolbar_children,control) end
        local action_rows
        if layout.compact then action_rows = {row(spells),row(targets)}
        else
            local all = {}
            for _, controls in {spells,targets} do for _, control in controls do table.insert(all,control) end end
            action_rows = {row(all)}
        end
        local messages = {status}
        if visual_warning and visual_warning ~= "" then table.insert(messages,"Visual mapping: "..visual_warning) end
        if d.source_error ~= "" then table.insert(messages,"Edited sources pending — open Authored sources. "..d.source_error) end
        overlays = {
            placed(ui.node({id="spell-player-bottom",style={background=P.panel}}),dock),
            placed(ui.row({id="spell-player-toolbar",style={gap=8,align="center",wrap=true,scroll=true,clip=true}},toolbar_children),toolbar),
            placed(ui.column({id="spell-player-actions",style={gap=4,scroll=true,clip=true}},action_rows),actions),
            placed(ui.column({id="spell-player-status",style={scroll=true,clip=true}}, {text(table.concat(messages," · "),16,if d.error~="" or d.source_error~="" or (visual_warning and visual_warning~="") then P.danger else P.muted)}),status_rect),
        }
    end
    if panes.equipment_open and (not layout or layout.pane == "equipment") then
        local details={text("Equipment",23)}
        local eq=object(d.equipment)
        for _, path in eq.slots do local s=object(path);table.insert(details,text(s.title..": "..(if s.occupied then s.item_id else "empty"),18,P.muted)) end
        local equip={}
        for _, path in d.wands do local item=object(path);table.insert(equip,cmd("Equip "..item.definition,"spell-equip:"..item.id,d,"equip",{item_id=item.id})) end
        table.insert(details,row(equip))
        table.insert(details,text("Physical cartridges",21))
        for _, path in supply.cartridges do local c=object(path);table.insert(details,cmd((if c.loaded then "Loaded " else "Load / replace ")..c.item_id..string.format(" · %.0f / %.0f",c.remaining,c.capacity),"spell-load:"..c.item_id,d,"load",{item_id=c.item_id},c.loaded)) end
        table.insert(details,cmd("Unload to Inventory","spell-unload",d,"unload"))
        local panel=ui.column({id="spell-equipment",style={absolute=true,left=16,top=pane_top,width=math.min(460,width-32),height=pane_height,scroll=true,clip=true,background=P.panel,padding=14,gap=9,radius=8}},details)
        table.insert(overlays,if drawer then placed(panel,drawer) else panel)
    end
    if panes.inventory_open and (not layout or layout.pane == "inventory") then
        local w=if drawer then drawer.width else math.min(700,width-32)
        local panel=ui.column({id="spell-inventory",style={absolute=true,right=16,top=pane_top,width=w,height=pane_height,scroll=true,clip=true,background=P.panel,padding=14,radius=8}}, {
            inventory.render(d.inventory,d.inventory_interaction,d.inventory_request,"spell_player_inventory",w-28,P),
        })
        table.insert(overlays,if drawer then placed(panel,drawer) else panel)
    end
    if revision.available() and panes.sources_open and (not layout or layout.pane == "sources") then
        local panel=revision.render(d,if drawer then drawer.width+32 else width,pane_top,pane_height,P)
        table.insert(overlays,if drawer then placed(panel,drawer) else panel)
    end
    if capacity.available() and panes.capacity_open and d.capacity ~= "" and not panes.inventory_open and not panes.equipment_open and not panes.sources_open then
        local panel=capacity.render(d,if drawer then drawer.width+32 else width,pane_top,pane_height,P)
        table.insert(overlays,if drawer then placed(panel,drawer) else panel)
    end
    return ui.node({id="spell-player-hud",style={crisp=true,absolute=true,left=0,right=0,top=0,bottom=0}},overlays)
end
return M
