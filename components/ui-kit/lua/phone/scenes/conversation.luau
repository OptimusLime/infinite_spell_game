local ui=require('ui')
local t=require('phone.theme')
local conversation={}
-- Aligned words establish timing; characters animate within each word.
local function spoken_text(turn,env)
    if turn.role=='user' or not turn.captions then return turn.content end
    if turn.interrupted then return turn.display or '' end
    local now=env.time or 0
    local parts={}
    for _,caption in turn.captions do
        for _,clock in env.audio_captions or {} do
            if clock.id==caption.id then caption.clock=clock;break end
        end
        local clock=caption.clock
        if not clock or clock.duration<=0 then break end
        if clock.elapsed>=clock.duration then caption.finished_at=caption.finished_at or now end
        local tail=if caption.finished_at then now-caption.finished_at else 0
        local elapsed=math.max(0,clock.elapsed+tail-0.15)
        local visible=''
        if caption.words and #caption.words>0 then
            for _,word in caption.words do
                local length=utf8.len(word.text) or #word.text
                local count=math.floor(math.clamp((elapsed-word.start)/math.max(0.04,word.finish-word.start),0,1)*length)
                local boundary=utf8.offset(word.text,count+1) or (#word.text+1)
                if count>0 then visible..=word.text:sub(1,boundary-1) end
                if count<length then break end
            end
        else
            -- Hosts without alignment still follow playback, with approximate positions.
            local length=utf8.len(caption.text) or #caption.text
            local count=math.floor(math.clamp(elapsed/clock.duration,0,1)*length)
            local boundary=utf8.offset(caption.text,count+1) or (#caption.text+1)
            if count>0 then visible=caption.text:sub(1,boundary-1) end
        end
        if #visible>0 then table.insert(parts,visible) end
        if elapsed<clock.duration then break end
    end
    turn.display=table.concat(parts,' '):gsub('  +',' '):gsub('^%s+','')
    return turn.display
end
function conversation.render(app,env,props)
    local turns={}
    for i,turn in app.state.turns do
        local outgoing=turn.role=='user'
        local visible=spoken_text(turn,env)
        if turn.interrupted and #visible==0 then continue end
        table.insert(turns,ui.row({id=tostring(i),style={width='100%',justify=if outgoing then 'end' else 'start'}},{
            ui.column({style={max_width='90%',min_width=0,padding=12,gap=5,border=2,radius=8,border_color=t.colors.text,background=if outgoing then t.colors.accent else t.colors.raised}},{
                t.text(if outgoing then 'YOU' else string.upper(props.name),13,t.colors.muted),
                t.text(if #visible>0 then visible else string.rep('.',1+math.floor((env.time or 0)*3)%3),if env.width<600 then 18 else 20,nil,{id='message',style={text_wrap='word'}}),
            }),
        }))
    end
    if #turns==0 then table.insert(turns,t.text(if app.state.connected then 'Say hello…' else '',16,t.colors.muted)) end
    return ui.column({id='conversation',style={width='100%',gap=12,min_width=0}},turns)
end
return conversation
