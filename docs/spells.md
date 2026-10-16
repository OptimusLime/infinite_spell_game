# Spells: the spell core

Paul: "everything is quote-unquote the spell system so the town is a spell system the npc is a spell system and when
i mean spell system i mean sensors effectors and an action and even the buildings themselves can potentially have
defenses and so they are potentially combinable." / "spell core is not option, pick reasonable. obviously
spellcraft is fie".

The core is `packages/spellcraft` in the engine (crate `component-spellcraft`). It is ported from creature_3d_studio's
`crates/spellcraft`. The data is in the engine's `spells/` folder.

## What was ported, and what was not

| Spellcraft part | Here | Why |
|---|---|---|
| Versioned YAML spells: `on_start` / `on_tick` / `on_contact`, ordered steps, typed handles | Same format (`spells/*.spell.yaml`), plus `kind`, `title`, `shape`, `element` | Paul's two visible parts, and what holds the spell |
| Component catalog (phases, typed ports, bounded params) | `packages/spellcraft/catalog.yaml`, 18 components, each with a mana price | The game's sensors and effectors |
| Validator (container capacity, mana bounds, phases, no forward or mistyped references) | Same checks | A spell is valid before it exists |
| Containers seed / field / grand (4/8/16 steps, 40/80/140 mana) | Same | |
| Cast instance: mana moved over once; spent on sensors, logic, rules, effectors; exhausted with the rest dissipated; no refunds | Same (`runtime.rs`) | "Computation consumes mana" |
| Lua 5.4 brains metered per VM instruction | **Luau** brains metered per VM work unit (each call and loop turn) | One binary links one Lua, and the engine's is Luau. Its interrupt is deterministic and still makes a looping brain cost more than a lean one |
| `WorldAdapter` | Narrowed to `form`, `sense`, `apply`, `exists`, `take_contact` | The moon town implements it (`town.rs`); a test fixture does too |
| Rapier physics, HTTP server, collaboration (Yrs), cloud, history, review, destruction | Not ported | Not needed to run the game's spells; they can come later behind the same adapter |

`crates/items`, the earlier port without mana that the bomb-arrow flow uses, is left as it is.

## Everything is a spell

| Thing | File | Sensor | Logic | Effect | Action |
|---|---|---|---|---|---|
| Wind (the player's start) | `wind.spell.yaml` | contact hook | — | push | shape **orb**, element **wind** |
| Fire (caught from falling flakes) | `fire.spell.yaml` | contact hook | — | burn | shape **wall**, element **fire** |
| Watchtower | `watchtower.spell.yaml` + `watchtower.lua` | **creature in my shadow** (each moon throws its own) | a Luau brain: anyone there? | **slow** | fed by the town's mana lines |
| Climber (purple brood) | `climber.spell.yaml` | — | — | goes for the seed, **climbs** fences (half speed) | health 4, vanishes at zero |
| Digger (orange brood) | `digger.spell.yaml` | — | — | goes for the seed, **digs** under fences (unseen there) | health 3, vanishes at zero |
| Town seed | `town-seed.spell.yaml` | bodies touching it | — | loses health for each one | health 24; at zero, **fall: the town is gone** |

The town's layout is in `spells/town.toml`: where the seed, hero and buildings stand, the fences, where the broods rise
and how fast the town refills its buildings. It matches the moon town's own coordinates.

## Cast-order fusion

The first spell cast gives the **shape** and the second gives the **element**. The first's element rides along as a
lesser effect.

| Cast | Shape (first's) | Element (second's) | Rides along | Result |
|---|---|---|---|---|
| fire → wind | wall | wind (push) | fire (burn) | **Wall of Flame** |
| wind → fire | orb | fire (burn, burst) | wind (push) | **Fireball** |
| fire → fire | wall | — | — | **Hearthglow**, a dud: a warm glow that harms nothing |
| anything else | first's | second's | first's | "Element Shape" (Frost Bolt), or the model's name |

The parts are closed sets and Rust builds the spell, so a fusion is always valid.

On-device fusion (`item-synthesis`, `engine:fuse_spells(first, second)`) sends the pair to the model with
`fusion::schema`. The schema allows only the cast order's shape and element, the carrier or nothing, and a 3–20 letter
name. `fusion::build` checks the choice and builds the spell. Known pairs are named in `spells/fusion.toml`.

## The town at work

`spellcraft.town(spec)` steps the town to the scene's time at 20 ticks a second. The spec carries the moons'
directions (from the World's environment) and the fire falling (from the weather). It returns what the HUD shows.

- Creatures rise while their moon is up, every 4 s, at most 6 at once.
- The watchtower's shadow runs away from each moon, `height × horizontal / vertical` long, up to its reach.
- Bolts and orbs home in on the nearest creature; a wall stands 2.5 m out across the creature's way.
- The town is a pure function of time: the same moons and casts give the same night. Time going back, or a change to
  any file in `spells/`, starts the town over.
- **Play keys:** 1–4 select a slot; space (or E) casts the selected spell. A cast arms it for 1.2 s ("FIRE → ?"):
  cast a second within that time and the two fuse in cast order, the armed one first; otherwise the armed spell goes
  alone. Casts aim at the nearest creature (the town's hero has no cursor or facing yet). In the editor's preview,
  not playing, the town casts on its own every 8 s so a still town still shows fusions.
- **Naming on this machine:** each fusion starts `engine:fuse_spells(first, second)`, which asks the local model
  with the fusion schema. The cursor blinks for at least 350 ms; at 1.5 s the fallback name is stamped; when the
  model's name arrives it swaps in quietly (`spellcraft.rename`) and becomes the spell's title. Measured on the local
  engine (Qwen3.5-9B): 1.4–3.4 s, names such as "Blazing Curtain" and "Gale Rampart" (fire→wind), "Whirling Ember"
  (wind→fire), "Warm Glow" and "Zephyr" (duds).

## The HUD's real state

The `Hud` node with `moment = live` reads the town:
- the seed's health (and its hit flash);
- the slots (wind, plus fire once caught);
- the fusion slot and the cast-order row with the fused name typing in;
- the brood (each creature a pip of its health in its moon's colour; a digger underground is hollow);
- creature marks and hit numbers placed in the world when the camera is fixed (the ink town);
- the watchtower's card (sensor, effect, mana spent) when its spell acts;
- "THE SEED BREAKS · THE TOWN IS GONE".

Through the pixel lens the camera follows the hero, so world marks wait for the scene to draw the creatures (renderer
lane) and the brood readout stands in.

In the editor, not playing, the town is shown as it stands 24 s in (`preview`). The fixed `moment` values (fusion,
alignment, building, danger, dawn) stay for stills.

## Tests (`cargo test -p component-spellcraft`, `-p component-item-synthesis spells`)

- The shipped spells are valid; the watchtower reads "Creature in my shadow" → "Slow".
- The validator refuses a forward reference, a container over capacity, and a param out of bounds.
- Fusion follows cast order: Wall of Flame, Fireball, the Hearthglow dud, Fire Bolt for an unnamed pair.
- A model's choice is checked and built by Rust; the schema offers only the legal parts.
- Mana meters sensing, logic and effects:
  - a looping brain spends more than a lean one;
  - work units are deterministic;
  - with too little mana the instance is exhausted, nothing acts, and every unit is accounted for;
  - a sensor that finds nothing makes the effect sleep for free.
- Creatures rise under their moon and the seed breaks: the town is gone, and nothing more happens. With no moon up the
  seed holds.
- The watchtower slows a creature in its shadow and pays for it; fire is caught from the flakes; a Fireball hits.

## Spell effects in the world (task 75, first pass)

`SpellFx` (packages/spellcraft, a World part) draws every live cast from the real town: the town's formed bodies
(each with its look: wind orb, fire wall, wall of flame, fireball, dud) and its moments (burst, hit, knockback, the
dud's warm puff). Knockback is a real shove that dies away in the town; impacts shake the camera a little in Play
(`runtime.shake`). `spellcraft-reel` renders the effects offscreen from a script of Play keys, each still keyed to the
event it shows (`spellcraft-reel scenes/town-ink.scene.luau player out --weather snow`; the pixel town with `--look
pixel --carry-fire --frame-hero`). One town per scene serves every reader (`with_town`, `town_at_time`).

Not yet at the studio bar. Three fresh art-director rounds, images only:

| Round | Reds | What changed after it |
|---|---|---|
| 1 | 8 | wind became a swirl; knockback ring and kick; fireball small and hot with a trail; impact flash, crown, embers; dud at the hands; spells start clear of the caster |
| 2 | 10 | stills keyed to the real events (most reds were beats one step early); flames without stripes; crest; longer smoke; wind core; smoother dud |
| 3 | 8 | (open) fire walls read as traffic cones; wall of flame as crystal; knockback too faint; fireball caught after impact; pixel effects tiny at the wide camera, pixel dud far from the visible villager |

The lesson: toon-shaded meshes, banded by the looks' shading, will not read as fire. Next: draw the spells' fire, smoke
and wind through the engine's particle graphs (as the weather's firefall and the bomb's explosion are), with SpellFx
emitting particles from the same town state; keep the meshes only for silhouettes (the wall's line, the orb's swirl).
