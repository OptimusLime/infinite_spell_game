# Queue

Snapshot of queue/queue.db, 2026-10-07T20:17:37. Edit through `queue/q.py`, not this file.

## Pieces

| # | | Task | Lane | Owner | P | Commits |
|---|---|---|---|---|---|---|
| 5 | 🟡 doing | Weather state as a function of the two moons (snow / embers / fire-in-snow + steam), seeded, tested | weather | weather-agent | 10 |  |
| 10 | 🔵 review | In-game UI spec docs/game-ui.md (seed ring, moon clock, spell slots, prompts, wave counter...) judged | game-ui | game-ui-agent | 10 | ee111f0 |
| 6 | 🟡 doing | Weather visuals as graphs in both looks (weather-snow/embers/firefall), ground/roof accumulation | weather | weather-agent | 12 |  |
| 11 | 🟡 doing | packages/game-ui kit in both looks, placed in the moon town scenes | game-ui | game-ui-agent | 12 |  |
| 3 | ⚪ todo | Hero contrast against the dark path (size done) | shader-graphs | shader-graph-agent | 15 | 43111e7 |
| 7 | 🟡 doing | In-game forecast panel (game UI, not editor), drawn with the game-ui kit | weather | weather-agent | 15 |  |
| 8 | 🟡 doing | Card 5 and card 7 sketches (forecast in-game, fire in the snow) | weather | weather-agent | 20 |  |
| 12 | 🟡 doing | Storyboard: re-judge round-3 fixes (recipe vs AI, marketplace vs written files, ten p.m., narrator named) | storyboard | storyboard-agent | 30 |  |
| 13 | 🔴 blocked | Storyboard: card 2 line 'I'm Paul, and I make the Atelico editor' — Paul's call (on camera) | storyboard | paul | 30 |  |
| 14 | 🟡 doing | Storyboard: draft 2-3 options for why the townsfolk freeze into card notes | storyboard | storyboard-agent | 30 |  |
| 15 | 🟡 doing | Storyboard yellows: fire-in-snow affects nothing; purple vs orange creatures behave the same; tower wins climax not player | storyboard | storyboard-agent | 35 |  |
| 9 | ⚪ todo | forecasts/omens.toml: what each alignment brings (placeholder lines) | weather | weather-agent | 40 |  |
| 16 | ⚪ todo | Storyboard: real sketches for the 10 cards without engine pictures | storyboard |  | 40 |  |
| 18 | ⚪ todo | Spell core: port creature studio spellcraft into the engine, point item-synthesis fusion at it | spells |  | 45 |  |
| 19 | ⚪ todo | On-device fusion (cards 10-11 can't be filmed without it) | spells |  | 50 |  |
| 17 | 🟡 doing | Storyboard: regenerate storyboard.pdf export (stale, old path) | storyboard | storyboard-agent | 60 |  |
| 1 | 🟢 done | Pixel-art look as a shader graph (graphs/pixel-world), live preview, parity judge | shader-graphs | shader-graph-agent | 10 | 43111e7 |
| 2 | 🟢 done | Cel/anime look as a shader graph (graphs/cel-world), live preview, parity judge | shader-graphs | shader-graph-agent | 10 | 43111e7 |
| 4 | 🟢 done | Continuous crane opening: sky -> crane -> player view -> hero walking (pixel town drawable in perspective) | shader-graphs | shader-graph-agent | 20 | 856549f |
| 38 | 🟢 done | Storyboard UI under design budget; screenplay; play-order board | storyboard |  | 90 |  |
| 39 | 🟢 done | Storyboard content: 13-card working-day path, 3 judge rounds | storyboard |  | 90 |  |
| 40 | 🟢 done | World round 3: pixel and cel rebuilt on plugins; one key shadow | shader-graphs |  | 90 |  |

## Software

| # | | Task | Lane | Owner | P | Commits |
|---|---|---|---|---|---|---|
| 20 | 🟡 doing | GPU renderer step 1: wgpu surface present + 3D texture sampled, no readback, shader warm-up | renderer | renderer-agent | 5 | 8230619 |
| 21 | 🟡 doing | GPU renderer step 2: neutral display-list interface, tiny-skia backend, zero pixel diff | renderer | renderer-agent | 6 |  |
| 22 | ⚪ todo | GPU renderer step 3-4: Skia CPU then Skia Ganesh on wgpu Metal; [host] renderer = cpu|gpu, runtime switch | renderer | renderer-agent | 7 |  |
| 23 | ⚪ todo | GPU renderer: identical-pixels test both backends + 8 ms frame-budget test | renderer | renderer-agent | 7 |  |
| 24 | ⚪ todo | GPU renderer step 5-7: port components, 3D post effects to GPU (render-3d split with shader agent), gpu default | renderer | renderer-agent | 8 |  |
| 35 | 🟡 doing | Queue in the editor: a Queue panel that reads queue/queue.db | editor | queue-agent | 20 |  |
| 42 | ⚪ todo | Find who switched Paul's main window to the Game Engine form and dropped the Storyboard tab (restored by coordinator) | hygiene | coordinator | 20 |  |
| 26 | ⚪ todo | One Claude session per window/tab | editor |  | 30 |  |
| 25 | 🟡 doing | Automatic test runs (CI): no Mac GPU runner; interim = renderer agent runs tests before each push | renderer | renderer-agent | 40 |  |
| 28 | ⚪ todo | Video playback inside the engine (no HTML hub) | editor |  | 50 |  |
| 33 | 🔵 review | atelico-applets PR #27 (layout text-measure cache): merge decision | hygiene | paul | 50 |  |
| 34 | 🔵 review | Engine PR #2 and game PR #1 (bootstrap -> main): keep current, merge when Paul says | hygiene | paul | 50 |  |
| 27 | ⚪ todo | Collaboration server (Rust, websockets, persistent state) | collab |  | 55 |  |
| 43 | ⚪ todo | scene-3d particle-graph test fails when run in parallel | editor |  | 55 |  |
| 29 | ⚪ todo | layout_matrix: welcome-modal 4 px spill | editor |  | 60 |  |
| 30 | ⚪ todo | button_press test failing (cause untraced) | editor |  | 60 |  |
| 32 | ⚪ todo | Delete 18 GB abandoned project copy in scratchpad (sbproj-atelico-partial) | hygiene |  | 70 |  |
| 36 | 🟢 done | START_HERE.md: update with today's state and point to queue/ | hygiene | coordinator | 15 | b161905 |
| 31 | 🟢 done | Engine working tree: tracked proof PNGs show as deleted (proof/node-graphs/...) — restore or explain | hygiene | coordinator | 25 |  |
| 37 | 🟢 done | Editor perf patch rounds (damage, caches, meter, scroll, one window, storyboard tab) | perf |  | 90 |  |
| 41 | 🟢 done | Renderer research (4 lanes + recommendation: Skia Ganesh on wgpu Metal) | renderer |  | 90 |  |
