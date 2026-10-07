-- Generic composition helpers. No renderer or theme is selected here.
local ui = {}
function ui.merge(...)
    local result = {}
    for _, value in {...} do if value then for key, item in value do result[key] = item end end end
    return result
end
function ui.node(props, children)
    local node = ui.merge(props)
    node.kind = node.kind or "node"
    node.children = children or node.children or {}
    return node
end
function ui.column(props, children)
    props = ui.merge(props, { style = ui.merge(props and props.style, { direction = "column" }) })
    return ui.node(props, children)
end
function ui.row(props, children) return ui.node(props, children) end
function ui.text(value, props) return ui.node(ui.merge(props, { kind = "text", text = tostring(value or "") })) end
function ui.button(props, children) return ui.node(ui.merge(props, { kind = "button" }), children) end
function ui.input(props)
    local children={}
    if (props.value or '')=='' and (props.placeholder or '')~='' then
        local style=props.style or {}
        table.insert(children,ui.text(props.placeholder,{style={absolute=true,left=style.padding or 12,top=style.padding or 12,
            font=style.font,font_size=style.font_size or 20,color='827b70',text_wrap='word'}}))
    end
    return ui.node(ui.merge(props,{kind='input'}),children)
end
function ui.image(props) return ui.node(ui.merge(props, { kind = "image" })) end
function ui.material(props) return ui.node(ui.merge(props, { kind = "material" })) end
-- An animated character face: `face = {character, row, col, drag, still, focus}`.
function ui.face(props) return ui.node(ui.merge(props, { kind = "face" })) end
function ui.action(event, props) return ui.merge(props, { event = event }) end
-- Path-scoped keys preserve focus and editable buffers when siblings update.
function ui.finish(node, parent)
    node.id = (parent or "") .. "/" .. (node.id or "root")
    for i, child in node.children or {} do
        child.id = child.id or tostring(i)
        ui.finish(child, node.id)
    end
    if node.children and #node.children == 0 then node.children = nil end
    return node
end
return ui
