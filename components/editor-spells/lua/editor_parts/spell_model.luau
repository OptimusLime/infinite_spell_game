-- Only input-event buffers and transport live here. Drafts, validation and file writes are Rust.
local M = {}
function M.dashboard() return data.object("runtime.spell_authoring.dashboard") end
function M.state() return data.object("spell_authoring_ui") end
function M.request() return data.object("runtime.spell_authoring.request") end
function M.record(ref) return if ref and ref ~= "" then data.object(ref) else nil end
function M.size() return data.object("runtime.size.spell-authoring-area") end
-- These keys identify native text buffers, never domain-object fields or Store record paths.
function M.bind(generation, owner, field) return `spell-edit:{generation}:{owner}:{field}` end
function M.value(inputs, generation, owner, field, fallback)
    local v = (inputs or {})[M.bind(generation, owner, field)]
    return if v ~= nil then v else tostring(fallback or "")
end
local function number(text, label, whole)
    local n = tonumber(text)
    if not n or n ~= n or math.abs(n) == math.huge or (whole and (n < 0 or n > 3600 or n % 1 ~= 0)) then
        M.state().input_error = label .. (if whole then " must be a whole number from 0 to 3600." else " must be a finite number.")
        return nil
    end
    return n
end
function M.command(r)
    local q, state = M.request(), M.state()
    if not q or not state then return end
    state.input_error = ""
    local function value(owner, field, fallback) return M.value(r, r.generation, owner, field, fallback) end
    q.expected_generation, q.operation = r.generation, r.operation
    if r.operation == "set_metadata" then
        local m = M.record(r.metadata)
        if not m then return end
        local allocation = number(value("metadata", "allocation", m.allocation), "Allocation", false)
        if allocation == nil then return end
        q.name, q.title = value("metadata", "name", m.name), value("metadata", "title", m.title)
        q.kind, q.source = value("metadata", "kind", m.kind), value("metadata", "source", m.source)
        q.shape, q.element = value("metadata", "shape", m.shape), value("metadata", "element", m.element)
        q.container, q.allocation = value("metadata", "container", m.container), allocation
    elseif r.operation == "set_step" then
        local s = M.record(r.step)
        if not s then return end
        local every = value(s.id, "every", if s.has_every then s.every else "")
        local ticks = if every == "" then 0 else number(every, "Tick interval", true)
        if ticks == nil then return end
        q.step_id = s.id
        q.target, q.input, q.output = value(s.id, "target", s.target), value(s.id, "input", s.input), value(s.id, "output", s.output)
        q.has_every, q.every = every ~= "", ticks
    elseif r.operation == "set_parameter" or r.operation == "clear_parameter" then
        local p = M.record(r.parameter)
        if not p then return end
        q.step_id, q.parameter, q.parameter_kind = r.step_id, p.key, p.kind
        if r.operation == "set_parameter" then
            if p.kind == "number" then
                local n = number(value(r.step_id, p.key, p.number), p.key, false)
                if n == nil then return end
                q.value_number = n
            else
                q.value_text = r.choice or value(r.step_id, p.key, p.text)
            end
        end
    elseif r.operation == "add_step" then
        q.step_id = value("new-step", "id", "")
        q.component, q.phase = r.component, r.phase
    elseif r.operation == "move_step" or r.operation == "remove_step" then
        q.step_id = r.step_id
        if r.operation == "move_step" then q.direction = r.direction end
    elseif r.operation == "set_contact" then
        local ticks = number(value("contact", "every", r.every), "Contact interval", true)
        if ticks == nil then return end
        q.contact_target, q.contact_every = value("contact", "target", r.target), ticks
    elseif r.operation == "new" or r.operation == "load" or r.operation == "save_as" then
        q.file = value("file", "path", r.file)
    end
    q.sequence += 1 -- Publish last: Rust sees one complete, typed intent with its displayed generation.
end
return M
