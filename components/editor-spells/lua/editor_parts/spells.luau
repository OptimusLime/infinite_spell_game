local ui, kit = require("ui"), require("kit")
local model = require("editor_parts.spell_model")
local part = { name = "spells", slot = "center", order = 0, docs = { "spell" }, edits = true, routes = {} }
local function text(value, opts)
    return kit.text(value or "", ui.merge({ size = 17, wrap = true, style = { width = "100%", shrink = 0 } }, opts or {}))
end
local function note(value) return text(value, { size = 14, color = kit.c.muted }) end
local function column(children, style, id)
    return ui.column({ id = id, style = ui.merge({ width = "100%", shrink = 0, min_width = 0, gap = 10 }, style or {}) }, children)
end
local function row(children, style)
    return ui.row({ style = ui.merge({ width = "100%", shrink = 0, min_width = 0, gap = 8, wrap = true, align = "center" }, style or {}) }, children)
end
local function button(label, id, args, opts)
    return kit.button(label, kit.route("spell_authoring_command", args), ui.merge({ id = id, height = 34, size = 14, style = { padding_x = 10, min_width = 0 } }, opts or {}))
end
local function nav(label, id, section, active)
    return kit.button(label, kit.route("spell_authoring_section", { section = section }), { id = id, active = active, height = 34, size = 14 })
end
local function field(label, d, owner, key, fallback, inputs)
    return column({ note(label), ui.input({ id = `spell-field:{owner}:{key}`, bind = model.bind(d.generation, owner, key), value = model.value(inputs, d.generation, owner, key, fallback), style = { width = "100%", height = 38, padding_x = 10, font_size = 16, color = kit.c.text, background = kit.c.header, border = 1, border_color = kit.c.border, radius = 5, clip = true, text_wrap = "none" } }) }, { gap = 4 })
end
local function choices(label, d, owner, key, fallback, values, inputs)
    local selected = model.value(inputs, d.generation, owner, key, fallback)
    local buttons = {}
    for _, value in values do
        table.insert(buttons, kit.button(value, ui.action("input", { key = model.bind(d.generation, owner, key), value = value }), { id = `spell-choice:{owner}:{key}:{value}`, active = value == selected, size = 14, height = 32 }))
    end
    return column({ note(label), row(buttons) }, { gap = 4 })
end
local function args(d, operation, extra) return ui.merge({ operation = operation, generation = d.generation }, extra or {}) end
local function metadata(d, inputs)
    local m = model.record(d.metadata)
    if not m then return column({ note("Create or open a definition first.") }) end
    return column({
        text("Spell details", { bold = true, size = 21 }),
        note("Apply these fields to the draft, then Save when its definition is valid."),
        field("Identifier", d, "metadata", "name", m.name, inputs),
        field("Display name", d, "metadata", "title", m.title, inputs),
        choices("Definition kind", d, "metadata", "kind", m.kind, { "spell", "building", "creature", "seed" }, inputs),
        choices("Energy source", d, "metadata", "source", m.source, { "wand", "town", "moon", "seed" }, inputs),
        choices("Container", d, "metadata", "container", m.container, m.containers, inputs),
        field("Allocated energy", d, "metadata", "allocation", m.allocation, inputs),
        field("Shape (optional)", d, "metadata", "shape", m.shape, inputs),
        field("Element (optional)", d, "metadata", "element", m.element, inputs),
        button("Apply details", "spell-apply-metadata", args(d, "set_metadata", { metadata = d.metadata }), { tone = "primary" }),
    })
end
local function files(d, inputs)
    return column({
        text("Definition file", { bold = true, size = 21 }),
        note("New starts a blank draft. Save as creates a new file; it never replaces an existing file."),
        field("Project path · .spell.yaml", d, "file", "path", d.file, inputs),
        row({
            button("New draft", "spell-new", args(d, "new", { file = d.file })),
            button("Open file", "spell-load", args(d, "load", { file = d.file })),
            button("Save as new file", "spell-save-as", args(d, "save_as", { file = d.file }), { disabled = not d.loaded or not d.valid }),
        }),
        button("Discard edits and reload", "spell-reload", args(d, "reload"), { disabled = d.file == "" or d.new_file }),
        note("Editing and saving do not change an active Player. Preview and admit the saved revision in that Player when ready."),
    })
end
local function parameter(d, step, ref, inputs)
    local p = model.record(ref)
    local contents = { row({ text(p.key, { bold = true, style = { grow = 1, width = 0, min_width = 0 } }), text(if p.has_value then "Set" else "Default", { size = 14, color = kit.c.muted, style = { width = "auto", shrink = 0 } }) }) }
    local command = args(d, "set_parameter", { step_id = step.id, parameter = ref })
    if p.kind == "choice" then
        local buttons = {}
        for _, choice in p.choices do table.insert(buttons, button(choice, `spell-param-choice:{step.id}:{p.key}:{choice}`, ui.merge(command, { choice = choice }), { active = p.text == choice })) end
        table.insert(contents, row(buttons))
        table.insert(contents, button("Use default", `spell-param-clear:{p.key}`, args(d, "clear_parameter", { step_id = step.id, parameter = ref }), { disabled = not p.has_value }))
    else
        table.insert(contents, field(if p.kind == "number" then "Value" else "Text", d, step.id, p.key, if p.kind == "number" then p.number else p.text, inputs))
        table.insert(contents, row({
            button("Apply", `spell-param-apply:{p.key}`, command),
            button("Use default", `spell-param-clear:{p.key}`, args(d, "clear_parameter", { step_id = step.id, parameter = ref }), { disabled = not p.has_value }),
        }))
    end
    if p.has_min or p.has_max then table.insert(contents, note((if p.has_min then `Min {p.min}` else "") .. (if p.has_min and p.has_max then " · " else "") .. (if p.has_max then `Max {p.max}` else ""))) end
    return column(contents, { padding_y = 10, border_bottom = 1, border_color = kit.c.border })
end
local function inspector(d, state, inputs)
    local s = model.record(d.selected_step)
    if not s then return column({ text("Select a step", { bold = true }), note("Inspect one part of the definition at a time, or add a catalog component to this phase.") }) end
    local items = {
        text(s.title, { bold = true, size = 21 }), note(s.id .. " · " .. s.component), note(s.description),
        row({
            button("Move up", "spell-step-up", args(d, "move_step", { step_id = s.id, direction = "up" }), { disabled = s.index == 0 }),
            button("Move down", "spell-step-down", args(d, "move_step", { step_id = s.id, direction = "down" }), { disabled = s.index + 1 >= s.total }),
            button("Remove", "spell-step-remove", args(d, "remove_step", { step_id = s.id })),
        }),
        kit.button(if state.connections_open then "Hide connections" else "Edit connections", kit.route("spell_authoring_connections", {}), { id = "spell-connections", size = 14, height = 34 }),
    }
    if state.connections_open then
        if s.takes_target then table.insert(items, field("Target · " .. s.target_type, d, s.id, "target", s.target, inputs)) end
        if s.takes_input then table.insert(items, field("Input · " .. s.input_type, d, s.id, "input", s.input, inputs)) end
        if s.takes_output then table.insert(items, field("Output handle · " .. s.output_type, d, s.id, "output", s.output, inputs)) end
        if s.phase == "tick" then table.insert(items, field("Run every N ticks · blank uses default", d, s.id, "every", if s.has_every then s.every else "", inputs)) end
        table.insert(items, button("Apply connections", "spell-apply-connections", args(d, "set_step", { step = d.selected_step })))
    end
    for _, ref in s.parameters do table.insert(items, parameter(d, s, ref, inputs)) end
    return column(items)
end
local function catalog(d, state, inputs)
    local cards = { text("Add a step", { bold = true, size = 21 }), note("Only catalog components valid for this phase are shown.") }
    for _, ref in d.components do
        local c = model.record(ref)
        table.insert(cards, ui.button({ id = "spell-component:" .. c.id, on_press = kit.route("spell_authoring_component", { id = c.id }), style = { width = "100%", direction = "column", align = "start", shrink = 0, padding = 10, gap = 4, border = 1, border_color = if ref == d.selected_component then kit.c.accent else kit.c.border, background = kit.c.header, radius = 5 } }, { text(c.title, { bold = true }), note(c.kind .. " · " .. c.description) }))
    end
    local selected = model.record(d.selected_component)
    local detail = if selected then column({
        text(selected.title, { bold = true, size = 21 }), note(selected.description),
        field("New step identifier", d, "new-step", "id", "", inputs),
        button("Add to phase", "spell-add-confirm", args(d, "add_step", { component = selected.id, phase = state.selected_phase }), { tone = "primary" }),
        note("Add the step, then set its handles and parameters. Validation will identify any missing connections."),
    }) else column({ note("Choose a component to see its description and add it.") })
    return column(cards), detail
end
function part.routes.spell_authoring_command(self, r) model.command(r) end
function part.routes.spell_authoring_section(self, r) local s = model.state(); s.section, s.input_error = r.section, ""; if r.section == "catalog" then s.connections_open = false end end
function part.routes.spell_authoring_phase(self, r) local s = model.state(); s.selected_phase, s.selected_step, s.section = r.phase, "", "steps" end
function part.routes.spell_authoring_step(self, r) local s = model.state(); s.selected_step, s.section, s.connections_open = r.id, "steps", false end
function part.routes.spell_authoring_component(self, r) model.state().selected_component = r.id end
function part.routes.spell_authoring_connections() local s = model.state(); s.connections_open = not s.connections_open end
function part.render(style, focus, inputs)
    local d, state = model.dashboard(), model.state()
    if not d or not state then return column({ note("Spell authoring requires the current native host.") }, style) end
    local measured = model.size()
    local width = math.max(280, (if measured then measured.w else 960) - 36)
    local m = model.record(d.metadata)
    local title = if m and m.title ~= "" then m.title elseif m and m.name ~= "" then m.name else "Spell authoring"
    local status = if d.conflict then "File changed" elseif not d.loaded then "No draft" elseif not d.valid then "Needs changes" elseif d.dirty or d.new_file then "Unsaved" else "Saved"
    local top = {
        row({ text(title, { size = 24, bold = true, style = { grow = 1, width = 0, min_width = 0 } }), text(status, { size = 14, color = if d.conflict or not d.valid then kit.c.danger else kit.c.muted, style = { width = "auto", shrink = 0 } }) }),
        text(if d.new_file then "New definition · choose a file with Save as" elseif d.file ~= "" then string.match(d.file, "[^/]+$") or d.file else "No file selected", { size = 14, color = kit.c.muted, style = { width = "100%", text_wrap = "none", text_overflow = "ellipsis", clip = true } }),
        row({
            if d.new_file then nav("Save as…", "spell-save", "file", false) else button("Save", "spell-save", args(d, "save"), { tone = "primary", disabled = not d.loaded or not d.valid or d.conflict }),
            nav("Steps", "spell-show-steps", "steps", state.section == "steps"),
            nav("Details", "spell-show-metadata", "metadata", state.section == "metadata"),
            nav("File…", "spell-show-file", "file", state.section == "file"),
        }),
    }
    local message = if state.input_error ~= "" then state.input_error elseif d.error ~= "" then d.error elseif d.validation ~= "" then d.validation else d.notice
    if message ~= "" then table.insert(top, text(message, { size = 14, color = if state.input_error ~= "" or d.error ~= "" or not d.valid then kit.c.danger else kit.c.muted })) end
    local body
    if state.section == "metadata" then body = metadata(d, inputs)
    elseif state.section == "file" or not d.loaded then body = files(d, inputs)
    else
        local phases = {}
        for _, ref in d.phases do
            local p = model.record(ref)
            table.insert(phases, kit.button(p.title .. " · " .. tostring(#p.steps), kit.route("spell_authoring_phase", { phase = p.key }), { id = "spell-phase:" .. p.key, active = state.selected_phase == p.key, height = 34, size = 14 }))
        end
        local left, right
        if state.section == "catalog" then left, right = catalog(d, state, inputs)
        else
            local list = { row({ text("Ordered steps", { bold = true, style = { grow = 1, width = 0 } }), nav("Add step", "spell-add-step", "catalog", false) }) }
            for _, ref in d.steps do
                local s = model.record(ref)
                table.insert(list, ui.button({ id = "spell-step:" .. s.id, on_press = kit.route("spell_authoring_step", { id = s.id }), style = { width = "100%", direction = "column", align = "start", padding = 10, shrink = 0, gap = 3, background = kit.c.header, border = 1, border_color = if ref == d.selected_step then kit.c.accent else kit.c.border, radius = 5 } }, { text(s.title, { bold = true }), note(s.id) }))
            end
            if #d.steps == 0 then table.insert(list, note("No steps in this phase.")) end
            if state.selected_phase == "contact" then
                table.insert(list, field("Contact body handle", d, "contact", "target", d.contact_target, inputs))
                table.insert(list, field("Sample every N ticks", d, "contact", "every", d.contact_every, inputs))
                table.insert(list, button("Apply contact hook", "spell-contact-apply", args(d, "set_contact", { target = d.contact_target, every = d.contact_every })))
            end
            left, right = column(list), inspector(d, state, inputs)
        end
        if width >= 620 then
            left.style.width, right.style.width = math.floor(width * 0.4) - 10, math.ceil(width * 0.6) - 10
            body = column({ row(phases), row({ left, right }, { wrap = false, gap = 20, align = "start" }) })
        else body = column({ row(phases), left, right }) end
    end
    return ui.column({ id = "panel:spells", style = ui.merge({ background = kit.c.panel, min_height = 0, min_width = 0, clip = true }, style) }, {
        { kind = "measure", name = "spell-authoring-area", style = { absolute = true, left = 0, top = 0, width = "100%", height = "100%" } },
        column(top, { padding = 18, border_bottom = 1, border_color = kit.c.border }),
        ui.column({ id = "spell-authoring-scroll:" .. state.section .. ":" .. state.selected_phase .. ":" .. state.selected_step, style = { width = "100%", grow = 1, min_height = 0, scroll = true, clip = true } }, {
            column({ body }, { width = width + 36, padding = 18, padding_bottom = 72 }),
        }),
    })
end
return part
