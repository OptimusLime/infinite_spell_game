# Queue

Snapshot of queue.db, 2026-10-08T10:58:33. Edit through `app queue` (or `queue/q.py`), not this file.

Object prerequisites describe planning order. Task dependencies control claims. Recorded done is not new user acceptance.

## Atelico app engine (`engine`)

27 open / 51 total · 8 doing · 12 review · 24 done · 1 ready todos

The native host, shared runtime and editor capabilities. Tasks retain their recorded status; done does not imply Paul accepted the result.

### Claude and Codex sessions (`engine.sessions`)

4 open / 4 total · 0 doing · 3 review · 0 done · 0 ready todos

Provider-neutral embedded rmux sessions, routing, visibility and process lifecycle.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 26 | 🔵 review | One Claude session per window/tab | queue-agent |  |
| 94 | 🔵 review | Support Claude and Codex in the same native session interface | codex-provider |  |
| 62 | 🔵 review | Shell commands in a session can write outside its owned files (only instructions stop it) | queue-agent |  |
| 61 | ⚪ todo | Terminal per window: host paints each window's Claude session into that window's agent surface (sessions, task 26) | renderer-agent |  |

### Build, integration and verification (`engine.build`)

4 open / 11 total · 3 doing · 1 review · 7 done · 0 ready todos

Consolidation, repository hygiene, verified checkpoints and proof. Existing done statuses remain historical agent reports.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 91 | 🟡 doing | Shared build broken: paint-skia gpu.rs texture() signature mismatch (renderer step 5) | renderer-agent |  |
| 93 | 🟡 doing | Consolidate engine lanes and preserve all worktree/checkpoint history | codex-coordinator |  |
| 52 | 🟡 doing | Quality gate: nothing to review without a real screencapture of Paul's window checked against a written bar | coordinator |  |
| 34 | 🔵 review | Engine PR #2 and game PR #1 (bootstrap -> main): keep current, merge when Paul says | paul |  |
| 72 | 🟢 done | Shared build broken: component_creatures unresolved in scene-3d | weather-agent |  |
| 46 | 🟢 done | DISK 100% full (20 GB free and falling): shared scratchpad copies — screenproj 158 GB, engine-copy 59 GB and growing (likely a copy including .atelico, which is 82 GB), sbproj-atelico-partial 21 GB | coordinator |  |
| 63 | 🟢 done | Shared build broken: duplicate component-spellcraft package | game-ui-agent |  |
| 82 | 🟢 done | world.wgsl fix uncommitted: committed HEAD can't draw the town | shader-graph-agent |  |
| 42 | 🟢 done | Find who switched Paul's main window to the Game Engine form and dropped the Storyboard tab (restored by coordinator) | coordinator |  |
| 31 | 🟢 done | Engine working tree: tracked proof PNGs show as deleted (proof/node-graphs/...) — restore or explain | coordinator |  |
| 32 | 🟢 done | Delete 18 GB abandoned project copy in scratchpad (sbproj-atelico-partial) |  |  |

### Renderer and performance (`engine.renderer`)

4 open / 9 total · 2 doing · 1 review · 5 done · 1 ready todos

Native GPU rendering, parity and frame budget.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 22 | 🔵 review | GPU renderer step 3-4: Skia CPU then Skia Ganesh on wgpu Metal; [host] renderer = cpu\|gpu, runtime switch | renderer-agent |  |
| 23 | ⚪ todo | GPU renderer: identical-pixels test both backends + 8 ms frame-budget test | renderer-agent |  |
| 24 | 🟡 doing | GPU renderer step 5-7: port components, 3D post effects to GPU (render-3d split with shader agent), gpu default | renderer-agent |  |
| 25 | 🟡 doing | Automatic test runs (CI): no Mac GPU runner; interim = renderer agent runs tests before each push | renderer-agent |  |
| 50 | 🟢 done | Paul's Town tab at 16 fps: profile it and get it under 8 ms (pixel upscale/particles/HUD to GPU) | renderer-agent |  |
| 20 | 🟢 done | GPU renderer step 1: wgpu surface present + 3D texture sampled, no readback, shader warm-up | renderer-agent |  |
| 21 | 🟢 done | GPU renderer step 2: neutral display-list interface, tiny-skia backend, zero pixel diff | renderer-agent |  |
| 37 | 🟢 done | Editor perf patch rounds (damage, caches, meter, scroll, one window, storyboard tab) |  |  |
| 41 | 🟢 done | Renderer research (4 lanes + recommendation: Skia Ganesh on wgpu Metal) |  |  |

### Shared runtime and layout (`engine.runtime`)

1 open / 8 total · 0 doing · 1 review · 7 done · 0 ready todos

Scene state, incremental layout, polling and retained rendering.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 90 | 🔵 review | Host type list misses SpellFx and Creatures component types (can't place them in scenes) | game-ui-agent |  |
| 57 | 🟢 done | Town frame: scene script rebuilds its full 831-sprite item tree to JSON every frame — make it incremental | runtime-agent |  |
| 58 | 🟢 done | Town frame: HUD and scene layouts rebuilt from scratch every frame — reuse layout when unchanged | runtime-agent |  |
| 59 | 🟢 done | Town frame: light probes re-baked every frame because walkers carry lights — bake only on change / dynamic lights separately | runtime-agent |  |
| 60 | 🟢 done | Editor file poll (dir scan, cut list) blocks the main thread for tens of ms — move off main thread | runtime-agent |  |
| 86 | 🟢 done | A HUD text changing every frame costs only its label (ticking clocks, countdowns) | runtime-agent |  |
| 81 | 🟢 done | Layout: a per-frame HUD text change rebuilds the whole kept layout (~20 ms); relayout only the changed leaf when its size is unchanged | runtime-agent |  |
| 43 | 🟢 done | scene-3d particle-graph test fails when run in parallel | runtime-agent |  |

### Editor interaction (`engine.editor`)

0 open / 2 total · 0 doing · 0 review · 2 done · 0 ready todos

Shared editor interaction correctness.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 29 | 🟢 done | layout_matrix: welcome-modal 4 px spill | runtime-agent |  |
| 30 | 🟢 done | button_press test failing (cause untraced) | runtime-agent |  |

### Video editing and playback (`engine.video`)

3 open / 4 total · 0 doing · 3 review · 1 done · 0 ready todos

Native cut playback, titles, music, exports and project routing.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 80 | 🔵 review | Engine video: play a cut from another project in the editor (cut paths are project-relative; game repo has no editor-video-viewer or [settings.ai]); reel must be playable in Paul's editor | runtime-agent |  |
| 78 | 🔵 review | Engine video: cut format lacks a sized lower-third title (label chip only), a black/fade-out shot, and any music bed | runtime-agent |  |
| 92 | 🔵 review | Engine video (044b965): title on a recording shot crashes: work dir not created before brand.title_at runs magick (needs create_dir_all(&work) before crates/cli/src/video/mod.rs:375); also the [music] bed is mixed mono into the voice track | runtime-agent |  |
| 28 | 🟢 done | Video playback inside the engine (no HTML hub) | runtime-agent |  |

### Storyboard editor (`engine.board`)

1 open / 2 total · 0 doing · 1 review · 1 done · 0 ready todos

Reusable board navigation, groups, planned paths and native presentation.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 95 | 🔵 review | Restore storyboard groups, native wires and whole/focused board navigation | codex-board |  |
| 38 | 🟢 done | Storyboard UI under design budget; screenplay; play-order board |  |  |

### Hierarchical work tracker (`engine.tracker`)

2 open / 3 total · 1 doing · 1 review · 1 done · 0 ready todos

Objects, execution prerequisites, activity and searchable history in the existing queue.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 96 | 🟡 doing | Turn the existing queue into hierarchical objects, prerequisites and searchable work history | codex-tracker |  |
| 73 | 🔵 review | Queue: urgent flag, severity, impact tags, due, dependency weight, computed rank; query/sort CLI; next honours urgency; Queue tab urgent strip | queue-agent |  |
| 35 | 🟢 done | Queue in the editor: a Queue panel that reads queue/queue.db | queue-agent |  |

### Collaboration (`engine.collab`)

1 open / 1 total · 1 doing · 0 review · 0 done · 0 ready todos

Peer synchronization of authoritative shared editing state. Derived queue/storyboard views remain local.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 27 | 🟡 doing | Collaboration server (Rust, websockets, persistent state) | collab-agent |  |

### Registry and publishing (`engine.registry`)

3 open / 3 total · 1 doing · 1 review · 0 done · 0 ready todos

Registry install/publish workflows and publication blockers.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 84 | 🔵 review | scene-3d: 4 failing tests block publishing it (and so every world part): the face stays in the lit band; the followed hero keeps every outline and eye pixel; a particle graph draws the same at the same moment; the Luau and manifest lists miss SpellFx in component.toml data_types | weather-agent |  |
| 67 | 🟡 doing | Registry publish failures: node-graph lockfile collision, scene-3d and item-synthesis failing tests block publishing | queue-agent |  |
| 66 | ⚪ todo | Marketplace install flow for card 2: Claude installs the 5 world parts from the registry live in the editor (app registry serve, publish) | queue-agent |  |

### Generic inventory (`engine.inventory`)

1 open / 1 total · 0 doing · 0 review · 0 done · 0 ready todos

Shared item identity, container/grid capacity, movement and expansion primitives. No mana-specific container API.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 98 | ⚪ todo | Create a reusable generic grid inventory and stable item identities |  |  |

### Generic equipment (`engine.equipment`)

1 open / 1 total · 0 doing · 0 review · 0 done · 0 ready todos

Shared equipment slots and equipped item references, independent from carrying inventory and mana resource policy.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 99 | ⚪ todo | Create reusable equipment slots independent from carried inventory |  |  |

### Tooling review and acceptance (`engine.review`)

1 open / 1 total · 0 doing · 0 review · 0 done · 0 ready todos

Implementation and technical validation precede a separate review of the working workspace with Paul. Content waits for explicit human acceptance.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 103 | ⚪ todo | Review current consolidated workspace, sessions and grouped task DAG with Paul | paul |  |

### Reusable concept and reference board (`engine.references`)

1 open / 1 total · 0 doing · 0 review · 0 done · 0 ready todos

A reusable comparison/review workspace for reference inputs, concept revisions and compatibility decisions across projects.
Planned after: engine.tracker

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 107 | ⚪ todo | Build reusable concept/reference-board review and comparison tooling |  |  |

## Shared applets (`applets`)

2 open / 2 total · 0 doing · 2 review · 0 done · 0 ready todos

Reusable components and UI kits consumed by game projects.

### Shared layout (`applets.layout`)

1 open / 1 total · 0 doing · 1 review · 0 done · 0 ready todos

Layout/text-measure caching and shared applet UI foundations.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 33 | 🔵 review | atelico-applets PR #27 (layout text-measure cache): merge decision | paul |  |

### Applet showcase (`applets.showcase`)

1 open / 1 total · 0 doing · 1 review · 0 done · 0 ready todos

Consolidated shared applets and preserved historical showcase variants.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 100 | 🔵 review | Consolidate shared applets and preserve historical showcase variants | codex-coordinator |  |

## Infinite Spell Game (`game`)

23 open / 42 total · 3 doing · 8 review · 18 done · 6 ready todos

The game and its physical spell rules. Paul defines the design; reel suggestions are not approved mechanics.

### Game HUD and UI kit (`game.hud`)

1 open / 6 total · 0 doing · 0 review · 5 done · 1 ready todos

Currently engine/packages/game-ui consumed by the game. Reuse of UI primitives is distinct from game-specific resource semantics.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 56 | ⚪ todo | Ink HUD: spell slot key numbers still tiny | game-ui-agent |  |
| 10 | 🟢 done | In-game UI spec docs/game-ui.md (seed ring, moon clock, spell slots, prompts, wave counter...) judged | game-ui-agent |  |
| 11 | 🟢 done | packages/game-ui kit in both looks, placed in the moon town scenes | game-ui-agent |  |
| 53 | 🟢 done | HUD bugs: stub 'Night 3' vs schedule day 30; ink chip says align in 4h40m while moons aligned; ink fire icon reads as water drop; spell bar touches bottom edge | game-ui-agent |  |
| 55 | 🟢 done | Ink HUD type too small at editor zoom (0.65): labels like ENDS 2H 0M barely readable | game-ui-agent |  |
| 47 | 🟢 done | Game UI: ink panels still read 'like a website' — brush display font; real data for seed/spells/creatures (stubs today) |  |  |

### Design and reference decisions (`game.design`)

1 open / 2 total · 1 doing · 0 review · 1 done · 0 ready todos

Confirmed rules and their source references; proposals and unresolved choices remain explicitly identified.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 97 | 🟡 doing | Consolidate original game and Creature references into an explicit game-design source of truth | codex-coordinator |  |
| 36 | 🟢 done | START_HERE.md: update with today's state and point to queue/ | coordinator |  |

### Town and world structure (`game.world`)

2 open / 2 total · 0 doing · 1 review · 0 done · 1 ready todos

Town layout, seed planting and shared scene structure.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 69 | 🔵 review | Seed planting moment: plant the seed, first houses grow around it (card 3) | weather-agent |  |
| 79 | ⚪ todo | Engine: pixel town and ink town have different layouts, so the same camera does not line up across the pixel->ink cut; ink look reads sepia not two-tone purple/orange | shader-graph-agent |  |

### Pixel and ink/cel looks (`game.looks`)

6 open / 12 total · 1 doing · 3 review · 5 done · 2 ready todos

Readable lighting, shadows, authored shader looks and the switch between them.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 51 | 🔵 review | Changing screens: the live switch between the pixel town and the ink world (dusk flip) in the town scene | shader-graph-agent |  |
| 54 | 🔵 review | Ink town in Paul's window looks foggy/washed sepia and soft (vs crisp cel sketches): fix the look | shader-graph-agent |  |
| 76 | 🔵 review | Engine: no second visible moon shadow in town scenes (second_shadow=1 shows nothing in pixel town or ink yard); reel shot 'two shadows' needs it | weather-agent |  |
| 89 | ⚪ todo | Night grade too flat: darker blue night ground, warm light pools under windows/lamps/fire impacts, cool rim on characters (ink look part, cel-world graph, pixel Sky palette) | shader-graph-agent |  |
| 88 | ⚪ todo | Moon-town seed plinth reads as a layer cake with noisy pillars; cel 10:00 grass murky with no sun shadow | shader-graph-agent |  |
| 3 | 🟡 doing | Hero contrast against the dark path (size done) | shader-graph-agent |  |
| 48 | 🟢 done | Buildings float: base shading/shadow doesn't touch the ground (both looks) | shader-graph-agent |  |
| 49 | ⚫ dropped | Houses float: contact shading/shadows offset from building bases (pixel + cel town) | shader-graph-agent |  |
| 1 | 🟢 done | Pixel-art look as a shader graph (graphs/pixel-world), live preview, parity judge | shader-graph-agent |  |
| 2 | 🟢 done | Cel/anime look as a shader graph (graphs/cel-world), live preview, parity judge | shader-graph-agent |  |
| 4 | 🟢 done | Continuous crane opening: sky -> crane -> player view -> hero walking (pixel town drawable in perspective) | shader-graph-agent |  |
| 40 | 🟢 done | World round 3: pixel and cel rebuilt on plugins; one key shadow |  |  |

### Moons, weather and forecast (`game.weather`)

2 open / 5 total · 0 doing · 2 review · 3 done · 0 ready todos

Deterministic moon-driven weather and visible consequences.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 77 | 🔵 review | Engine: weather gives embers only in daylight (no orange-moon night embers, no heat haze at night); reel shot 'embers' needs night embers | weather-agent |  |
| 6 | 🔵 review | Weather visuals as graphs in both looks (weather-snow/embers/firefall), ground/roof accumulation | weather-agent |  |
| 5 | 🟢 done | Weather state as a function of the two moons (snow / embers / fire-in-snow + steam), seeded, tested | weather-agent |  |
| 7 | 🟢 done | In-game forecast panel (game UI, not editor), drawn with the game-ui kit | weather-agent |  |
| 9 | 🟢 done | forecasts/omens.toml: what each alignment brings (placeholder lines) | weather-agent |  |

### Physical spell system (`game.magic`)

5 open / 7 total · 1 doing · 2 review · 2 done · 1 ready todos

Spellcraft, input-driven casting, spell effects and the shared authoritative town state.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 109 | ⚪ todo | Implement authored-spell select, aim and fire interaction |  |  |
| 75 | 🟡 doing | Spell VFX in the world, juicy, both looks: wind orb, fire wall, wall of flame, fireball, Hearthglow dud; screen shake/hit flash; for the reel | game-ui-agent |  |
| 83 | ⚪ todo | One shared town simulation from spellcraft for HUD marks and creature rendering (no drift) | game-ui-agent |  |
| 64 | 🔵 review | Play keys drive casting (today the town auto-casts every 8 s) | game-ui-agent |  |
| 19 | 🔵 review | On-device fusion (cards 10-11 can't be filmed without it) | game-ui-agent |  |
| 18 | 🟢 done | Spell core: port creature studio spellcraft into the engine, point item-synthesis fusion at it | game-ui-agent |  |
| 85 | 🟢 done | One shared town per scene: spellcraft owns the simulation; the HUD, SpellFx and packages/creatures read the same town (no drift) | game-ui-agent |  |

### Creatures (`game.creatures`)

1 open / 3 total · 0 doing · 0 review · 2 done · 1 ready todos

Creature simulation, visibility, animation and polish across looks.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 65 | ⚪ todo | Pixel town: draw creatures in the scene (marks only show in ink town) | game-ui-agent |  |
| 68 | 🟢 done | Creatures visible in both looks: climbers and diggers modelled/sprited, animated, rising from the ground per moon (card 7) | weather-agent |  |
| 87 | 🟢 done | Creatures polish: bigger pixel stalker sprite; beetle hidden behind the well in brood camera; steam swirl over the hero's face; flat night grade | weather-agent |  |

### Wands and cartridge integration (`game.inventory`)

2 open / 2 total · 0 doing · 0 review · 0 done · 0 ready todos

Game energy behavior composed from the shared Inventory and Equipment systems: physical cartridges, equipped wand loading/replacement, finite authored-spell debit, identity-aware HUD energy totals and mana-consuming expansion.
Planned after: engine.equipment, engine.inventory, game.design

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 101 | ⚪ todo | Model finite physical energy cartridges as inventory items and sum them in the bottom HUD |  |  |
| 102 | ⚪ todo | Add grid capacity and mana-consuming inventory expanders |  |  |

### Playable interaction proof (`game.proof`)

1 open / 1 total · 0 doing · 0 review · 0 done · 0 ready todos

Prove authored-spell finite cartridge debit and camera readability after tooling acceptance and the reviewed game baseline; this is not new reel content.
Planned after: engine.review, game.camera, game.design, game.inventory, game.magic

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 104 | ⚪ todo | Verify authored-spell finite cartridge debit and readable camera |  |  |

### Gameplay camera and scale (`game.camera`)

1 open / 1 total · 0 doing · 0 review · 0 done · 0 ready todos

Compare readable gameplay scale, wider battlefield framing and tilt/lens variants without inventing numeric settings or a film camera.
Planned after: engine.review, game.design

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 108 | ⚪ todo | Compare gameplay camera scale and tilt/lens variants |  |  |

### Reference and moodboard decisions (`game.references`)

1 open / 1 total · 0 doing · 0 review · 0 done · 0 ready todos

Review existing candidate inputs and concept revisions into explicit In/Maybe/No/Future decisions, separate from storyboard and film production.
Planned after: engine.references, engine.review

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 110 | ⚪ todo | Review reference and moodboard decisions with existing candidate inputs | paul |  |

## Story and presentation (`content`)

8 open / 15 total · 0 doing · 0 review · 6 done · 0 ready todos

Narrative, sketches, demonstration footage and editing. Presentation choices do not silently change game rules.
Planned after: engine.review

### Storyboard narrative and sketches (`content.storyboard`)

5 open / 11 total · 0 doing · 0 review · 6 done · 0 ready todos

Authored shot sequence, dialogue, sketches and unresolved narrative calls.
Planned after: game.design, game.proof

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 105 | ⚪ todo | Review a new storyboard proposal after tooling and playable proof | paul |  |
| 16 | 🔴 blocked | Historical, unapproved content — Storyboard: real sketches for the 10 cards without engine pictures | storyboard-agent |  |
| 45 | 🔴 blocked | Historical, unapproved content — Storyboard: a failed fusion near the climax; re-judge round-10 post-fixes (ring stops on last digger) | storyboard-agent |  |
| 13 | 🔴 blocked | Historical, unapproved content — Storyboard: card 2 line 'I'm Paul, and I make the Atelico editor' — Paul's call (on camera) | paul |  |
| 14 | 🔴 blocked | Historical, unapproved content — Storyboard: draft 2-3 options for why the townsfolk freeze into card notes | paul |  |
| 8 | 🟢 done | Historical, unapproved content — Card 5 and card 7 sketches (forecast in-game, fire in the snow) | weather-agent |  |
| 44 | 🟢 done | Historical, unapproved content — Storyboard round 10: re-judge card 13 (player wins climax) and the dud-spell / two-moons-make-fire yellows | storyboard-agent |  |
| 12 | 🟢 done | Historical, unapproved content — Storyboard: re-judge round-3 fixes (recipe vs AI, marketplace vs written files, ten p.m., narrator named) | storyboard-agent |  |
| 15 | 🟢 done | Historical, unapproved content — Storyboard yellows: fire-in-snow affects nothing; purple vs orange creatures behave the same; tower wins climax not player | storyboard-agent |  |
| 17 | 🟢 done | Historical, unapproved content — Storyboard: regenerate storyboard.pdf export (stale, old path) | storyboard-agent |  |
| 39 | 🟢 done | Historical, unapproved content — Storyboard content: 13-card working-day path, 3 judge rounds |  |  |

### Animatic and reel (`content.reel`)

3 open / 4 total · 0 doing · 0 review · 0 done · 0 ready todos

Demonstration cuts and source footage. Historical agent outputs require review against the confirmed game design.
Planned after: content.storyboard, game.design

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 71 | 🔴 blocked | Historical, unapproved content — Reel v1: punchy 30-45 s AI-gamedev video from real engine footage (twin-moon day/night, pixel to ink switch, wicked weather, spell fusion, live graph edit), rendered offline | storyboard-agent |  |
| 70 | 🔴 blocked | Historical, unapproved content — Animatic: render the 15-card storyboard as a timed video in the engine (real scene captures + narration TTS), playable inside the editor | storyboard-agent |  |
| 106 | ⚪ todo | Produce a reel from the approved storyboard proposal |  |  |
| 74 | ⚫ dropped | Historical, unapproved content — Reel v1: 30-45 s punchy twin-moon reel (day/night timelapse, pixel->ink switch, wicked weather, spell fusion, editor+Claude beat) rendered offline from real engine footage; storyboard/exports/reel-twin-moons-v1.mp4 + contact sheet | storyboard-agent |  |
