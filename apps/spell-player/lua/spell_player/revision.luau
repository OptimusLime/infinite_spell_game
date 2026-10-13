-- Optional typed capability: old running hosts keep their existing player until this producer is built.
local ui = require("ui")
local M = {}
function M.available() return data.schema("PlayerRevisionView") ~= nil end
local function object(ref) return if ref and ref ~= "" then data.object(ref) else nil end
function M.toggle(r)
    if not M.available() then return end
    local state = object(r.ui); if not state then return end
    state.sources_open = not state.sources_open
    if state.sources_open then
        state.inventory_open,state.equipment_open = false,false
        -- An older running producer can hot reload this UI before the optional fixture is built.
        if data.schema("PlayerCapacityRequest") ~= nil then state.capacity_open = false end
    end
end
function M.request(r)
    if not M.available() then return end
    local request = object(r.request); if not request then return end
    request.operation, request.spell_id = r.operation,r.spell_id
    request.expected_current_revision = r.current
    request.expected_candidate_revision = r.candidate or ""
    request.expected_preview_token = r.token or 0
    request.sequence = request.sequence + 1
end
local function label(text,size,color)
    return ui.text(text,{style={width="100%",font_size=size,color=color,text_wrap="word",shrink=0}})
end
local function button(text,id,args,p)
    return ui.button({id=id,on_press=ui.action("route",{name="spell_player_revision",args=args}),style={background=p.button,border=1,border_color=p.border,padding=10,radius=6,shrink=0}}, {label(text,18,p.text)})
end
function M.render(d,width,top,height,p)
    local review=object(d.revision_review)
    local children={label("Authored spell sources",23,p.text),label("Preview first; admit an exact revision for future casts. Current finite state stays.",17,p.muted)}
    local previews={}
    for _, ref in d.spells do
        local spell=object(ref)
        table.insert(previews,button("Preview "..spell.id,"spell-revision-preview:"..spell.id,{request=d.revision_request,operation="preview",spell_id=spell.id,current=spell.revision},p))
    end
    table.insert(children,ui.row({style={width="100%",wrap=true,gap=8,shrink=0}},previews))
    if review and review.spell_id~="" then
        table.insert(children,label(review.spell_id.." · from revision "..string.sub(review.current_revision,1,12),17,p.muted))
        if review.candidate_revision~="" then
            table.insert(children,label(review.current_title.." → "..review.candidate_title,20,p.text))
            table.insert(children,label(string.format("Allocation %.0f → %.0f · candidate %s",review.current_allocation,review.candidate_allocation,string.sub(review.candidate_revision,1,12)),18,p.muted))
        end
        if review.valid and review.changed then
            table.insert(children,button("Admit this revision","spell-revision-admit",{request=d.revision_request,operation="admit",spell_id=review.spell_id,current=review.current_revision,candidate=review.candidate_revision,token=review.preview_token},p))
        end
        if review.error~="" then table.insert(children,label(review.error,18,p.danger)) end
        if review.notice~="" then table.insert(children,label(review.notice,17,p.muted)) end
        for _, file in review.source_files do
            table.insert(children,ui.text(file,{style={width="100%",min_width=0,font_size=15,color=p.muted,text_wrap="none",text_overflow="ellipsis",shrink=0}}))
        end
    end
    if d.source_error~="" and string.find(d.source_error,"invalid",1,true) then table.insert(children,label(d.source_error,16,p.danger)) end
    return ui.column({id="spell-sources",style={absolute=true,left=16,top=top,width=math.min(720,width-32),height=height,scroll=true,clip=true,background=p.panel,padding=14,gap=9,radius=8}},children)
end
return M
