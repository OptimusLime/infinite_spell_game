import urllib.request, urllib.parse, json, pathlib, time, hashlib
BASE='http://127.0.0.1:7890'
OUT=pathlib.Path('/Users/paul/coding/infinite_spell_game/proof/playable-casting')
def request(path,body=None):
    data=None if body is None else (body.encode() if isinstance(body,str) else json.dumps(body).encode())
    req=urllib.request.Request(BASE+path,data=data,headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(req,timeout=65) as r:
        raw=r.read()
        return raw if r.headers.get_content_type()=='image/png' else json.loads(raw or b'null')
def get(path): return request('/get?path='+urllib.parse.quote(path,safe=''))['value']
def setv(path,value): return request('/set',{'path':path,'value':value})
def step(n=4):
    result=None
    for _ in range(n):
        time.sleep(.02)
        result=request('/step','1')
    return result
def press(id):
    scene=request('/scene')
    boxes=scene.get('boxes',[])+scene.get('editor_boxes',[])
    b=next((v for v in boxes if v['id']==id),None)
    if not b: raise AssertionError(f'{id} missing; boxes='+str([x['id'] for x in boxes]))
    x,y,w,h=b['rect']; request('/input',{'click':{'x':x+w/2,'y':y+h/2}}); step(4)
    return b

def dashboards():
    return get('runtime.spell_player.sessions')
def state():
    sessions=dashboards(); assert len(sessions)==1,sessions.keys()
    key,content=next(iter(sessions.items())); d=content['dashboard']; s=get(d['supply'])
    return {'root':'runtime.spell_player.sessions.'+key,'dashboard':d,'supply':s,
       'receipts':[get(p) for p in s['receipts']], 'instances':[get(p) for p in s['instances']],
       'cartridges':[get(p) for p in s['cartridges']], 'bodies':[get(p) for p in d['bodies']]}
def record(name):
    v=state(); OUT.mkdir(parents=True,exist_ok=True); (OUT/(name+'.json')).write_text(json.dumps(v,indent=2)+'\n'); return v

def shot(name):
    raw=request('/shot?target=window&scale=1'); OUT.mkdir(parents=True,exist_ok=True); p=OUT/(name+'.png'); p.write_bytes(raw); return p

def open_doc(suffix):
    setv('editor.ui.doc','scenes/casting-study-'+suffix+'.scene.luau'); step(5)
