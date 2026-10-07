-- Shared reel palette and native controls.
local ui=require('ui')
local pixels=require('themes.portal.theme')
local t={colors={background='fdf8ef',panel='fdf8ef',raised='eee6d8',border='1a1100',text='1a1100',muted='594d37',accent='aa96ff',purple='aa96ff',green='aa96ff',red='e87d75'}}
local c=t.colors
t.font='themes/atelico/roobert.ttf'
function t.text(value,size,color,props)
    return ui.text(value,ui.merge(props,{style=ui.merge({font=t.font,font_size=math.floor(size or 18),color=color or c.text,shrink=0,text_wrap='word'},props and props.style)}))
end
function t.panel(props,children)
    return ui.column(ui.merge(props,{style=ui.merge({background=c.panel,border=2,border_color=c.border,padding=14,gap=10,min_width=0,shrink=0},props and props.style)}),children)
end
function t.button(label,action,props)
    props=props or {}
    local primary=props.tone=='call'
    local color=if primary then c.green elseif props.tone=='end' then c.red else c.raised
    return ui.button(ui.merge(props,{on_press=action,
        style=ui.merge({height=48,min_height=48,shrink=0,padding_x=12,align='center',justify='center',border=2,border_color=if primary then '1a1100' elseif props.tone=='end' then '1a1100' else c.border,background=color,min_width=0,gap=8},props.style),
        hover={background=if primary then 'baaaff' elseif props.tone=='end' then 'e8ab50' else 'e8dfce'}}),{
        t.text(label,props.size or 20,c.text,{style={text_wrap='none'}}),
    })
end
function t.icon(name,action,props)
    props=props or {}
    return ui.button(ui.merge(props,{on_press=action,style=ui.merge({width=44,height=44,min_height=44,shrink=0,align='center',justify='center',border=2,border_color=c.text,background=c.background},props.style),hover={background=c.raised}}),{
        ui.image({texture='themes/atelico/'..name..'.png',style={width=26,height=26}}),
    })
end
function t.input(id,value,placeholder,action,props)
    return ui.input(ui.merge(props,{id=id,bind=id,value=value or '',placeholder=placeholder or '',on_submit=action,focus_border=c.accent,
        style=ui.merge({width='100%',min_width=0,height=48,min_height=48,shrink=0,padding=10,border=2,border_color=c.border,background=c.background,color=c.text,font=t.font,font_size=24,text_wrap='none'},props and props.style)}))
end
function t.portal(props)
    return ui.material(ui.merge({material={shader='phone/portal.wgsl',uniforms={{0.67,0.59,1,1},{0,160,0,0}}}},props))
end
function t.background() return pixels.background() end
function t.contact(sheet,action,props)
    return ui.button(ui.merge(props,{on_press=action,
        style={width='100%',height=88,min_height=88,padding=6,gap=16,align='center',shrink=0,border=2,radius=44,border_color=c.border,background=c.panel},
        hover={background=c.raised}}),{
        ui.node({style={width=72,height=72,shrink=0,radius=36,clip=true,border=2,border_color=c.border,background=c.accent}},{
            ui.material({material={shader='phone/avatar.wgsl',texture=require('phone.avatar').texture(sheet.id)},style={width='100%',height='100%'}}),
        }),
        t.text(sheet.name,27,nil,{style={grow=1,min_width=0}}),
        t.text(if props and props.edit then '✎' else '↗',24,nil,{style={padding_right=16}}),
    })
end
return t
