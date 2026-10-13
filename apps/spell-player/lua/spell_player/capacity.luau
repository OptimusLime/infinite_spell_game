-- A fixture-only view of Rust-owned grants and the shared debit journal; no prices, clocks or balances live here.
local ui = require("ui")
local M = {}
function M.available() return data.schema("PlayerCapacityRequest") ~= nil end
function M.activate(r)
    local q = data.object(r.request)
    if not q then return end
    q.item_id, q.source_id = r.item_id, r.source_id
    q.expected_config, q.expected_remaining = r.configuration, r.remaining
    q.expected_token = r.token
    q.sequence += 1 -- Explicit admission last; display/refresh never submits a payment.
end
function M.render(d, width, top, height, palette)
    local capacity = data.object(d.capacity)
    local supply = data.object(d.supply)
    if not capacity or not supply then return nil end
    local w = math.min(900, width - 32)
    local function text(value, size, color)
        return ui.text(value, { style = { width = w-28, font_size = size or 18, color = color or palette.text, text_wrap = "word", shrink = 0 } })
    end
    local children = {
        text("Capacity · engineering fixture", 23),
        text(capacity.note, 16, palette.muted),
        text(`World tick {capacity.tick} · {if capacity.clock_running then "clock advancing" else "clock stopped — payments locked"}`, 17),
    }
    for _, ref in capacity.expanders do
        local e = data.object(ref)
        if e then
            table.insert(children, text(`{e.item_id}: {e.baseline_width}×{e.baseline_height} → {e.expanded_width}×{e.expanded_height} · {e.phase}`, 20))
            if e.revision > 0 then table.insert(children, text(`Paid through tick {e.paid_until} · grant revision {e.revision}`, 17)) end
            if e.reason ~= "" then table.insert(children, text(e.reason, 18, palette.danger)) end
            if #e.overflow_items > 0 then table.insert(children, text(`{#e.overflow_items} item(s) extend beyond the baseline. All contents remain preserved.`, 17)) end
            local sources = {}
            for _, source_ref in supply.cartridges do
                local source = data.object(source_ref)
                if source then
                    local enabled = e.can_activate and source.remaining >= e.activation_cost
                    table.insert(sources, ui.button({ id = `capacity-activate:{e.item_id}:{source.item_id}`,
                        on_press = if enabled then ui.action("route", { name = "spell_player_capacity_activate", args = {
                            request = d.capacity_request, item_id = e.item_id, source_id = source.item_id,
                            configuration = e.configuration, remaining = source.remaining, token = d.token,
                        } }) else nil,
                        disabled = not enabled,
                        style = { padding = 10, background = palette.button, border = 1, border_color = palette.border, radius = 6, shrink = 0 },
                    }, { ui.text(`Activate from {source.item_id}: {e.activation_cost} energy ({source.remaining} available)`, { style = { font_size = 18, color = if enabled then palette.text else palette.muted } }) }))
                end
            end
            table.insert(children, ui.row({ style = { width = w-28, wrap = true, gap = 8, shrink = 0 } }, sources))
        end
    end
    table.insert(children, text(`Shared payments · {#capacity.debits}`, 20))
    for i, ref in capacity.debits do
        if i > #capacity.debits-6 then
            local debit = data.object(ref)
            if debit then table.insert(children, text(`{debit.purpose} · {debit.cartridge_id}: {debit.before} → {debit.after} (−{debit.amount})`, 17)) end
        end
    end
    return ui.column({ id = "spell-capacity", style = { absolute = true, right = 16, top = top, width = w, height = height, scroll = true, clip = true, background = palette.panel, padding = 14, gap = 9, radius = 8 } }, children)
end
return M
