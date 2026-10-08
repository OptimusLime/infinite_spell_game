-- Optional reusable form components. Apps can compose primitives directly.
local ui = require("ui")
local theme = require("theme")
local form = {}
function form.render(env)
    local content = {}
    local results = {}
    local artwork=nil
    for _, spec in env.components or {} do
        local value = env.values and env.values[spec.id] or ""
        if spec.kind == "text_input" then
            table.insert(content,theme.input(spec.id,spec.label,env.inputs[spec.id],{
                placeholder=spec.placeholder,on_submit=spec.submit and ui.action("route",{name=spec.submit}) or nil,
            }))
        elseif spec.kind == "button" then
            table.insert(content,theme.button(spec.label,ui.action("route",{name=spec.action}),{
                id=spec.id,primary=spec.tone=="primary",disabled=env.busy~=nil and not spec.always_enabled,
            }))
        elseif spec.kind == "select" then
            local selected = env.inputs[spec.id] or spec.options[1]
            local index=table.find(spec.options,selected) or 1
            local next_value=spec.options[index % #spec.options+1]
            table.insert(content,ui.row({id=spec.id,style={align="center",justify="space_between"}}, {
                theme.text(spec.label,18,theme.colors.muted),
                theme.button(selected.."  >",ui.action("input",{key=spec.id,value=next_value})),
            }))
        elseif spec.kind == "image" then
            local fallback=if env.active=="creature_forge" then "creature.png" else "worlds.png"
            artwork=ui.image({id=spec.id,texture=env.image or ("themes/atelico/illustrations/"..fallback),style={width="100%",aspect=if env.active=="creature_forge" then 1 else 1.5,radius=24}})
            if not env.image and env.active=="imaginary_places" then
                artwork=require('ui.editorial').image("themes/atelico/illustrations/worlds.png",env.time,{width="100%",aspect=1.5,radius=24},spec.id)
            end
            if artwork then
                artwork=ui.node({style={width='100%',padding=5,border=3,border_color=theme.colors.text,radius=30,background=theme.colors.accent,clip=true}},{artwork})
            end
        else
            if spec.kind=="status" and env.busy and spec.id=="status" then value=env.busy.label end
            if value~="" then
                table.insert(results,theme.text(value,22,nil,{id=spec.id,style={width="100%"}}))
            end
        end
    end
    table.insert(content,ui.column({id='results',style={width='100%',height=96,shrink=0,scroll=true}},results))
    local fields=theme.panel({id="form",style={width=if artwork and env.width>=900 then '45%' else '100%',grow=1,min_width=0,padding=0,border=0,gap=12}},content)
    if not artwork then fields.style.max_width=680;return fields end
    return require('ui.editorial').compose(env,artwork,content,if env.active=='creature_forge' then 1 else 1.5)
end
return form
