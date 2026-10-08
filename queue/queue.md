# Queue

Snapshot of queue.db, 2026-10-07T23:38:00. Edit through `app queue` (or `queue/q.py`), not this file.

## Pieces

| # | | Task | Group | Lane | Owner | P | Commits |
|---|---|---|---|---|---|---|---|
| 71 | 🟡 doing | Reel v1: punchy 30-45 s AI-gamedev video from real engine footage (twin-moon day/night, pixel to ink switch, wicked weather, spell fusion, live graph edit), rendered offline |  | storyboard | storyboard-agent | 1 |  |
| 75 | 🟡 doing | Spell VFX in the world, juicy, both looks: wind orb, fire wall, wall of flame, fireball, Hearthglow dud; screen shake/hit flash; for the reel |  | spells | game-ui-agent | 2 |  |
| 51 | ⚪ todo | Changing screens: the live switch between the pixel town and the ink world (dusk flip) in the town scene |  | shader-graphs | shader-graph-agent | 9 |  |
| 54 | ⚪ todo | Ink town in Paul's window looks foggy/washed sepia and soft (vs crisp cel sketches): fix the look |  | shader-graphs | shader-graph-agent | 10 |  |
| 6 | 🔵 review | Weather visuals as graphs in both looks (weather-snow/embers/firefall), ground/roof accumulation |  | weather | weather-agent | 12 | ca43e28 (game 15e4edd) |
| 68 | 🟡 doing | Creatures visible in both looks: climbers and diggers modelled/sprited, animated, rising from the ground per moon (card 7) |  | creatures | weather-agent | 14 |  |
| 64 | 🔵 review | Play keys drive casting (today the town auto-casts every 8 s) |  | spells | game-ui-agent | 15 | 873a3aa |
| 16 | 🟡 doing | Storyboard: real sketches for the 10 cards without engine pictures |  | storyboard | storyboard-agent | 16 |  |
| 70 | 🟡 doing | Animatic: render the 15-card storyboard as a timed video in the engine (real scene captures + narration TTS), playable inside the editor |  | storyboard | storyboard-agent | 18 |  |
| 69 | ⚪ todo | Seed planting moment: plant the seed, first houses grow around it (card 3) |  | creatures | weather-agent | 20 |  |
| 3 | 🟡 doing | Hero contrast against the dark path (size done) |  | shader-graphs | shader-graph-agent | 30 | 0cc6556 d937ff2 |
| 65 | ⚪ todo | Pixel town: draw creatures in the scene (marks only show in ink town) |  | spells | game-ui-agent | 30 |  |
| 45 | 🟡 doing | Storyboard: a failed fusion near the climax; re-judge round-10 post-fixes (ring stops on last digger) |  | storyboard | storyboard-agent | 45 |  |
| 19 | 🔵 review | On-device fusion (cards 10-11 can't be filmed without it) |  | spells | game-ui-agent | 50 | 873a3aa,4be4b69 |
| 56 | ⚪ todo | Ink HUD: spell slot key numbers still tiny |  | game-ui | game-ui-agent | 60 |  |
| 13 | 🔴 blocked | Storyboard: card 2 line 'I'm Paul, and I make the Atelico editor' — Paul's call (on camera) |  | storyboard | paul | 70 |  |
| 14 | 🔴 blocked | Storyboard: draft 2-3 options for why the townsfolk freeze into card notes |  | storyboard | paul | 70 | 56eb78b |
| 74 | ⚫ dropped | Reel v1: 30-45 s punchy twin-moon reel (day/night timelapse, pixel->ink switch, wicked weather, spell fusion, editor+Claude beat) rendered offline from real engine footage; storyboard/exports/reel-twin-moons-v1.mp4 + contact sheet |  | storyboard | storyboard-agent | 1 |  |
| 48 | 🟢 done | Buildings float: base shading/shadow doesn't touch the ground (both looks) |  | shader-graphs | shader-graph-agent | 8 | 6c4a799 |
| 49 | ⚫ dropped | Houses float: contact shading/shadows offset from building bases (pixel + cel town) |  | shader-graphs | shader-graph-agent | 8 |  |
| 1 | 🟢 done | Pixel-art look as a shader graph (graphs/pixel-world), live preview, parity judge |  | shader-graphs | shader-graph-agent | 10 | 43111e7 |
| 2 | 🟢 done | Cel/anime look as a shader graph (graphs/cel-world), live preview, parity judge |  | shader-graphs | shader-graph-agent | 10 | 43111e7 |
| 5 | 🟢 done | Weather state as a function of the two moons (snow / embers / fire-in-snow + steam), seeded, tested |  | weather | weather-agent | 10 | 00ac59d d30d790 9f36134 |
| 10 | 🟢 done | In-game UI spec docs/game-ui.md (seed ring, moon clock, spell slots, prompts, wave counter...) judged |  | game-ui | game-ui-agent | 10 | ee111f0 |
| 11 | 🟢 done | packages/game-ui kit in both looks, placed in the moon town scenes |  | game-ui | game-ui-agent | 12 | 4113b01 |
| 53 | 🟢 done | HUD bugs: stub 'Night 3' vs schedule day 30; ink chip says align in 4h40m while moons aligned; ink fire icon reads as water drop; spell bar touches bottom edge |  | game-ui | game-ui-agent | 12 | 067a25b |
| 7 | 🟢 done | In-game forecast panel (game UI, not editor), drawn with the game-ui kit |  | weather | weather-agent | 15 | 00ac59d d30d790 9f36134 |
| 4 | 🟢 done | Continuous crane opening: sky -> crane -> player view -> hero walking (pixel town drawable in perspective) |  | shader-graphs | shader-graph-agent | 20 | 856549f |
| 8 | 🟢 done | Card 5 and card 7 sketches (forecast in-game, fire in the snow) |  | weather | weather-agent | 20 | 00ac59d d30d790 9f36134 |
| 55 | 🟢 done | Ink HUD type too small at editor zoom (0.65): labels like ENDS 2H 0M barely readable |  | game-ui | game-ui-agent | 20 | 5378b13 |
| 44 | 🟢 done | Storyboard round 10: re-judge card 13 (player wins climax) and the dud-spell / two-moons-make-fire yellows |  | storyboard | storyboard-agent | 25 | a31ef39 |
| 12 | 🟢 done | Storyboard: re-judge round-3 fixes (recipe vs AI, marketplace vs written files, ten p.m., narrator named) |  | storyboard | storyboard-agent | 30 | 56eb78b |
| 15 | 🟢 done | Storyboard yellows: fire-in-snow affects nothing; purple vs orange creatures behave the same; tower wins climax not player |  | storyboard | storyboard-agent | 35 | 56eb78b |
| 47 | 🟢 done | Game UI: ink panels still read 'like a website' — brush display font; real data for seed/spells/creatures (stubs today) |  | game-ui |  | 35 | 325c3bb |
| 9 | 🟢 done | forecasts/omens.toml: what each alignment brings (placeholder lines) |  | weather | weather-agent | 40 | 00ac59d d30d790 9f36134 |
| 18 | 🟢 done | Spell core: port creature studio spellcraft into the engine, point item-synthesis fusion at it |  | spells | game-ui-agent | 45 | aa8a701 |
| 17 | 🟢 done | Storyboard: regenerate storyboard.pdf export (stale, old path) |  | storyboard | storyboard-agent | 60 | 56eb78b |
| 38 | 🟢 done | Storyboard UI under design budget; screenplay; play-order board |  | storyboard |  | 90 |  |
| 39 | 🟢 done | Storyboard content: 13-card working-day path, 3 judge rounds |  | storyboard |  | 90 |  |
| 40 | 🟢 done | World round 3: pixel and cel rebuilt on plugins; one key shadow |  | shader-graphs |  | 90 |  |

## Software

| # | | Task | Group | Lane | Owner | P | Commits |
|---|---|---|---|---|---|---|---|
| 50 | 🟡 doing | Paul's Town tab at 16 fps: profile it and get it under 8 ms (pixel upscale/particles/HUD to GPU) |  | renderer | renderer-agent | 1 |  |
| 52 | 🟡 doing | Quality gate: nothing to review without a real screencapture of Paul's window checked against a written bar |  | hygiene | coordinator | 1 |  |
| 72 | 🟡 doing | Shared build broken: component_creatures unresolved in scene-3d |  | hygiene | weather-agent | 1 |  |
| 73 | 🟡 doing | Queue: urgent flag, severity, impact tags, due, dependency weight, computed rank; query/sort CLI; next honours urgency; Queue tab urgent strip |  | editor | queue-agent | 1 |  |
| 57 | 🟡 doing | Town frame: scene script rebuilds its full 831-sprite item tree to JSON every frame — make it incremental |  | runtime | runtime-agent | 2 | d0d8ee6 4125ff9 |
| 58 | 🟡 doing | Town frame: HUD and scene layouts rebuilt from scratch every frame — reuse layout when unchanged |  | runtime | runtime-agent | 2 | f670dc3 |
| 59 | ⚪ todo | Town frame: light probes re-baked every frame because walkers carry lights — bake only on change / dynamic lights separately |  | runtime | runtime-agent | 2 |  |
| 60 | ⚪ todo | Editor file poll (dir scan, cut list) blocks the main thread for tens of ms — move off main thread |  | runtime | runtime-agent | 2 |  |
| 22 | 🟡 doing | GPU renderer step 3-4: Skia CPU then Skia Ganesh on wgpu Metal; [host] renderer = cpu|gpu, runtime switch |  | renderer | renderer-agent | 7 |  |
| 23 | ⚪ todo | GPU renderer: identical-pixels test both backends + 8 ms frame-budget test |  | renderer | renderer-agent | 7 |  |
| 24 | ⚪ todo | GPU renderer step 5-7: port components, 3D post effects to GPU (render-3d split with shader agent), gpu default |  | renderer | renderer-agent | 8 |  |
| 67 | 🟡 doing | Registry publish failures: node-graph lockfile collision, scene-3d and item-synthesis failing tests block publishing |  | marketplace | queue-agent | 11 |  |
| 66 | ⚪ todo | Marketplace install flow for card 2: Claude installs the 5 world parts from the registry live in the editor (app registry serve, publish) |  | marketplace | queue-agent | 12 |  |
| 62 | 🔵 review | Shell commands in a session can write outside its owned files (only instructions stop it) |  | editor | queue-agent | 25 | eeec9c8 |
| 26 | 🔵 review | One Claude session per window/tab |  | editor | queue-agent | 30 | 5afa9d9,7284b1d (engine) |
| 25 | 🟡 doing | Automatic test runs (CI): no Mac GPU runner; interim = renderer agent runs tests before each push |  | renderer | renderer-agent | 40 |  |
| 61 | ⚪ todo | Terminal per window: host paints each window's Claude session into that window's agent surface (sessions, task 26) |  | renderer |  | 45 |  |
| 28 | ⚪ todo | Video playback inside the engine (no HTML hub) |  | editor |  | 50 |  |
| 33 | 🔵 review | atelico-applets PR #27 (layout text-measure cache): merge decision |  | hygiene | paul | 50 |  |
| 34 | 🔵 review | Engine PR #2 and game PR #1 (bootstrap -> main): keep current, merge when Paul says |  | hygiene | paul | 50 |  |
| 27 | ⚪ todo | Collaboration server (Rust, websockets, persistent state) |  | collab |  | 55 |  |
| 43 | ⚪ todo | scene-3d particle-graph test fails when run in parallel |  | editor |  | 55 |  |
| 29 | ⚪ todo | layout_matrix: welcome-modal 4 px spill |  | editor |  | 60 |  |
| 30 | ⚪ todo | button_press test failing (cause untraced) |  | editor |  | 60 |  |
| 46 | 🟢 done | DISK 100% full (20 GB free and falling): shared scratchpad copies — screenproj 158 GB, engine-copy 59 GB and growing (likely a copy including .atelico, which is 82 GB), sbproj-atelico-partial 21 GB |  | hygiene | coordinator | 1 |  |
| 63 | 🟢 done | Shared build broken: duplicate component-spellcraft package |  | hygiene | game-ui-agent | 1 |  |
| 20 | 🟢 done | GPU renderer step 1: wgpu surface present + 3D texture sampled, no readback, shader warm-up |  | renderer | renderer-agent | 5 | 8230619 |
| 21 | 🟢 done | GPU renderer step 2: neutral display-list interface, tiny-skia backend, zero pixel diff |  | renderer | renderer-agent | 6 | d10ffa7 |
| 36 | 🟢 done | START_HERE.md: update with today's state and point to queue/ |  | hygiene | coordinator | 15 | b161905 |
| 35 | 🟢 done | Queue in the editor: a Queue panel that reads queue/queue.db |  | editor | queue-agent | 20 | 463997e a20f33a a6a900d |
| 42 | 🟢 done | Find who switched Paul's main window to the Game Engine form and dropped the Storyboard tab (restored by coordinator) |  | hygiene | coordinator | 20 |  |
| 31 | 🟢 done | Engine working tree: tracked proof PNGs show as deleted (proof/node-graphs/...) — restore or explain |  | hygiene | coordinator | 25 |  |
| 32 | 🟢 done | Delete 18 GB abandoned project copy in scratchpad (sbproj-atelico-partial) |  | hygiene |  | 70 |  |
| 37 | 🟢 done | Editor perf patch rounds (damage, caches, meter, scroll, one window, storyboard tab) |  | perf |  | 90 |  |
| 41 | 🟢 done | Renderer research (4 lanes + recommendation: Skia Ganesh on wgpu Metal) |  | renderer |  | 90 |  |
