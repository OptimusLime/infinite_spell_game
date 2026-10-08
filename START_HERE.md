# Infinite Spell Game: start here (updated 2026-10-07 evening)

## The thrust
Paul is building a game and, at the same time, the tools to make it and the video about it, all inside the Atelico app engine (Rust + Luau, hot reload, Claude in rmux panes). The game has two celestial cycles (two moons, or a sun and a moon) at different frequencies. Their forecast overlap makes the weather and decides which monsters rise from the ground. The world has two looks: a cosy 3D pixel-art town and a frozen, cel-shaded ink "other side". You plant a town seed anywhere in an infinite world and defend it. Everything (spells, buildings, NPCs, towns) is a spell made of sensors, effectors and an action, and spells and buildings fuse. The first deliverable is the AI game dev "day in the life" video, sketched in the engine's own Storyboard tab, and built from engine parts (the marketplace in use).

## Paul's words (verbatim; don't make him repeat them)
- Two cycles: "I fundamentally disagree with the idea that we're going to be moving away from a two cycle thing and actually my idea was to lean into it a little bit more"
- Moons: "the concept is a little bit more like forecasting the moons. And so we have different visual effects for different moons. And when the moons align, that's when we end up with things." / "it's kind of like there's two systems and their forecasting and the combination of them create the environment"
- Weather: "the weather is kind of a function of those objects ... it's raining fire in a snowy time" / "I'm really into weather systems"
- Enemies: "enemies at night that come out of the ground or enemies that come out depending on the color of the moon"
- Town seed: "a town is really about setting kind of the heart or the seed of the town ... if that seed gets blown up, then everybody dies" / "the good guys are frozen in place. And so that means that ... your things are vulnerable"
- Spells: "everything is quote-unquote the spell system so the town is a spell system the npc is a spell system and when i mean spell system i mean sensors effectors and an action and even the buildings themselves can potentially have defenses and so they are potentially combinable"
- Fusion: "i already told you item fusion was my gut institnct ... I will continue to suggest it is the KEY feature, but we have to work up to it. First, we need something visually interesting, THEN we need item fusion to sit on top of it."
- Collaboration: "from the get-go we really we need this object to be a collaborative object"
- Video in the engine: "I want to stop with this thing where I'm basically using an HTML server to display some of my videos. And we're going to build our own UI that does the video part inside of the object."
- Storyboard: "We will have cardboard 'Cards' that represent all our connected linear video ideas. But the cardboard cards might come from a group of underlying concepts. there is the pixel world, the cel world, weather stuff. and we want to have somethign like a slideshow presentation of all the cards. That will represent our sketch of the video. It will form a sort of checklist for video creation agent."
- Nesting: "You should be able to represent added components through subfolders of other folders, so we should be able to say the path, or the group path and then the component path."
- Both at once: "I need to construct the helper to make the concept, and at the same time, we are creating and feeding the conecpt itself"
- Camera: "start in that view, and then pan from the cineamtic view to the 'player view' and then start moving around the town. Again, we are looking for 'iconic' moments"
- Whiteboard: "the subtle black and other stuff is proably not as effective as 100% opacity objects with a littel more brightness in the organization. I do like the nested concepts though"
- Editing: "you would think that there would be an 'edit' button that popped open the AI bar for me to talk about editing that specific thing"
- Claude button: "I would expect the button itself to be fairly consistent at different screen sizes and not propotion, even if the thing it open IS propottional"
- Sessions per window: "someone says they use X number of sessions to focus on specific windows/tabs in their own aigamedev workflow, we shouldh ave that functionality too!"
- Agents: "only these two agents, make sure they dont collide too much, and lets make sure we are commiting and pushing where appropraite."
- Research: "use AGENTS TONDO THISNWORK why the fuck are you doing it yoursled and not coordianteng ... 10 fucking web aearche sis that a fucking joke"
- Judging: "send it to a NAIVE AGNET tell them not to read ANYHTING other than your messages, and then react and attempt to reconstruct your point EXACTLY" / "i need red-to-green refactors here ... You can't be changing till you see the reds"
- AI engine: "do not commit videos to ai-engine. basically commit nothing to ai-egnien without my sayos. app engine you can modify thx."
- Hand-offs: "i dont watn to lose your knowledge of whats going on to poorly written slop"

## The queue (source of truth for all work)
Every task, owner, status, commit and note lives in `queue/queue.db` (SQLite). Read and change it only through `python3 queue/q.py` (`list`, `show ID`, `add`, `set`, `note`); `queue/queue.md` is a readable snapshot. Agents set their own tasks to doing/review/blocked; the coordinator checks and marks done. Nothing exists only in chat.

## More of Paul's words (2026-10-07 afternoon)
- Agents: "what the heck, are you not using agents to do this work? didy ou read anything about how we work? my verbatim comments?"
- Screens: "LOOK AT A FUCKING SCREENSHOT OF A FUCKING SINGEL FUCKING THING AND YOU SHOULD SEE THIS SHIT" (use real `screencapture`, not `app shot`)
- Meter: "just say teh FUCKIGN frames per second ... FUCKING CONDENSE IT"
- Screenplay: "It needs to be a fuckknng SCREENPLAY ... you are about to get a fucking BUDGET on what you allow on this screen, and you need to JUSTIFY THAT FUCKING BUDGET"
- Looks: "it looks WORSE than the pixel world we could isntall from the plugins ... did you start from scratch" / "shadows that fight looks TERRIBLE"
- Renderer: "The only way to code this is the right way." / "the backend for the UI should be swappable" / "1/3 of the FPS being A COPY OPERATION. how is that not a total red flag?"
- Decisions: "please stop coming back to me for nonsense approvals ... If you need to make irreversible decisions, then come to me, otheriwse ... make good choices and document"
- Game UI: "why would the forecast panel be in the editor? thats part of the game bud." / "We are missing in game UI elements, which is why the game looks so baren and werid."
- Queue: "organize into some type of queue thats more resilient and organized like on device sql or something you read from" / "i am tired of your hshit dropping"

## Rules added today
- Agents test only on `atelico-host --hidden` hosts; never open windows on Paul's editor (port 7878) or send OS mouse/keyboard events.
- Shared files: each agent stages only its own hunks (`git apply --cached`); never `git stash`.
- Renderer: Skia Ganesh on wgpu's Metal device behind a swappable cpu/gpu backend (docs/perf/render-backend.md, docs/perf/research/recommendation.md in the engine).

## How to work (rules learned this session)
- Research = several agents in parallel, one lane each, 30+ sourced searches, written to files. Then synthesise.
- Concepts and storyboards go through clean judges (no files, no memory, all instructions in the prompt): two naive players plus senior devs (juice/game feel, systems, art director). Their complaints become a numbered 🔴🟡🟢 issue table; edit only to turn reds green; new judges each round.
- Few agents, one lane each, separate files; only one agent restarts Paul's editor; renders take `.atelico/recording.lock`.
- Always full absolute paths, a colour-coded checklist, short plain answers. Git: branches, normal commits and merges, never force-push or rebase, main only via PR.
- Also read `/Users/paul/coding/atelico/atelico-app-engine/docs/working-with-paul.md` and the memory index `~/.claude/projects/-Users-paul-coding-atelico-atelico-applets/memory/MEMORY.md`.

## Where things are
| What | Path |
|---|---|
| This repo (game, storyboard, sketches) | `/Users/paul/coding/infinite_spell_game` (branch `bootstrap`) |
| Engine (all code, packages) | `/Users/paul/coding/atelico/atelico-app-engine` (branch `universal-ai-app-engine`) |
| Spell concepts source | `/Users/paul/coding/creatures/creature_3d_studio` (crates/spellcraft; twin moons in crates/studio_core/src/day_night.rs) |
| Research | `docs/research/` here (creature studio, engine, weather and moons); `/Users/paul/coding/atelico/atelico-app-engine/docs/community/mechanics-research/` (two-loop, merge/fusion, juice, tower defense, chance, viral, two-worlds and spellcraft) |
| Concept judging trail | `/Users/paul/coding/atelico/atelico-app-engine/docs/aigamedev-video/core-loop/` (Jar Castle rounds 1–6; Lantern Side rounds 1–4) |
| Storyboard cards | `storyboard/<group>/<card>.toml`, order in `storyboard/path.toml`, exports in `storyboard/exports/` |
| World sketches | `storyboard/sketches/world/` (cinematic-sky.png, player-view.png, pan.mp4, pixel-night-two-moons.png, ink-dusk.png, switch-pixel-to-ink.mp4, moon-alignment.png) |

## Running it
- Editor: `cd /Users/paul/coding/atelico/atelico-app-engine && ./target/debug/app run` (port 7878). It runs the engine's own project; `app init` on this repo stopped because 3D packages aren't in the local registry (publishing needs `app registry serve`, and node-graph, scene-3d and item-synthesis fail to publish). This repo's `atelico.toml`, `app/`, `editor/`, `scenes/` are that partial scaffold.
- Storyboard: open the "storyboard" window (form `forms/storyboard.toml`); until the editor loads cards itself, run `./target/debug/app --port 7878 storyboard --watch`. Export: `app storyboard export screenplay|html|yaml|all`.
- Moons: `moon-forecast --seed N` (packages/sky). Camera moves: `camera-paths/*.path.toml`, `render-path`.

## State of the work
| # | Item | Status |
|---|---|---|
| 1 | Twin moons in `packages/sky` (purple period 1.0 tilt 30°, orange 0.8 tilt 15°), seeded schedule, alignment query | 🟢 engine c115134 and earlier |
| 2 | Pixel world and ink world looks (packages pixel-world, ink-world, moon-town), wipe between them | 🟢 but they are scene settings, not shader-forge graphs; ink "paper" is weak |
| 3 | Cinematic camera: low sky shot, crane down to isometric, follow the hero (packages/camera-path) | 🟢 slight shift at the snap; hero too small |
| 4 | Storyboard tab: light whiteboard, nested groups, linear path, Board / Screenplay / Slideshow, Edit opens Claude with the card's context, exports | 🟢 engine 47e02bf, game 0557f76 |
| 5 | Editor performance round 1: host no longer pins a core (99% → 11%) | 🟢 engine 994e881 |
| 6 | Perf round 2 (agent running, uncommitted in crates/editor, crates/host: damage.rs, meter.rs, present.rs): partial repaints, honest meter, fixed-size Claude button, storyboard loaded by the editor itself | 🟡 |
| 7 | Bugs Paul hit, queued to the perf agent: main window (boats scene) wraps ~200 px after resize; scrolling broken (Screenplay, maybe everywhere); 77–129 ms per frame; console error "editor: render: runtime error store path parts are strings, got nil" | 🔴 |
| 8 | World round 2 (agent running, Paul said go): a real town instead of the island, a bigger hero, moon periods that drift (alignments now every 4 days at the same hour), a second shadow map so each moon casts its own coloured shadow, the forecast UI | 🟡 |
| 9 | Several Claude sessions per project, one per window/tab | ⚪ after perf |
| 10 | Spell core: creature studio `crates/spellcraft` (decided 2026-10-07; mana + Lua brains); port it into the engine and point item-synthesis fusion at its format | ⚪ next |
| 11 | Collaboration server (Rust, websockets, persistent state; start from creature studio's Yrs + coordinator design) | ⚪ |
| 12 | Duplicate cut files `talks/vc.cut.toml` and `docs/vc-conversation/vc.cut.toml` load into the same place; fixed: the deck's cut is now `talks/vc-deck.cut.toml` (engine 55b5302) | 🟢 |

## Concept so far (input, not spec)
Judged best across rounds: the town's buildings cast working shadows on the dangerous side (a lamp becomes a beacon, a watchtower a turret, the bakery a healing shrine); the frozen ink dusk ("the baker caught mid-wave"); fusion by cast order (first spell gives the shape, second the element; no menu); a glowing leash to the nearest beacon that snaps and spills your light. Every judge: prototype one street, 3 shapes × 3 elements, 4 buildings, one tileset with an ink shader, a week-6 kill test.
