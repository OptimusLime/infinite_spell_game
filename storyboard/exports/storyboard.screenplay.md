# Infinite Spell Game

*A screenplay from the storyboard: 13 scenes, about 140 s.*

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

Clock: 10:00. Claude's pane shows one new file, seed.luau, being written. Then in the game: the player presses a glowing seed into open ground; the first houses grow up around it. The seed keeps pulsing at the town's centre with a ring that shows its health.

> **NARRATOR**  
> The one rule no part has, Claude writes: you plant a seed, and the town grows around it. If the seed breaks, the town is gone.

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
> The sky part runs each moon on its own clock, so I ask it when they next line up: in four game days. I skip ahead.

## 6. THE WORLD FREEZES

*The world › Cel world · in progress · `world/cel/frozen-ink.toml`*

The two moons touch in the sky. Hard cut from warm pixels to flat ink: two or three tones, hard black lines, purple and orange the only colours. People stopped mid-step, lamps gone cold. From here the picture stays in the game through the night; the corner clock and any editor beat are small overlays.

*Sketch: The look switch exists on the island: sketches/world/switch-pixel-to-ink.png and switch-pixel-to-ink.mp4; the town version waits on world round 2.*

> **NARRATOR**  
> The moons meet. The ink-world part restyles every house, tree and townsperson in one pass. Nothing is redrawn, so nothing comes out off-style. The townsfolk freeze.

## 7. FIRE IN THE SNOW

*Weather & moons · not started · `sky/fire-in-snow.toml`*

A snowfield in the ink world; burning flakes come down and hiss into steam where they land.

> **NARRATOR**  
> Both moons at once: fire falls into the snow.

## 8. MONSTERS FROM THE GROUND

*Weather & moons · not started · `sky/monsters-rise.toml`*

The ground cracks under both moons' light; purple and orange ink creatures pull themselves up and walk toward the glowing seed.

> **NARRATOR**  
> Each moon raises its own creatures. Tonight both are up, so purple ones and orange ones climb out together and head for the seed.

## 9. SHADOWS THAT FIGHT

*Town seed · not started · `town/shadow-defence.toml`*

The watchtower in the ink night with its spell beside it: sensor "creature in my shadow", effect "slow". Each moon throws its own coloured shadow from the tower; a creature steps into the purple one and drags to a crawl.

*Sketch: One shadow per moon is in progress (world round 2): sketches/world/two-shadows.png. The sensor/effect spell needs the spell core port (not started).*

> **NARRATOR**  
> The spells session made everything a spell: a sensor and an effect. The watchtower is one. Sensor: a creature in my shadow. Effect: slow it.

## 10. ORDER MATTERS

*Spells · not started · `spells/cast-order.toml`*

Clock: 18:00. A small overlay in the corner: the Fusion tile installs, while the picture stays in the ink night. The player casts fire then wind (two icons pop above the head in order): a wall of flame rises across the street and stops a line of creatures. Then wind then fire: a fireball arcs into a crowd and bursts. Each result's name writes in as it is invented.

> **NARRATOR**  
> Six p.m.: Claude installs the last part, fusion. Its small AI model runs on the player's machine and invents what two spells make. Fire then wind: a wall of flame. Wind then fire: a fireball.

## 11. FIRE INTO THE TOWER

*Spells · not started · `spells/fuse-building.toml`*

The player casts fire at the watchtower; its spell gains a second effect, "burn", and a new name writes in; the shadows get burning edges. A creature steps in, slows, and catches fire.

*Sketch: Needs the spell core and on-device fusion (the LoRA) before the shoot; nothing to capture yet.*

> **NARRATOR**  
> The tower is a spell too, so I cast fire into it. The model invents a new tower: slow, then burn.

## 12. THE SEED HOLDS

*Town seed · not started · `town/hold-the-seed.toml`*

Creatures pile onto the seed; its health ring drains to a sliver. The creatures still coming must cross the tower's burning shadow, and go up in flames. Above, the moons slide apart; the ring stops at its last sliver.

> **NARRATOR**  
> The last wave reaches the seed; its ring is almost empty. They cross the tower's shadow and burn. The moons part, and the seed holds.

## 13. THE MORNING AFTER

*The world › Pixel world · not started · `world/pixel/morning-after.toml`*

Dawn: back to warm pixels, the townsfolk moving again. Pull back into the editor, clock 19:00: today's six parts highlighted in the marketplace (Pixel world, Moon town, Sky, Ink world, Camera path, Fusion) beside the four files Claude wrote (seed, monsters, spells, weather); a small line "fusion: on-device · no server · $0 per fusion"; the game's name, Infinite Spell Game.

> **NARRATOR**  
> Today: six parts installed, four files written by Claude. Every part, fusion included, is in the Atelico marketplace. Ask your agent for it.

