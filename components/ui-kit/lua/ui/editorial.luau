local ui=require('ui')
local t=require('phone.theme')
local avatar=require('phone.avatar')
local editorial={}
function editorial.image(texture,time,style,id)
    local height=if texture:find('worlds',1,true) then 256 else 384
    local frame=avatar.frame(avatar,time or 0)[4]
    local x=(frame%8)*384
    local y=math.floor(frame/8)*height
    return ui.image({id=id,texture=texture:gsub('%.png$','-atlas.png'),atlas={x,y,x+384,y+height},style=style})
end
function editorial.art(texture,time)
    return ui.node({style={width='100%',aspect=1,radius=24,border=3,border_color=t.colors.text,clip=true}},{
        editorial.image(texture,time,{width='100%',height='100%',radius=21}),
    })
end
function editorial.compose(env,art,children,aspect)
    local wide=env.width>=900
    return (if wide then ui.row else ui.column)({style={width='100%',shrink=0,gap=if wide then 28 else 16,min_width=0,align=if wide then 'start' else 'center'}},{
        ui.column({style={width=if wide then '46%' else 200*(aspect or 1),max_width='100%',shrink=0}}, {art}),
        ui.column({style={width=if wide then '50%' else '100%',grow=if wide then 1 else 0,shrink=0,min_width=0,gap=14}},children),
    })
end
function editorial.layout(env,texture,children)
    return editorial.compose(env,editorial.art(texture,env.time),children)
end
return editorial
