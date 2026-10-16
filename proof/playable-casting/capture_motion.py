from capture_control import *
FRAMES=pathlib.Path('/tmp/game104-motion-frames');FRAMES.mkdir(exist_ok=True)
actions={30:'spell-fire',75:'spell-fire',105:'spell-equipment-toggle',130:'spell-unload',155:'spell-load:blue',180:'spell-equipment-toggle',200:'spell-select:orb',225:'spell-aim:side',250:'spell-fire'}
receipts=[]
start=state();assert start['supply']['total']==50
start_clock=request('/clock')
for frame in range(390):
    if frame in actions:
        id=actions[frame];boxes=request('/scene')['boxes'];b=next(b for b in boxes if b['id']==id);x,y,w,h=b['rect'];request('/input',{'click':{'x':x+w/2,'y':y+h/2}})
        receipts.append({'frame':frame,'at_seconds':frame/30,'action':id,'rect':b['rect']})
    step(1)
    raw=request('/shot?target=window&scale=1');(FRAMES/f'{frame:05d}.png').write_bytes(raw)
    if frame in [0,35,80,160,255,389]:
        s=record(f'motion-{frame:03d}');receipts.append({'frame':frame,'state_total':s['supply']['total'],'receipts':len(s['receipts']),'error':s['dashboard']['error']})
end=record('depleted-replaced-and-push');assert end['supply']['total']==20;assert len(end['receipts'])==4
assert [r['allocation'] for r in end['receipts']]==[20,10,10,20]
assert next(c for c in end['cartridges'] if c['item_id']=='amber')['remaining']==0
assert next(c for c in end['cartridges'] if c['item_id']=='blue')['remaining']==20
assert next(b for b in end['bodies'] if b['id']=='side')['x']>1.7
r={'capture':'Native game7890 control API frames, one actual stepped simulation/draw frame per PNG; no cuts/time acceleration','fps':30,'frames':390,'duration_seconds':13,'size':[1100,800],'start_clock':start_clock,'end_clock':request('/clock'),'actions':receipts,'source_directory':'/tmp/game104-motion-frames','output':'finite-casting-1100x800.mp4'}
(OUT/'motion-receipt.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
