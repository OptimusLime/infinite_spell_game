-- Native reusable renderer. All authoritative fields come from typed userdata; local tables are UI nodes/geometry.
local ui = require("ui")
local model = require("inventory_grid.model")
local M = { request = model.request }
local function text(value, p, size, color)
    return ui.text(value, { style = { font_size = size or p.font_size, color = color or p.text,
        text_wrap = "word", shrink = 0, width = "100%" } })
end
local function action(route, request, operation, extra)
    local args = extra or {}
    args.request, args.operation = request, operation
    return ui.action("route", { name = route, args = args })
end
local function button(label, id, on_press, p, extra)
    return ui.button({ id = id, on_press = on_press, style = ui.merge({ padding = 8, radius = 5,
        background = p.button, border_color = p.border, border = 1, min_height = 36, shrink = 0 }, extra) }, {
        text(label, p),
    })
end
local function item(ref, cell, selected, request, route, font_size, text_color, button_color, border, selected_color, item_color, accent)
    local p = { font_size = math.min(font_size, 18), text = text_color, button = button_color, border = border, selected = selected_color, item = item_color, accent = accent }
    local row = model.object(ref)
    if not row then return nil end
    local w, h = if row.rotated then row.height else row.width, if row.rotated then row.width else row.height
    local node = button(row.id, "inventory-item:" .. row.id,
        action(route, request, "select", { item_id = row.id }), p, { absolute = true,
        left = row.column * cell + 3, top = row.row * cell + 3, width = w * cell - 6, height = h * cell - 6,
        clip = true, justify = "center", min_height = 0, padding = 5, background = if selected then p.selected else p.item,
        border_color = if selected then p.accent else p.border, border = if selected then 2 else 1 })
    node.children[1].style.text_wrap = "none"
    node.children[1].style.text_overflow = "ellipsis"
    node.children[1].style.min_width = 0
    return node
end
local function grid(ref, interaction_ref, request, route, width, p)
    local g, state = model.object(ref), model.object(interaction_ref)
    if not g or not state then return nil end
    local cell = math.clamp(math.floor((width - 2) / math.max(1, g.width)), 28, 64)
    local nodes = {}
    -- A bounded renderer cannot allocate arbitrary u16 grids. Core capacity has no such UI limit.
    if g.cells > 1024 then
        return text(g.id .. ": too large for this grid view (maximum 1024 cells).", p)
    end
    for y = 0, g.height - 1 do
        for x = 0, g.width - 1 do
            table.insert(nodes, ui.button({ id = `inventory-cell:{g.id}:{x}:{y}`,
                on_press = action(route, request, "preview", { grid_id = g.id, column = x, row = y, rotated = state.rotated }),
                style = { absolute = true, left = x * cell + 1, top = y * cell + 1,
                    width = cell - 2, height = cell - 2, background = p.cell, border_color = p.border, border = 1, radius = 3 } }))
        end
    end
    for _, ref_item in g.items do
        -- Stable primitive unit arguments let unrelated items keep their field-level render cache.
        table.insert(nodes, unit(ref_item, item, ref_item, cell, state.selected_item == ref_item, request, route, p.font_size, p.text, p.button, p.border, p.selected, p.item, p.accent))
    end
    if state.preview_grid == ref then
        table.insert(nodes, ui.node({ id = "inventory-preview", style = { absolute = true,
            left = state.column * cell + 2, top = state.row * cell + 2,
            width = state.width * cell - 4, height = state.height * cell - 4,
            border = 3, border_color = if state.valid then p.success else p.danger,
            background = if state.valid then "#4caf5030" else "#ed615530", radius = 5 } }))
    end
    return ui.column({ id = "inventory-grid:" .. g.id, style = { gap = 8, shrink = 0, width = g.width * cell } }, {
        text(g.id, p, p.font_size + 1),
        text(`{g.width} × {g.height} · {g.used_cells}/{g.cells} cells`, p, p.font_size - 5, p.muted),
        ui.node({ id = "inventory-cells:" .. g.id, style = { width = g.width * cell, height = g.height * cell,
            shrink = 0, clip = true } }, nodes),
    })
end
function M.render(inventory_ref, interaction_ref, request, route, width, p)
    local inv, state = model.object(inventory_ref), model.object(interaction_ref)
    if not inv or not state then return text("Reading Inventory…", p) end
    local selected = model.object(state.selected_item)
    local controls = {}
    if selected then
        table.insert(controls, text(selected.definition, p, p.font_size + 3))
        table.insert(controls, text(`ID {selected.id} · {selected.width} × {selected.height} · owner {selected.owner}`, p, p.font_size - 2, p.muted))
        local buttons = {}
        local dest = model.object(if state.preview_grid ~= "" then state.preview_grid else state.selected_grid)
        if selected.rotatable and dest then
            table.insert(buttons, button("Rotate preview", "inventory-rotate", action(route, request, "preview", {
                grid_id = dest.id, column = state.column, row = state.row, rotated = not state.rotated }), p))
        end
        if state.valid then
            table.insert(buttons, button("Move item", "inventory-move", action(route, request, "commit", { expected_token = state.token }), p, { background = p.selected }))
        end
        table.insert(buttons, button("Cancel", "inventory-cancel", action(route, request, "cancel"), p))
        table.insert(controls, ui.row({ style = { gap = 8, wrap = true, width = "100%", shrink = 0 } }, buttons))
    end
    table.insert(controls, text(if state.error ~= "" then state.error else state.message, p, p.font_size - 1,
        if state.error ~= "" then p.danger else p.muted))
    local children = { ui.column({ id = "inventory-inspector", style = { width = "100%", padding = 12,
        gap = 8, shrink = 0, background = p.panel, border_color = p.border, border = 1, radius = 8 } }, controls) }
    local grids = {}
    for _, ref in inv.grids do
        table.insert(grids, grid(ref, interaction_ref, request, route, width, p))
    end
    table.insert(children, ui.row({ style = { width = width, gap = 24, align = "start", wrap = true, shrink = 0 } }, grids))
    -- Slot occupants remain inspectable. Transfer out of a slot belongs to its Equipment/loading owner.
    for _, ref in inv.items do
        local row = model.object(ref)
        if row and row.location_kind == "slot" then
            table.insert(children, button(row.definition .. " · " .. row.slot, "inventory-item:" .. row.id,
                action(route, request, "select", { item_id = row.id }), p))
        end
    end
    return ui.column({ style = { width = width, gap = 18, shrink = 0 } }, children)
end
return M
