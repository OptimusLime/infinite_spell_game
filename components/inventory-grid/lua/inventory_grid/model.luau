-- The supplied paths are references published by Rust. Requests are intent, never copied Inventory.
local M = {}
function M.object(ref)
    return if ref and ref ~= "" then data.object(ref) else nil
end
function M.request(r)
    local request = M.object(r.request)
    if not request then return end
    request.operation = r.operation
    request.item_id = r.item_id or ""
    request.grid_id = r.grid_id or ""
    request.column = r.column or 0
    request.row = r.row or 0
    request.rotated = r.rotated or false
    request.expected_token = r.expected_token or 0
    request.sequence = request.sequence + 1
end
return M
