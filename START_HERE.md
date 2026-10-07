# Infinite Spell Game: start here

A game about two moons (or a sun and a moon) on different cycles. Their overlap, forecast ahead of time, makes the weather and decides which monsters rise from the ground. You plant a town seed anywhere in an infinite world and defend it; during the dangerous phase your townsfolk freeze. Everything is a spell: sensors + effectors + an action. That covers spells, buildings, NPCs and towns. Built inside the Atelico app engine (Rust + Luau, hot reload, rmux Claude panes). Collaborative from day one.

Read first:
1. Paul's words: the 2026-10-07 brief (summary and transcript) is in the engine repo session notes; the memory file is `~/.claude/projects/-Users-paul-coding-atelico-atelico-applets/memory/infinite-spell-game.md`.
2. `docs/research/creature-studio-digest.md`: the spell system (crates/spellcraft, YAML phases, mana-metered Lua), the existing twin moons (purple period 1.0, orange 0.8, in `creature_3d_studio/crates/studio_core/src/day_night.rs`), collaboration (Yrs drafts, Cloudflare coordinator).
3. `docs/research/app-engine-digest.md`: `app init`, the sky/day-night packages (one moon, no weather), items + item-synthesis, sync (single machine), rmux sessions, in-engine video playback (no audio).
4. `docs/research/weather-and-moons.md`: Icarus and others; tides as the model for two cycles; 10 sun×moon / moon×moon combinations; a forecast UI sketch; clip hooks.
5. Earlier concept judging (Lantern Side): `/Users/paul/coding/atelico/atelico-app-engine/docs/aigamedev-video/core-loop/`. Loved by every judge: buildings that cast working shadows onto the dangerous side, frozen townsfolk, fusion by cast order, a leash to the nearest beacon. Input, not spec.

## Components

| # | Component | What it does | Starts from |
|---|---|---|---|
| 1 | Twin sky + time | Two bodies with their own period, tilt and colour; a seeded, deterministic schedule; alignments (like spring tides) | Engine sky/day-night package + creature studio moon orbit maths and colour table |
| 2 | Forecast | Computes the next N alignments and what they bring (weather, enemy type, danger window) | New; pure function of the schedule |
| 3 | Forecast UI | Two cycles shown as rings or a timeline, with the next overlaps and their warnings (sky, sound, UI in steps) | New; weather research §forecast UI |
| 4 | Weather | (body A state, body B state) → weather: snow, fire rain in snow, storms… Particles, shader graph, cel shading, one style | Engine particles, fire, water, cel-shading, shader-forge |
| 5 | Spell core | Sensors, effectors, actions, mana; one format for spells, buildings, NPCs and towns | DECISION: engine `crates/items` (TOML steps, Rust physics) vs creature studio `crates/spellcraft` (YAML phases, metered Lua, Rapier) |
| 6 | Fusion | Combine two spells (cast order or a bench); a model picks from fixed choices, Rust builds the result; results cached per pair | Engine `item-synthesis` (one strap-on mode today) |
| 7 | Everything is a spell | Buildings, NPCs and the town seed are long-lived spell instances; buildings can combine | New, on 5 |
| 8 | Town seed | Plant a seed anywhere; build around it; if it's destroyed, everyone dies; townsfolk freeze in the danger phase | New |
| 9 | Enemies | Rise from the ground by moon colour and weather; hunt the seed and its buildings | New, on 5 and 2 |
| 10 | Infinite world | Chunked terrain around wherever the seed is planted | Engine terrain / world packages |
| 11 | Collaboration server | Rust backend, websockets, persistent state; one trusted simulation host; drafts merge | Creature studio design (Yrs + coordinator); engine store/sync |
| 12 | In-engine review | Watch prototypes and videos inside the engine, not the HTML hub; audio later | Engine video packages, editor-video-viewer |

## Several sessions on one project
One lane per session, separate files, separate port, one lock per shared resource (`.atelico/recording.lock` for renders). Proposed lanes:

| Lane | Owns | Port |
|---|---|---|
| A sky | components 1–3 (`packages/twin-sky`, `packages/forecast`) | 8711 |
| B weather | component 4 (`packages/weather`) | 8712 |
| C spells | components 5–7 (`crates/spells` or `packages/spells`) | 8713 |
| D town | components 8–10 (`packages/town`, `packages/enemies`) | 8714 |
| E server | component 11 (`server/`) | 8715 |
| F video | component 12 + storyboards (`packages/review`, `docs/video/`) | 8716 |

## First steps (in order)
1. ⚪ `app init ~/coding/infinite_spell_game` from the engine; make the 3D packages available (publish to the local registry or point at the engine's `packages/`).
2. ⚪ Paul decides the spell format (component 5).
3. ⚪ Lane A: two moons in the sky with the creature studio periods; a forecast of the next alignments printed and drawn.
4. ⚪ Lane B: three weather states from the combination table, one style.
5. ⚪ Lane F: first clip storyboard ("the forecast says fire will rain on the snow tonight"), tested with clean judges (red to green) before any render.
6. ⚪ Lanes C–E start once 1–2 are done.

Git: work on branches, normal commits and merges, no force-push or rebase, main only via PR.

## Scheduled (Paul, 2026-10-07)
- After the editor performance fix: several Claude sessions per project, each focused on its own window or tab (as AI game devs run "a session per window"). Needs a look at the host/agent backend (crates/agent, crates/host) first.
- In progress: world shading (3D pixel world, cel-shaded anime world, day/night with two coloured moons) and a storyboard tab (cards from concept groups, linear path, slideshow) in the engine repo; storyboard content lives in this repo under `storyboard/`.
