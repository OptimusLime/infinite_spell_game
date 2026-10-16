-- Shared atlas geometry for bundled contacts and generated portal scenes.
local avatar={columns=8,rows=4,frames=32,fps=16}
function avatar.frame(animation,time)
    local last=math.max(1,animation.frames-1)
    local cursor=math.floor(time*animation.fps)%(last*2)
    return {animation.columns,animation.rows,animation.frames,if cursor<=last then cursor else last*2-cursor}
end
function avatar.texture(id) return 'themes/atelico/contacts/'..id..'-atlas.png' end
return avatar
