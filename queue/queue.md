# Queue

Snapshot of queue.db, 2026-10-13T13:20:46. Edit through `app queue` (or `queue/q.py`), not this file.

Object prerequisites describe planning order. Task dependencies control claims. Recorded done is not new user acceptance.

## Working Together (`working`)

4 open / 35 total · 0 doing · 1 review · 31 done · 2 ready todos

Plan, coordinate, inspect and review our work while building the tools we use. Work Tracker and AI sessions are projects in their own right.

### Work Tracker (`engine.tracker`)

1 open / 12 total · 0 doing · 1 review · 11 done · 0 ready todos

Understand what we are making, what is happening now, what is ready or blocked, and what needs review. A catalog of concrete parts connects to a grouped task DAG and searchable history. Implemented dashboard, typed model, bindings and history are available for inspection; human acceptance remains tracked separately in task103.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 96 | 🟢 done | Work Tracker: dashboard, catalog and history | codex-tracker |  |
| 113 | 🟢 done | Organize parts and component relationships | codex-coordinator |  |

#### Clear next actions (`tracker.clarity`)

1 open / 2 total · 0 doing · 1 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 165 | 🔵 review | Review the next-action dashboard in use | paul |  |
| 163 | 🟢 done | Show one concrete next action and owner per project | codex-coordinator |  |

#### Work model (`tracker.model`)

0 open / 2 total · 0 doing · 0 review · 2 done · 0 ready todos

Rust types define work parts, tasks, relationship kinds, status and dashboard projections. JSON and tables are conversion boundaries; business rules stay in typed code.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 111 | 🟢 done | Typed work model and data bindings | codex-tracker |  |
| 73 | 🟢 done | Queue: urgent flag, severity, impact tags, due, dependency weight, computed rank; query/sort CLI; next honours urgency; Queue tab urgent strip | codex-coordinator |  |

#### Queue database (`tracker.database`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

SQLite persists parts separately from tasks, with stable identities, task prerequisites and append-only activity. Additive migrations preserve earlier work and decisions.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 35 | 🟢 done | Queue in the editor: a Queue panel that reads queue/queue.db | queue-agent |  |

#### Data bindings (`tracker.bindings`)

0 open / 0 total · 0 doing · 0 review · 0 done · 0 ready todos

Rust Data schemas and checked store writes expose live object fields to Luau. Field revisions invalidate the UI that reads them. Normalized object references prevent nested copied tables from becoming a second domain model.

#### Dashboard and inspection (`tracker.dashboard`)

0 open / 5 total · 0 doing · 0 review · 5 done · 0 ready todos

Readable top-level areas, compact status summaries, drill-down part pages, relationships and a complete grouped task DAG. The overview stays useful without exposing every task at once.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 112 | 🟢 done | Compact dashboard and inspection | codex-dashboard |  |
| 145 | 🟢 done | Make the task graph readable before expanding historical work | codex-coordinator |  |
| 132 | 🟢 done | Keep task graph titles inside readable cards | codex-coordinator |  |
| 157 | 🟢 done | Keep every collapsed task group reachable and visible when fitting the graph | codex-coordinator |  |
| 161 | 🟢 done | Route grouped task dependencies around intervening cards | codex-coordinator |  |

#### Activity and search (`tracker.history`)

0 open / 0 total · 0 doing · 0 review · 0 done · 0 ready todos

Preserve decisions, notes, source evidence and task changes. Search exact passages by meaning with a local keyword fallback. Recorded task state is distinct from observed agent activity and user acceptance.

### Claude and Codex sessions (`engine.sessions`)

0 open / 5 total · 0 doing · 0 review · 5 done · 0 ready todos

Provider-neutral embedded rmux sessions, routing, visibility and process lifecycle.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 94 | 🟢 done | Support Claude and Codex in the same native session interface | codex-provider |  |
| 62 | 🟢 done | Shell commands in a session can write outside its owned files (only instructions stop it) | queue-agent |  |
| 26 | 🟢 done | One Claude session per window/tab | queue-agent |  |
| 61 | 🟢 done | Terminal per window: host paints each window's Claude session into that window's agent surface (sessions, task 26) | codex-coordinator |  |
| 131 | 🟢 done | Make fresh-project onboarding reflect the selected Claude or Codex session | codex-coordinator |  |

### Collaboration (`engine.collab`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

Peer synchronization of authoritative shared editing state. Derived queue/storyboard views remain local.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 27 | 🟢 done | Collaboration server (Rust, websockets, persistent state) | collab-agent |  |

### Tooling review and acceptance (`engine.review`)

2 open / 2 total · 0 doing · 0 review · 0 done · 1 ready todos

Implementation and technical validation precede a separate review of the working workspace with Paul. Content waits for explicit human acceptance.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 103 | ⚪ todo | Review the integrated tooling walkthrough | paul |  |
| 171 | ⚪ todo | Prepare one concrete tooling review with a runnable entry point | codex-coordinator |  |

### Build, integration and verification (`engine.build`)

1 open / 15 total · 0 doing · 0 review · 14 done · 1 ready todos

Consolidation, repository hygiene, verified checkpoints and proof. Existing done statuses remain historical agent reports.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 34 | ⚪ todo | Check engine PR 2 and game PR 1 against current branches | codex-coordinator |  |
| 72 | 🟢 done | Shared build broken: component_creatures unresolved in scene-3d | weather-agent |  |
| 91 | 🟢 done | Shared build broken: paint-skia gpu.rs texture() signature mismatch (renderer step 5) | renderer-agent |  |
| 93 | 🟢 done | Consolidate engine lanes and preserve all worktree/checkpoint history | codex-coordinator |  |
| 46 | 🟢 done | DISK 100% full (20 GB free and falling): shared scratchpad copies — screenproj 158 GB, engine-copy 59 GB and growing (likely a copy including .atelico, which is 82 GB), sbproj-atelico-partial 21 GB | coordinator |  |
| 52 | 🟢 done | Quality gate: nothing to review without a real screencapture of Paul's window checked against a written bar | coordinator |  |
| 63 | 🟢 done | Shared build broken: duplicate component-spellcraft package | game-ui-agent |  |
| 82 | 🟢 done | world.wgsl fix uncommitted: committed HEAD can't draw the town | shader-graph-agent |  |
| 42 | 🟢 done | Find who switched Paul's main window to the Game Engine form and dropped the Storyboard tab (restored by coordinator) | coordinator |  |
| 31 | 🟢 done | Engine working tree: tracked proof PNGs show as deleted (proof/node-graphs/...) — restore or explain | coordinator |  |
| 143 | 🟢 done | Publish and activate the verified shared component updates | codex-coordinator |  |
| 151 | 🟢 done | Upgrade the shared editor shell without overwriting project customizations | codex-coordinator |  |
| 155 | 🟢 done | Activate verified terminal and body-mapping improvements in the main editor | codex-coordinator |  |
| 160 | 🟢 done | Release the typed player and scene presentation as compatible shared packages | codex-release |  |
| 32 | 🟢 done | Delete 18 GB abandoned project copy in scratchpad (sbproj-atelico-partial) |  |  |

## Content (`content`)

3 open / 27 total · 0 doing · 0 review · 17 done · 0 ready todos

Reference decisions, storyboards and video editing tools, plus the posts and videos made with them. Approved content goals explain which capabilities we build next. Existing historical film drafts remain unapproved.
Planned after: engine.review

### Storyboard (`engine.board`)

0 open / 2 total · 0 doing · 0 review · 2 done · 0 ready todos

Reusable storyboard authoring tool: hierarchical groups, relationships, paths and inspection. A particular video storyboard is a separate artifact with its own versions and review decisions.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 95 | 🟢 done | Restore storyboard groups, native wires and whole/focused board navigation | codex-board |  |
| 38 | 🟢 done | Storyboard UI under design budget; screenplay; play-order board |  |  |

### Video Sequencer (`engine.video`)

0 open / 5 total · 0 doing · 0 review · 5 done · 0 ready todos

Reusable video assembly and playback tool: source clips, timing, titles, narration, music and export. Individual edits are separate content artifacts.

#### Timeline and cut model (`video.timeline`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

Typed Cut, Shot, Source, Music and caption records describe a video edit independently of its editor UI.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 78 | 🟢 done | Invalidate rendered shots after title placement, size or fade edits | codex-coordinator |  |

#### Native playback (`video.player`)

0 open / 3 total · 0 doing · 0 review · 3 done · 0 ready todos

Play and inspect a cut inside the app engine, including cuts authored in a different project with project-relative source paths.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 80 | 🟢 done | Engine video: play a cut from another project in the editor (cut paths are project-relative; game repo has no editor-video-viewer or [settings.ai]); reel must be playable in Paul's editor | runtime-agent |  |
| 28 | 🟢 done | Video playback inside the engine (no HTML hub) | runtime-agent |  |
| 124 | 🟢 done | Make short motion clips precise to review in the native video player | codex-coordinator |  |

#### Video export (`video.export`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

Render the authored cut with source footage, titles, narration, music and captions. Export evidence belongs to the specific content artifact being rendered.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 92 | 🟢 done | Engine video (044b965): title on a recording shot crashes: work dir not created before brand.title_at runs magick (needs create_dir_all(&work) before crates/cli/src/video/mod.rs:375); also the [music] bed is mixed mono into the voice track | runtime-agent |  |

### Reference and Concept Board (`engine.references`)

0 open / 4 total · 0 doing · 0 review · 4 done · 0 ready todos

A reusable comparison/review workspace for reference inputs, concept revisions and compatibility decisions across projects.
Planned after: engine.tracker

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 107 | 🟢 done | Build reusable concept/reference-board review and comparison tooling | codex-coordinator |  |
| 149 | 🟢 done | Open source pages and motion evidence directly from reference cards | codex-coordinator |  |
| 128 | 🟢 done | Keep reference category controls compact so the images lead |  |  |

#### Focused comparisons (`references.comparisons`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 166 | 🟢 done | Add focused comparisons to the reusable Reference Board | codex-coordinator |  |

### r/AIGameDev posts (`content.aigamedev`)

3 open / 16 total · 0 doing · 0 review · 6 done · 0 ready todos

Content grounded in actual game design choices, reference comparisons and playable evidence. Content goals inform build priorities; no historical storyboard narrative has been approved.

#### First game-development post (`content.first-post`)

3 open / 3 total · 0 doing · 0 review · 0 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 114 | ⚪ todo | Draft the next post purpose from verified gameplay evidence | codex-coordinator |  |
| 105 | ⚪ todo | Prepare a storyboard proposal for the agreed demonstration | codex-coordinator |  |
| 106 | ⚪ todo | Produce a reel from the approved storyboard proposal | codex-coordinator |  |

#### Storyboard drafts (`content.storyboard`)

0 open / 10 total · 0 doing · 0 review · 6 done · 0 ready todos

Authored shot sequence, dialogue, sketches and unresolved narrative calls.
Planned after: game.design, game.proof

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 16 | ⚫ dropped | Historical, unapproved content — Storyboard: real sketches for the 10 cards without engine pictures | storyboard-agent |  |
| 8 | 🟢 done | Historical, unapproved content — Card 5 and card 7 sketches (forecast in-game, fire in the snow) | weather-agent |  |
| 44 | 🟢 done | Historical, unapproved content — Storyboard round 10: re-judge card 13 (player wins climax) and the dud-spell / two-moons-make-fire yellows | storyboard-agent |  |
| 12 | 🟢 done | Historical, unapproved content — Storyboard: re-judge round-3 fixes (recipe vs AI, marketplace vs written files, ten p.m., narrator named) | storyboard-agent |  |
| 15 | 🟢 done | Historical, unapproved content — Storyboard yellows: fire-in-snow affects nothing; purple vs orange creatures behave the same; tower wins climax not player | storyboard-agent |  |
| 45 | ⚫ dropped | Historical, unapproved content — Storyboard: a failed fusion near the climax; re-judge round-10 post-fixes (ring stops on last digger) | storyboard-agent |  |
| 17 | 🟢 done | Historical, unapproved content — Storyboard: regenerate storyboard.pdf export (stale, old path) | storyboard-agent |  |
| 13 | ⚫ dropped | Historical, unapproved content — Storyboard: card 2 line 'I'm Paul, and I make the Atelico editor' — Paul's call (on camera) | paul |  |
| 14 | ⚫ dropped | Historical, unapproved content — Storyboard: draft 2-3 options for why the townsfolk freeze into card notes | paul |  |
| 39 | 🟢 done | Historical, unapproved content — Storyboard content: 13-card working-day path, 3 judge rounds |  |  |

#### Video edits (`content.reel`)

0 open / 3 total · 0 doing · 0 review · 0 done · 0 ready todos

Demonstration cuts and source footage. Historical agent outputs require review against the confirmed game design.
Planned after: content.storyboard, game.design

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 71 | ⚫ dropped | Historical, unapproved content — Reel v1: punchy 30-45 s AI-gamedev video from real engine footage (twin-moon day/night, pixel to ink switch, wicked weather, spell fusion, live graph edit), rendered offline | storyboard-agent |  |
| 70 | ⚫ dropped | Historical, unapproved content — Animatic: render the 15-card storyboard as a timed video in the engine (real scene captures + narration TTS), playable inside the editor | storyboard-agent |  |
| 74 | ⚫ dropped | Historical, unapproved content — Reel v1: 30-45 s punchy twin-moon reel (day/night timelapse, pixel->ink switch, wicked weather, spell fusion, editor+Claude beat) rendered offline from real engine footage; storyboard/exports/reel-twin-moons-v1.mp4 + contact sheet | storyboard-agent |  |

## Infinite Spell Game (`game`)

10 open / 58 total · 1 doing · 2 review · 45 done · 1 ready todos

The game and its physical spell rules. Paul defines the design; reel suggestions are not approved mechanics.

### Design and reference decisions (`game.design`)

1 open / 3 total · 0 doing · 1 review · 2 done · 0 ready todos

Confirmed rules and their source references; proposals and unresolved choices remain explicitly identified.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 97 | 🔵 review | Check the game-rules summary for omissions | paul |  |
| 170 | 🟢 done | Prepare the confirmed-rules summary and isolate actual unresolved game choices | codex-coordinator |  |
| 36 | 🟢 done | START_HERE.md: update with today's state and point to queue/ | coordinator |  |

### Town and world structure (`game.world`)

1 open / 2 total · 0 doing · 0 review · 1 done · 0 ready todos

Town layout, seed planting and shared scene structure.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 69 | ⚪ todo | Review the historical seed-planting and house-growth concept | paul |  |
| 79 | 🟢 done | Verify transforms and actor state remain shared when comparing render styles | codex-coordinator |  |

### Physical spell system (`game.magic`)

1 open / 8 total · 0 doing · 0 review · 5 done · 0 ready todos

Spellcraft, input-driven casting, spell effects and the shared authoritative town state.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 75 | ⚪ todo | Connect readable spell effects to authoritative casts and reviewed visual references | codex-coordinator |  |
| 83 | ⚫ dropped | One shared town simulation from spellcraft for HUD marks and creature rendering (no drift) | game-ui-agent |  |
| 64 | 🟢 done | Route gameplay keys through explicit authored-spell selection and finite casting | codex-coordinator |  |
| 18 | 🟢 done | Spell core: port creature studio spellcraft into the engine, point item-synthesis fusion at it | game-ui-agent |  |
| 19 | ⚫ dropped | On-device fusion (cards 10-11 can't be filmed without it) | game-ui-agent |  |
| 85 | 🟢 done | One shared town per scene: spellcraft owns the simulation; the HUD, SpellFx and packages/creatures read the same town (no drift) | codex-coordinator |  |
| 109 | 🟢 done | Implement saved-spell selection and named-target firing | codex-coordinator |  |
| 156 | 🟢 done | Connect authoritative casting bodies to the existing town presentation | codex-coordinator |  |

### Wands and cartridge integration (`game.inventory`)

1 open / 2 total · 0 doing · 0 review · 1 done · 1 ready todos

Game energy behavior composed from the shared Inventory and Equipment systems: physical cartridges, equipped wand loading/replacement, finite authored-spell debit, identity-aware HUD energy totals and mana-consuming expansion.
Planned after: engine.equipment, engine.inventory, game.design

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 102 | ⚪ todo | Choose how depleted inventory expanders handle occupied cells | paul |  |
| 101 | 🟢 done | Model finite physical energy cartridges as inventory items and sum them in the bottom HUD | codex-coordinator |  |

### Game HUD (`game.hud`)

0 open / 7 total · 0 doing · 0 review · 7 done · 0 ready todos

Game-facing equipment, inventory and power displays composed from reusable UI components. The confirmed power display sums remaining energy in inventory cartridges; generic inventory/equipment remain shared systems.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 159 | 🟢 done | Reserve an unobstructed battlefield beside shared player controls | codex-coordinator |  |
| 10 | 🟢 done | In-game UI spec docs/game-ui.md (seed ring, moon clock, spell slots, prompts, wave counter...) judged | game-ui-agent |  |
| 11 | 🟢 done | packages/game-ui kit in both looks, placed in the moon town scenes | game-ui-agent |  |
| 53 | 🟢 done | HUD bugs: stub 'Night 3' vs schedule day 30; ink chip says align in 4h40m while moons aligned; ink fire icon reads as water drop; spell bar touches bottom edge | game-ui-agent |  |
| 55 | 🟢 done | Ink HUD type too small at editor zoom (0.65): labels like ENDS 2H 0M barely readable | game-ui-agent |  |
| 47 | 🟢 done | Game UI: ink panels still read 'like a website' — brush display font; real data for seed/spells/creatures (stubs today) |  |  |
| 56 | 🟢 done | Ink HUD: spell slot key numbers still tiny | codex-coordinator |  |

### Gameplay camera and scale (`game.camera`)

0 open / 2 total · 0 doing · 0 review · 2 done · 0 ready todos

Compare readable gameplay scale, wider battlefield framing and tilt/lens variants without inventing numeric settings or a film camera.
Planned after: engine.review, game.design

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 108 | 🟢 done | Compare gameplay camera scale and tilt/lens variants | codex-coordinator |  |
| 140 | 🟢 done | Compare depth-aware tilt focus during actual casting | codex-coordinator |  |

### Pixel and ink/cel looks (`game.looks`)

1 open / 12 total · 0 doing · 0 review · 10 done · 0 ready todos

Readable lighting, shadows, authored shader looks and the switch between them.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 51 | ⚪ todo | Review the historical automatic look-switch experiment | paul |  |
| 54 | 🟢 done | Compare snowy cel grading while preserving the snow-free baseline | codex-coordinator |  |
| 76 | 🟢 done | Verify independent shadow directions for mesh and sprite casters | codex-coordinator |  |
| 48 | 🟢 done | Buildings float: base shading/shadow doesn't touch the ground (both looks) | shader-graph-agent |  |
| 49 | ⚫ dropped | Houses float: contact shading/shadows offset from building bases (pixel + cel town) | shader-graph-agent |  |
| 1 | 🟢 done | Pixel-art look as a shader graph (graphs/pixel-world), live preview, parity judge | shader-graph-agent |  |
| 2 | 🟢 done | Cel/anime look as a shader graph (graphs/cel-world), live preview, parity judge | shader-graph-agent |  |
| 89 | 🟢 done | Compare night lighting and actor separation against reference candidates | codex-coordinator |  |
| 88 | 🟢 done | Reproduce the reported missing daylight shadow in the cel town | codex-coordinator |  |
| 4 | 🟢 done | Continuous crane opening: sky -> crane -> player view -> hero walking (pixel town drawable in perspective) | shader-graph-agent |  |
| 3 | 🟢 done | Compare caster contrast at battlefield framing | codex-coordinator |  |
| 40 | 🟢 done | World round 3: pixel and cel rebuilt on plugins; one key shadow |  |  |

### Moons, weather and forecast (`game.weather`)

0 open / 5 total · 0 doing · 0 review · 5 done · 0 ready todos

Deterministic moon-driven weather and visible consequences.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 77 | 🟢 done | Verify night embers and haze at a recorded deterministic moment | codex-coordinator |  |
| 5 | 🟢 done | Weather state as a function of the two moons (snow / embers / fire-in-snow + steam), seeded, tested | weather-agent |  |
| 6 | 🟢 done | Validate graph-driven weather, accumulation and refraction in both render paths | codex-coordinator |  |
| 7 | 🟢 done | In-game forecast panel (game UI, not editor), drawn with the game-ui kit | weather-agent |  |
| 9 | 🟢 done | forecasts/omens.toml: what each alignment brings (placeholder lines) | weather-agent |  |

### Creatures (`game.creatures`)

0 open / 3 total · 0 doing · 0 review · 3 done · 0 ready todos

Creature simulation, visibility, animation and polish across looks.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 68 | 🟢 done | Creatures visible in both looks: climbers and diggers modelled/sprited, animated, rising from the ground per moon (card 7) | weather-agent |  |
| 87 | 🟢 done | Creatures polish: bigger pixel stalker sprite; beetle hidden behind the well in brood camera; steam swirl over the hero's face; flat night grade | weather-agent |  |
| 65 | 🟢 done | Pixel town: draw creatures in the scene (marks only show in ink town) | game-ui-agent |  |

### Reference and moodboard decisions (`game.references`)

3 open / 11 total · 0 doing · 1 review · 8 done · 0 ready todos

Review existing candidate inputs and concept revisions into explicit In/Maybe/No/Future decisions, separate from storyboard and film production.
Planned after: engine.references, engine.review

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 110 | ⚪ todo | Review reference and moodboard decisions with existing candidate inputs | paul |  |
| 164 | 🟢 done | Prepare a three-frame battlefield framing comparison | codex-coordinator |  |

#### Readable battlefield (`game.battlefield`)

2 open / 2 total · 0 doing · 1 review · 0 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 167 | 🔵 review | Choose the framing for the next battlefield camera test | paul |  |
| 168 | ⚪ todo | Capture matched camera tests for the selected battlefield framing | codex-coordinator |  |

#### Research pixel towns and battlefield environments (`game.references.pixel`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 116 | 🟢 done | Research pixel towns and battlefield environments | codex-references |  |

#### Research anime cel shading and Dragon Ball visual treatment (`game.references.cel`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 117 | 🟢 done | Research anime cel shading and Dragon Ball visual treatment | codex-coordinator |  |

#### Research spell crafting, aiming and casting interaction (`game.references.casting`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 118 | 🟢 done | Research spell crafting, aiming and casting interaction | codex-coordinator |  |

#### Research spell shapes, motion and impact effects (`game.references.effects`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 119 | 🟢 done | Research spell shapes, motion and impact effects | codex-coordinator |  |

#### Research inventory, equipment, power and spell HUD interfaces (`game.references.ui`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 120 | 🟢 done | Research inventory, equipment, power and spell HUD interfaces | codex-coordinator |  |

#### Research tilt-shift miniature scale and battlefield framing (`game.references.lens`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 121 | 🟢 done | Research tilt-shift miniature scale and battlefield framing | codex-coordinator |  |

#### Research dark moonlit worlds and day-night variants (`game.references.moon`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 122 | 🟢 done | Research dark moonlit worlds and day-night variants | codex-references |  |

### Playable interaction proof (`game.proof`)

2 open / 3 total · 1 doing · 0 review · 1 done · 0 ready todos

Prove authored-spell finite cartridge debit and camera readability after tooling acceptance and the reviewed game baseline; this is not new reel content.
Planned after: engine.review, game.camera, game.design, game.inventory, game.magic

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 104 | 🟢 done | Verify finite game-project casting with fixed aim targets | codex-coordinator |  |

#### Playable spellcasting loop (`game.playable`)

2 open / 2 total · 1 doing · 0 review · 0 done · 0 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 172 | 🟡 doing | Update the game's shared casting packages and launch the finite study | codex-coordinator |  |
| 173 | ⚪ todo | Connect shared movement and world aiming to the finite Player | codex-coordinator |  |

## Shared Platform and Components (`engine`)

5 open / 53 total · 0 doing · 0 review · 48 done · 2 ready todos

The app engine and reusable building blocks shared by our tools and game. Repository location is implementation metadata; this area describes their purpose.

### App Engine (`engine.app`)

4 open / 35 total · 0 doing · 0 review · 31 done · 1 ready todos

Native host and editor that can inspect, run and hot reload the software being built. Renderer, runtime and editor interaction are separate components of this project.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 133 | 🟢 done | Exclude hidden scene controls from the native UI action inventory | codex-coordinator |  |
| 134 | 🟢 done | Route scrolling to visible app scroll containers | codex-coordinator |  |
| 136 | 🟢 done | Prevent icon rendering from automatically starting model inference | codex-coordinator |  |

#### Renderer and performance (`engine.renderer`)

4 open / 19 total · 0 doing · 0 review · 15 done · 1 ready todos

Native GPU rendering, parity and frame budget.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 23 | ⚪ todo | GPU renderer: identical-pixels test both backends + 8 ms frame-budget test | codex-coordinator |  |
| 25 | 🔴 blocked | Automate software checks and provision a separate Mac GPU gate | codex-coordinator |  |
| 147 | 🔴 blocked | Compose host annotations and embedded previews from owned native frames | codex-coordinator |  |
| 50 | 🟢 done | Paul's Town tab at 16 fps: profile it and get it under 8 ms (pixel upscale/particles/HUD to GPU) | renderer-agent |  |
| 22 | 🟢 done | GPU renderer step 3-4: Skia CPU then Skia Ganesh on wgpu Metal; [host] renderer = cpu\|gpu, runtime switch | codex-coordinator |  |
| 24 | 🟢 done | Complete GPU component drawing and post-effect parity | codex-coordinator |  |
| 20 | 🟢 done | GPU renderer step 1: wgpu surface present + 3D texture sampled, no readback, shader warm-up | renderer-agent |  |
| 21 | 🟢 done | GPU renderer step 2: neutral display-list interface, tiny-skia backend, zero pixel diff | renderer-agent |  |
| 150 | 🟢 done | Clear obsolete scene controls and hit targets when app sources change | codex-coordinator |  |
| 146 | 🟢 done | Draw the embedded terminal through the shared native Canvas | codex-coordinator |  |
| 125 | 🟢 done | Preserve the full pixel camera view in smaller editor previews | codex-coordinator |  |
| 126 | 🟢 done | Keep pixel preview sampling consistent between GPU and CPU lens paths | codex-coordinator |  |
| 141 | 🟢 done | Keep existing camera lens effects on GPU | codex-coordinator |  |
| 152 | 🟢 done | Isolate last-good weather configuration by canonical project | codex-coordinator |  |
| 154 | 🟢 done | Remove the repeated World readback regression in native frame rendering | codex-coordinator |  |
| 158 | 🟢 done | Remove secondary-light receiver self-shadow pattern on flat mesh ground | codex-coordinator |  |
| 37 | 🟢 done | Editor perf patch rounds (damage, caches, meter, scroll, one window, storyboard tab) |  |  |
| 41 | 🟢 done | Renderer research (4 lanes + recommendation: Skia Ganesh on wgpu Metal) |  |  |

##### Responsive Inspector (`renderer.inspector`)

1 open / 1 total · 0 doing · 0 review · 0 done · 1 ready todos

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 169 | ⚪ todo | Test whether retaining the Inspector color picker reduces repaint cost | codex-coordinator |  |

#### Runtime and hot reload (`engine.runtime`)

0 open / 9 total · 0 doing · 0 review · 9 done · 0 ready todos

Run app components and update them while the host remains open. Typed data bindings and shared layout are separate reusable dependencies.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 57 | 🟢 done | Town frame: scene script rebuilds its full 831-sprite item tree to JSON every frame — make it incremental | runtime-agent |  |
| 58 | 🟢 done | Town frame: HUD and scene layouts rebuilt from scratch every frame — reuse layout when unchanged | runtime-agent |  |
| 59 | 🟢 done | Town frame: light probes re-baked every frame because walkers carry lights — bake only on change / dynamic lights separately | runtime-agent |  |
| 60 | 🟢 done | Editor file poll (dir scan, cut list) blocks the main thread for tens of ms — move off main thread | runtime-agent |  |
| 86 | 🟢 done | A HUD text changing every frame costs only its label (ticking clocks, countdowns) | runtime-agent |  |
| 81 | 🟢 done | Layout: a per-frame HUD text change rebuilds the whole kept layout (~20 ms); relayout only the changed leaf when its size is unchanged | runtime-agent |  |
| 90 | 🟢 done | Host type list misses SpellFx and Creatures component types (can't place them in scenes) | game-ui-agent |  |
| 43 | 🟢 done | scene-3d particle-graph test fails when run in parallel | runtime-agent |  |
| 162 | 🟢 done | Resolve relative project paths before launching a native host | codex-coordinator |  |

#### Editor interaction (`engine.editor`)

0 open / 4 total · 0 doing · 0 review · 4 done · 0 ready todos

Shared editor interaction correctness.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 129 | 🟢 done | Show scene import failures when opening a document instead of silently displaying another scene |  |  |
| 142 | 🟢 done | Redraw paused app scenes when viewport sizing changes | codex-coordinator |  |
| 29 | 🟢 done | layout_matrix: welcome-modal 4 px spill | runtime-agent |  |
| 30 | 🟢 done | button_press test failing (cause untraced) | runtime-agent |  |

### Spell runtime and finite casting (`engine.spellcraft`)

0 open / 4 total · 0 doing · 0 review · 4 done · 0 ready todos

Typed Rust spell definitions, validated cartridge composition over generic Inventory/Equipment, idempotent cast admission, physics adapter and normalized Data projections. The game consumes these APIs through a separate native player component.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 138 | 🟢 done | Define and account for funded spell instance completion | codex-coordinator |  |
| 127 | 🟢 done | Reusable finite cartridge and authored casting core |  |  |
| 130 | 🟢 done | Admit an explicitly reviewed authored spell revision without resetting player state | codex-coordinator |  |

#### Spell authoring (`spellcraft.authoring`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

The current Player can inspect and admit external source edits; that is not a crafting UI. Reuse the existing SpellDefinition/catalog validation and document boundaries. Game projects own authored files; authoring behavior belongs in a reusable package.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 135 | 🟢 done | Provide native authoring for saved spell definitions | codex-coordinator |  |

### Data Layer (`engine.data`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

Shared typed state and reactive binding infrastructure, adapted from ai-battleground. Data schemas define fields; the store validates writes and revisions; typed script proxies bind UI reads to changes.

#### Data schemas (`data.schemas`)

0 open / 0 total · 0 doing · 0 review · 0 done · 0 ready todos

Rust Data and DataKind traits plus derive macros describe checked fields and enum choices. File, JSON and script boundaries convert into typed objects.

#### Reactive Store (`data.store`)

0 open / 0 total · 0 doing · 0 review · 0 done · 0 ready todos

Store, ObjRef and per-field revisions provide one checked mutation path, typed read/write access, atomic batches and change subscriptions.

#### Typed Lua bindings (`data.bindings`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

data.object returns a userdata proxy. Field reads record render dependencies; field writes go through the checked store. Only render units affected by changed fields need to run again.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 115 | 🟢 done | Extend typed UI records to remaining storyboard and session consumers | codex-data |  |

### Graph Canvas (`engine.graph`)

0 open / 0 total · 0 doing · 0 review · 0 done · 0 ready todos

Reusable node-graph component for nodes, grouped regions, wires, pan and zoom. Work dependencies and storyboard paths use this visual machinery with distinct typed domain rules.

### UI Kit (`engine.ui`)

0 open / 2 total · 0 doing · 0 review · 2 done · 0 ready todos

Reusable native controls, layout, typography and themes shared across app and editor tools. The game HUD uses these components without owning their generic implementation.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 139 | 🟢 done | Initialize native text editing from the displayed field value | codex-coordinator |  |
| 153 | 🟢 done | Render JPEG reference images through the shared layout image loader | codex-coordinator |  |

### Shared applets (`applets`)

1 open / 2 total · 0 doing · 0 review · 1 done · 1 ready todos

Reusable components and UI kits consumed by game projects.

#### Shared layout (`applets.layout`)

1 open / 1 total · 0 doing · 0 review · 0 done · 1 ready todos

Layout/text-measure caching and shared applet UI foundations.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 33 | ⚪ todo | Check applets PR 27 against the current consolidated branch | codex-coordinator |  |

#### Applet showcase (`applets.showcase`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

Consolidated shared applets and preserved historical showcase variants.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 100 | 🟢 done | Consolidate shared applets and preserve historical showcase variants | codex-coordinator |  |

### Registry and publishing (`engine.registry`)

0 open / 6 total · 0 doing · 0 review · 6 done · 0 ready todos

Registry install/publish workflows and publication blockers.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 84 | 🟢 done | scene-3d: 4 failing tests block publishing it (and so every world part): the face stays in the lit band; the followed hero keeps every outline and eye pixel; a particle graph draws the same at the same moment; the Luau and manifest lists miss SpellFx in component.toml data_types | codex-coordinator |  |
| 148 | 🟢 done | Isolate shared-package proofs from live registries and hosts | codex-coordinator |  |
| 67 | 🟢 done | Registry publish failures: node-graph lockfile collision, scene-3d and item-synthesis failing tests block publishing | codex-coordinator |  |
| 66 | 🟢 done | Verify reusable world component installation into a fresh project | codex-coordinator |  |
| 137 | 🟢 done | Allow explicit registry package upgrades without shadowing them with installed copies | codex-coordinator |  |
| 144 | 🟢 done | Detect native component upgrades that require a rebuilt host | codex-coordinator |  |

### Generic inventory (`engine.inventory`)

0 open / 2 total · 0 doing · 0 review · 2 done · 0 ready todos

Shared item identity, container/grid capacity, movement and expansion primitives. No mana-specific container API.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 98 | 🟢 done | Create a reusable generic grid inventory and stable item identities | codex-coordinator |  |
| 123 | 🟢 done | Create reusable grid inventory UI over the typed Inventory component | codex-coordinator |  |

### Generic equipment (`engine.equipment`)

0 open / 1 total · 0 doing · 0 review · 1 done · 0 ready todos

Shared equipment slots and equipped item references, independent from carrying inventory and mana resource policy.

| # | State | Task | Owner | Legacy group |
|---|---|---|---|---|
| 99 | 🟢 done | Create reusable equipment slots independent from carried inventory | codex-coordinator |  |
