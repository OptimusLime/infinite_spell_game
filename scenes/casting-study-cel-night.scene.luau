-- Provisional game integration study. Same game-owned configuration/session in all four comparison documents.
-- Presentation, dimensions and balance remain unreviewed; this is not an in-game look-switch mechanic.
return {
 name = "Casting study — cel night (unreviewed)", width = 1100, height = 720, preview = "frame",
 root = {name="root",type="App",props={app="spell-player",title="Spell player",header=false,padding=0,spacing=0,align="stretch",colors={background="#182339"}},children={
  {name="world",type="World",props={stage=true,camera="study",ground="#3e4e62",outline=1.4},children={
   {name="study",type="Camera",props={x=5,y=6,z=-6,look_x=0.4,look_y=-1.3,look_z=1.1,fov=42,hour=22.5,time=0,target="Provisional comparison; framing and art are unreviewed"}},
  }},
  {name="player",type="SpellPlayer",props={config="players/casting-study.player.toml",session="casting-study",world="world"}},
 }},
}
