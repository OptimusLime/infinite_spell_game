# Infinite Spell Game: reel

*A screenplay from the storyboard: 10 scenes, about 20 s.*

## 1. COLD OPEN: THE FLIP

*Reel · not started · `reel/cold-flip.toml`*

1.5 s, no text. The pixel town at night, then a hard cut to the frozen ink town and back, on the first beat: the hook in the first second.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 2. TWIN-MOON TIMELAPSE

*Reel · not started · `reel/twin-moon-timelapse.toml`*

5 s, no text. Locked-off over the cosy pixel town: the sun drops, the sky goes violet, the purple moon then the orange moon rise; the lamps click on in a ripple down the street; shadows swing round as the moons climb.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 3. TWO MOONS, TWO SHADOWS

*Reel · not started · `reel/two-shadows.toml`*

4 s, no text. Close on the watchtower at night, slow push in: two shadows at once, one per moon, each tinted the other moon's colour.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 4. PIXEL TO INK

*Reel · not started · `reel/pixel-to-ink.toml`*

3 s, no text. Same framing, cut on the beat: warm pixel town, frozen flat ink (two tones, hard black lines, purple and orange only), pixel, ink, faster each time, landing on ink.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 5. SNOW

*Reel · not started · `reel/snow.toml`*

2.5 s, no text. Ink square, heavy snow under the purple moon.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 6. EMBERS

*Reel · not started · `reel/embers.toml`*

2.5 s, no text. Pixel street under the orange moon: embers drifting up, heat haze bending the houses behind.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 7. FIRE FALLING INTO SNOW

*Reel · not started · `reel/firefall.toml`*

4 s, no text. Both moons up, low camera: burning flakes with star cores and tails fall into the snowy square and hiss into steam round the hero's feet.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 8. SPELLS FUSE

*Reel · not started · `reel/fusion.toml`*

6 s. Fire then fire: a sad little glow, its name types in, "Hearthglow" (2 s). Then fire then wind: the two icons snap together above the hero's head, a Wall of Flame rips across the street and its name types in letter by letter, "Wall of Flame" (4 s).

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 9. MADE IN THE EDITOR

*Reel · not started · `reel/editor-claude.toml`*

5 s. The Atelico editor, readable at phone size: a short prompt on the Claude card ("freeze the town into ink when the moons meet") and the town in the scene view swings from pixel to ink live; a node in the look graph dragged on the beat. No burned-in tagline.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

## 10. TITLE

*Reel · not started · `reel/title.toml`*

2.5 s. The ink town under both moons, fire still falling; "Infinite Spell Game" small in the lower third. Cut to black.

*Sketch: Reel v1 shot: rendered offline from the engine (render-path / render-scene on a project-copy hidden host) into storyboard/reel/clips/.*

> **NARRATOR**  
> 

---

# Not on the path

## THREE SESSIONS, ONE BUG

*The working day · not started · `making/agents.toml`*

Clock: 12:00. The game window fills the frame; along its bottom edge, a strip of three Claude panes labelled Monsters, Spells, Weather. A creature appears inside a house and stands in the kitchen; in the Monsters pane one line of monsters.luau lights up; next run, the creatures climb out of the ground in the street.

*Sketch: Capture on the shoot day: the game full frame with a strip of three Claude panes (one session per window is engine item 9, not built yet); a real spawn bug (monsters inside houses) and its one-line fix.*

> **NARRATOR**  
> Noon. I split Claude into three sessions, monsters, spells and weather, each in its own files. The first build spawns monsters inside the houses; the monsters session fixes its own file, and nothing else changes.

## PARTS, NOT CODE

*The working day · not started · `making/parts.toml`*

A clock in the corner: 9:00. The Atelico editor with one Claude pane. Typed: "a cosy pixel town under two moons, purple and orange". The marketplace panel opens; five tiles (Pixel world, Moon town, Sky, Ink world, Camera path) tick and install in about two seconds, and the scene view takes over the frame: the town running, the camera craning down from the moons.

*Sketch: Capture on the shoot day: the editor with one Claude pane, the marketplace panel, five parts installing, the scene view starting. Needs the install flow from the marketplace (not built yet).*

> **NARRATOR**  
> Nine a.m. I'm Paul, and I make the Atelico editor. I ask Claude for a pixel town under two moons. It doesn't write a renderer: it installs five parts from the Atelico marketplace that are built to fit together, so adding the camera doesn't break the sky.

## FIRE IN THE SNOW

*Weather & moons · not started · `sky/fire-in-snow.toml`*

The town square in the ink night, snow underfoot; burning flakes come down and hiss into steam where they land. The player runs under them and catches two; two fire icons fill the slots beside the wind icon.

*Sketch: The weather agent's rule (packages/weather): purple alone snow, orange alone embers, aligned firefall with steam. This card needs the flakes to be catchable as fire spells.*

> **NARRATOR**  
> The weather session's rule: the purple moon brings snow, the orange one embers, and together the embers fall as fire into the snow. Every burning flake I catch is a fire spell.

## THE FORECAST

*Weather & moons · in progress · `sky/forecast.toml`*

Clock: 15:00. Over the game, a small forecast overlay for three seconds: two moon icons sliding together on one timeline, "4 days". The game skips forward through four nights in a few seconds.

*Sketch: Shown as a three-second overlay over the game, not the editor panel. The data is the real forecast from packages/sky (moon-forecast --seed 22): sketches/world/forecast.png.*

> **NARRATOR**  
> The sky part forecasts when the moons line up: four game days. I skip ahead.

## CLIMBERS AND DIGGERS

*Weather & moons · not started · `sky/monsters-rise.toml`*

The ground cracks under both moons' light; purple and orange ink creatures pull themselves up and go for the glowing seed: the purple ones scale a house wall, the orange ones sink into the ground and surface past it.

> **NARRATOR**  
> Each moon raises its own creatures: purple ones climb over walls, orange ones dig under them. Tonight both are up, and they head for the seed.

## ORDER MATTERS

*Spells · not started · `spells/cast-order.toml`*

Big, readable order: the two spell icons pop above the player's head one after the other. Fire then wind: a wall of flame rises across the street and stops a line of creatures. Wind then fire: a fireball arcs into a crowd and bursts. Each result's typed entry flashes for half a second.

> **NARRATOR**  
> Order matters. Fire then wind: a wall of flame. Wind then fire: a fireball.

## FIRE INTO THE TOWER

*Spells · not started · `spells/fuse-building.toml`*

The player casts fire at the watchtower; its spell entry rewrites: sensor "creature in its shadow", effect "slow" becomes "slow, then burn", under a new name; the shadows get burning edges. A creature steps in, slows, and catches fire.

*Sketch: Needs the spell core and on-device fusion (queue tasks 18, 19) before the shoot; nothing to capture yet.*

> **NARRATOR**  
> The tower is a spell too, so I cast fire into it. The model invents a new tower: slow, then burn.

## FUSION ARRIVES, AND A DUD

*Spells · not started · `spells/fusion-arrives.toml`*

Clock 18:00, the same night running: a small overlay in the corner: the Fusion tile installs, while the picture stays in the ink night. The player casts fire then fire: a dull orange glow; its typed entry flashes up, "Hearthglow · effect: light", and the creatures walk straight through it.

*Sketch: Needs the Fusion part and on-device fusion (queue tasks 18, 19); the dud must be a real model output on the shoot day, not staged.*

> **NARRATOR**  
> Six p.m.: Claude installs the last part, fusion, while the night keeps running. A small AI model on the player's machine invents what two spells make. Not always something good: fire then fire, a warm glow that does nothing.

## THE SEED HOLDS

*Town seed · not started · `town/hold-the-seed.toml`*

Orange diggers burst up beside the seed, just outside the tower's shadow; the ring drains to a sliver. A held beat on the player turning to the tower and casting wind into it. Its spell entry rewrites live (a new name; effect "sweep" added) and its burning shadow turns round the seed like a lighthouse beam. Each digger it crosses goes up in flames (hit-stop, shake); the last burns a step from the seed and the ring stops draining at its last sliver. Only then do the moons slide apart above, the night's reward.

*Sketch: The finish: the lighthouse beam sweeping round the seed on its last sliver as the moons part.*

> **NARRATOR**  
> Then diggers surface right by the seed, just outside the tower's shadow, and its ring is almost empty. I cast wind into the tower to make the shadow move. The model invents a shadow that turns like a lighthouse beam. The last digger burns a step from the seed, and then the moons part.

## FLAME ON THE ROOFTOPS

*Town seed · not started · `town/rooftop-wall.toml`*

Purple climbers top the house walls and cross the roofs toward the seed. The player rushes a cast: wind then fire; the fireball arcs over the roofline and bursts harmlessly in the street behind; the ring drops. The player catches another falling flake and casts fire then wind: a wall of flame runs along the roofline and the climbers fall burning. The seed's ring has drained to a quarter.

*Sketch: Needs climbers, the catchable flake and fusion; nothing to capture yet.*

> **NARRATOR**  
> The last wave: climbers come over the walls. I fuse in a hurry, wind then fire, and the fireball sails over them. One more flake, fire then wind: a wall of flame along the rooftops, and they fall.

## PLANTING THE SEED

*Town seed · not started · `town/seed-planted.toml`*

Clock: 10:00. Claude's pane shows one new file, seed.luau, being written. Then in the game: the player presses a glowing seed into open ground; the first houses grow up around it. The seed keeps pulsing at the town's centre with a ring that shows its health; one spell slot at the screen's edge holds a wind icon.

> **NARRATOR**  
> The one rule no part has, Claude writes: you plant a seed, and the town grows around it. If the seed breaks, the town is gone. You start with one spell: wind.

## SHADOWS THAT FIGHT

*Town seed · not started · `town/shadow-defence.toml`*

The player's wind spell icon, then the watchtower in the ink night with its spell beside it: sensor "creature in its shadow", effect "slow". Each moon throws its own coloured shadow from the tower; a creature steps into the purple one and drags to a crawl.

*Sketch: One shadow per moon is in progress (world round 2): sketches/world/two-shadows.png. The sensor/effect spell needs the spell core port (not started).*

> **NARRATOR**  
> The spells session made everything a spell: a sensor and an effect. My wind is one; the watchtower is another. Sensor: a creature in its shadow. Effect: slow it.

## THE WORLD FREEZES

*The world › Cel world · in progress · `world/cel/frozen-ink.toml`*

Two flashes of earlier tries: creatures reach the seed, it shatters, the town greys out (corner clock 15:30, then 16:40). Then clock 17:50, a fresh try: the two moons touch in the sky. Hard cut from warm pixels to flat ink: two or three tones, hard black lines, purple and orange the only colours. People stopped mid-step, lamps gone cold. The picture stays in the game; the corner clock and any editor beat are small overlays.

*Sketch: Why the townsfolk freeze (options for Paul; spoken line unchanged). Paul: "the good guys are frozen in place. And so that means that ... your things are vulnerable". A) Defenceless town: by day the townsfolk light the lamps and man the tower; frozen, nothing defends but the spells you set up in daylight, so the forecast is your preparation window. B) Breakable people: the frozen townsfolk are ink statues; a creature that touches one shatters it, and at dawn the town is smaller, so you guard people as well as the seed. C) Light leash: you stay unfrozen only inside the seed's glow; step out of it and you freeze like them, so every trip for a burning flake is a risk. Sketch frames: sketches/world/switch-pixel-to-ink.png and switch-pixel-to-ink.mp4 (island); the town version waits on world round 2.*

> **NARRATOR**  
> Third try at this night. The moons meet. The ink-world part restyles every house, tree and townsperson in one pass. Nothing is redrawn, so nothing comes out off-style. The townsfolk freeze.

## THE MORNING AFTER

*The world › Pixel world · not started · `world/pixel/morning-after.toml`*

Dawn: back to warm pixels, the townsfolk moving again. Pull back into the editor, clock 19:00: today's six parts highlighted in the marketplace (Pixel world, Moon town, Sky, Ink world, Camera path, Fusion) beside the four files Claude wrote (seed, monsters, spells, weather); a small line "fusion: on-device · no server · $0 per fusion"; the game's name, Infinite Spell Game.

> **NARRATOR**  
> Today: six parts installed, four files written by Claude. Every part, fusion included, is in the Atelico marketplace. Ask your agent for it.

## A TOWN AT NIGHT

*The world › Pixel world · in progress · `world/pixel/night-town.toml`*

One continuous move. Start low and cinematic, looking up at the night sky: the purple moon and the orange moon big in the skybox. Pan down without a cut to the game's own camera, the high isometric view over the cosy pixel town, snow on the roofs and lamps lit. Then the character starts walking through the town, and for one second the whole town flips to frozen ink and back: a glimpse of tonight.

*Sketch: The opening frame: sketches/world/cinematic-sky.png (low, up at the two moons). It pans to sketches/world/player-view.png (the isometric player view); the move itself is sketches/world/pan.mp4.*

> **NARRATOR**  
> A town under two moons, one purple, one orange. At nine this morning, it didn't exist.

