# Infinite Spell Game

*A screenplay from the storyboard: 14 scenes, about 177 s.*

## 1. A TOWN AT NIGHT

*The world › Pixel world · in progress · `world/pixel/night-town.toml`*

One continuous move. Start low and cinematic, looking up at the night sky: the purple moon and the orange moon big in the skybox. Pan down without a cut to the game's own camera, the high isometric view over the cosy pixel town, snow on the roofs and lamps lit. Then the character starts walking through the town.

*Sketch: The opening frame: sketches/world/cinematic-sky.png (low, up at the two moons). It pans to sketches/world/player-view.png (the isometric player view); the move itself is sketches/world/pan.mp4.*

> **NARRATOR**  
> A town under two moons, one purple, one orange. At nine this morning, it didn't exist.

## 2. PARTS, NOT CODE

*The working day · not started · `making/parts.toml`*

A clock in the corner: 9:00. The Atelico editor with one Claude pane. Typed: "a cosy pixel town under two moons, purple and orange". The marketplace panel opens; Claude ticks Pixel world, Moon town, Sky, Ink world and Camera path, and each installs. The scene view starts: the camera cranes down from the moons onto the town.

*Sketch: Capture on the shoot day: the editor with one Claude pane, the marketplace panel, five parts installing, the scene view starting. Needs the install flow from the marketplace (not built yet).*

> **NARRATOR**  
> Nine a.m. I'm Paul, and I make the Atelico editor. I ask Claude for a pixel town under two moons. It doesn't write a renderer: it installs five parts from the Atelico marketplace that are built to fit together, so adding the camera doesn't break the sky.

## 3. PLANTING THE SEED

*Town seed · not started · `town/seed-planted.toml`*

Clock: 10:00. Claude's pane shows one new file, seed.luau, being written. Then in the game: the player presses a glowing seed into open ground; the first houses grow up around it. The seed keeps pulsing at the town's centre with a ring that shows its health; one spell slot at the screen's edge holds a wind icon.

> **NARRATOR**  
> The one rule no part has, Claude writes: you plant a seed, and the town grows around it. If the seed breaks, the town is gone. You start with one spell: wind.

## 4. THREE SESSIONS, ONE BUG

*The working day · not started · `making/agents.toml`*

Clock: 12:00. The Claude pane splits into three, labelled Monsters, Spells, Weather, each docked beside its own editor window. A creature walks straight through a house wall; one line changes in monsters.luau, highlighted; the next run, it walks around. The other two panes keep working.

*Sketch: Capture on the shoot day: three Claude sessions, one per editor window (engine item 9, not built yet); a real wall-clipping bug and its one-line fix.*

> **NARRATOR**  
> Noon. Three jobs left: monsters, spells, weather. I split Claude into three sessions, one per editor window, each changing only its own files. The first build lets monsters walk through walls; the monsters session fixes its own file, and nothing else changes.

## 5. THE FORECAST

*Weather & moons · in progress · `sky/forecast.toml`*

Clock: 15:00. Typed to Claude: "when do the moons line up next?" The forecast panel opens: a ring per moon and a bar per moon's hours in the sky, the alignment four days ahead highlighted. The game skips forward through four nights in a few seconds.

*Sketch: The real forecast panel from packages/sky (moon-forecast --seed 22).*

> **NARRATOR**  
> The sky part runs each moon on its own clock, so I ask Claude when they next line up: in four game days. I skip ahead to that night.

## 6. THE WORLD FREEZES

*The world › Cel world · in progress · `world/cel/frozen-ink.toml`*

Two flashes of earlier tries: creatures reach the seed, it shatters, the town greys out (corner clock 15:30, then 16:40). Then clock 17:50, a fresh try: the two moons touch in the sky. Hard cut from warm pixels to flat ink: two or three tones, hard black lines, purple and orange the only colours. People stopped mid-step, lamps gone cold. The picture stays in the game; the corner clock and any editor beat are small overlays.

*Sketch: Why the townsfolk freeze (options for Paul; spoken line unchanged). Paul: "the good guys are frozen in place. And so that means that ... your things are vulnerable". A) Defenceless town: by day the townsfolk light the lamps and man the tower; frozen, nothing defends but the spells you set up in daylight, so the forecast is your preparation window. B) Breakable people: the frozen townsfolk are ink statues; a creature that touches one shatters it, and at dawn the town is smaller, so you guard people as well as the seed. C) Light leash: you stay unfrozen only inside the seed's glow; step out of it and you freeze like them, so every trip for a burning flake is a risk. Sketch frames: sketches/world/switch-pixel-to-ink.png and switch-pixel-to-ink.mp4 (island); the town version waits on world round 2.*

> **NARRATOR**  
> Third try at this night. The moons meet. The ink-world part restyles every house, tree and townsperson in one pass. Nothing is redrawn, so nothing comes out off-style. The townsfolk freeze.

## 7. CLIMBERS AND DIGGERS

*Weather & moons · not started · `sky/monsters-rise.toml`*

The ground cracks under both moons' light; purple and orange ink creatures pull themselves up and go for the glowing seed: the purple ones scale a house wall, the orange ones sink into the ground and surface past it.

> **NARRATOR**  
> Each moon raises its own creatures: purple ones climb over walls, orange ones dig under them. Tonight both are up, and they head for the seed.

## 8. SHADOWS THAT FIGHT

*Town seed · not started · `town/shadow-defence.toml`*

The player's wind spell icon, then the watchtower in the ink night with its spell beside it: sensor "creature in my shadow", effect "slow". Each moon throws its own coloured shadow from the tower; a creature steps into the purple one and drags to a crawl.

*Sketch: One shadow per moon is in progress (world round 2): sketches/world/two-shadows.png. The sensor/effect spell needs the spell core port (not started).*

> **NARRATOR**  
> The spells session made everything a spell: a sensor and an effect. My wind is one; the watchtower is another. Sensor: a creature in my shadow. Effect: slow it.

## 9. FIRE IN THE SNOW

*Weather & moons · not started · `sky/fire-in-snow.toml`*

The town square in the ink night, snow underfoot; burning flakes come down and hiss into steam where they land. The player runs under them and catches one; a fire icon fills the slot beside the wind icon.

*Sketch: The weather agent owns the fire-in-snow state and visuals (queue tasks 5, 6, 8); this card needs the flake to be catchable as a fire spell.*

> **NARRATOR**  
> Both moons at once: the weather session's fire falls into the snow. Every burning flake I catch is a fire spell.

## 10. ORDER MATTERS

*Spells · not started · `spells/cast-order.toml`*

Clock 18:00, the same night running: a small overlay in the corner: the Fusion tile installs, while the picture stays in the ink night. The player casts fire then wind (two icons pop above the head in order): a wall of flame rises across the street and stops a line of creatures. Then wind then fire: a fireball arcs into a crowd and bursts. Each result writes in as a typed spell: name, sensor, effect.

> **NARRATOR**  
> Six p.m.: Claude installs the last part, fusion, while the night keeps running. A small AI model on the player's machine invents what two spells make. Fire then wind: a wall of flame. Wind then fire: a fireball.

## 11. FIRE INTO THE TOWER

*Spells · not started · `spells/fuse-building.toml`*

The player casts fire at the watchtower; its spell entry rewrites: sensor "creature in my shadow", effect "slow" becomes "slow, then burn", under a new name; the shadows get burning edges. A creature steps in, slows, and catches fire.

*Sketch: Needs the spell core and on-device fusion (queue tasks 18, 19) before the shoot; nothing to capture yet.*

> **NARRATOR**  
> The tower is a spell too, so I cast fire into it. The model invents a new tower: slow, then burn.

## 12. FLAME ON THE ROOFTOPS

*Town seed · not started · `town/rooftop-wall.toml`*

Purple climbers top the house walls and cross the roofs toward the seed. The player catches a falling flake and casts fire then wind: a wall of flame runs along the roofline and the climbers fall burning. The seed's ring has drained to a quarter.

*Sketch: Needs climbers, the catchable flake and fusion; nothing to capture yet.*

> **NARRATOR**  
> The last wave: climbers come over the walls first. One burning flake, then wind: a wall of flame runs along the rooftops, and they fall.

## 13. THE SEED HOLDS

*Town seed · not started · `town/hold-the-seed.toml`*

Orange diggers burst up beside the seed, just outside the tower's shadow; the ring drains to a sliver. The player casts wind into the burning watchtower; its spell entry rewrites live (a new name; effect "sweep" added) and its burning shadow turns round the seed like a lighthouse beam. Each digger it crosses goes up in flames (hit-stop, shake); the last burns a step from the seed. A beat later the moons slide apart above; the ring stops at its last sliver.

*Sketch: The finish: the lighthouse beam sweeping round the seed on its last sliver as the moons part.*

> **NARRATOR**  
> Then diggers surface right by the seed, and its ring is almost empty. The tower's shadow can't reach them, so I cast wind into it to make it move. The model invents a shadow that turns like a lighthouse beam. The last digger burns a step from the seed, and then the moons part.

## 14. THE MORNING AFTER

*The world › Pixel world · not started · `world/pixel/morning-after.toml`*

Dawn: back to warm pixels, the townsfolk moving again. Pull back into the editor, clock 19:00: today's six parts highlighted in the marketplace (Pixel world, Moon town, Sky, Ink world, Camera path, Fusion) beside the four files Claude wrote (seed, monsters, spells, weather); a small line "fusion: on-device · no server · $0 per fusion"; the game's name, Infinite Spell Game.

> **NARRATOR**  
> Today: six parts installed, four files written by Claude. Every part, fusion included, is in the Atelico marketplace. Ask your agent for it.

