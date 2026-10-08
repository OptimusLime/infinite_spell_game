local ui = require("ui")
local graphics = require("graphics")
local theme = {
    name = "Pixel Portal",
    font = "themes/atelico/roobert.ttf",
    colors = { background="fdf8ef", panel="fdf8ef", raised="eee6d8", border="1a1100",
        text="1a1100", muted="594d37", accent="aa96ff", purple="aa96ff", danger="d68000" },
    materials = {
        portal = graphics.material({ shader=graphics.shader("themes/portal/portal.wgsl"),
            uniforms={{0.67,0.59,1,1},{0.35,128,1,0}} }),
        wallpaper = graphics.material({ shader=graphics.shader("themes/portal/stars.wgsl"),
            uniforms={{0.55,0.66,0.65,0.45}} }),
    },
    icons = {},
}
local c = theme.colors
local patterns = {
    phone={"................","...###....###...","..#####..#####..","..#####..#####..","...###....###...","...##......##...","...##......##...","...###....###...","....########....",".....######.....","......####......","................","....#......#....","......#..#......","................","................"},
    places={"................","............#...","...###.....###..","..#####.....#...","...###..........","................",".........##.....","....##..####....","...##########...","..############..",".##############.",".##############.","................","....######......","................","................"},
    oracle={"................",".......##.......","......####......",".....######.....","....########....","...##########...","..####....####..",".####..##..####.","..####....####..","...##########...","....########....",".....######.....","......####......",".......##.......","................","................"},
    app={"................","......####......","......####......","......####......","......####......","..############..","..############..","..############..","..############..","..############..","......####......","......####......","......####......","................","................","................"},
}
for name, rows in patterns do theme.icons[name] = graphics.texture(graphics.pixels(rows, { ["#"]=if name=="places" then c.purple else c.accent })) end
function theme.text(value, size, color, props)
    return ui.text(value, ui.merge(props, {style=ui.merge({font=theme.font,font_size=size or 24,color=color or c.text},props and props.style)}))
end
function theme.panel(props, children)
    return ui.column(ui.merge(props,{style=ui.merge({padding=22,border=2,border_color=c.border,background=c.panel,gap=14},props and props.style)}), children)
end
function theme.button(label, action, props)
    local primary = props and props.primary
    return ui.button(ui.merge(props, {on_press=action,
        style=ui.merge({padding_x=18,padding_y=8,min_height=42,align="center",justify="center",border=2,
            border_color=if primary then c.accent else c.border,background=if primary then c.accent else c.raised},props and props.style),
        hover={background=if primary then "baaaff" else "e8dfce"},
    }), {theme.text(label,20,c.text,{style={text_wrap="none"}})})
end
function theme.input(id, label, value, props)
    return ui.column({id=id.."_field",style={gap=8,width="100%"}}, {
        theme.text(label,18,c.muted),
        ui.input(ui.merge(props, {id=id,bind=id,value=value or "",focus_border=c.accent,
            style=ui.merge({min_height=48,width="100%",padding=12,border=2,border_color=c.border,
                background=c.background,color=c.text,font=theme.font,font_size=20,text_wrap="word",height=136,max_height=136,shrink=0},props and props.style)})),
    })
end
function theme.portal(props)
    return ui.material(ui.merge({material=theme.materials.portal},props))
end
function theme.background()
    return ui.node({id="background",style={absolute=true,width="100%",height="100%",background=c.background,z=-10}})
end

return theme
