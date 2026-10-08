# Evidence

Primary evidence only: captures of Paul's real editor window (`screencapture -l`), profiles of his running editor (`sample`), and commands the coordinator ran. Agents' own renders live with their work, not here.

One folder per day: `evidence/<YYYY-MM-DD>/`. Each file is named `<where>-<what it shows>[-<measured value>].<ext>`, so the name alone says what was proven. Never `shot1.png`, `r1s.png`, `after4.png`.

| File | Proves | Queue |
|---|---|---|
| 2026-10-07/editor-single-window-with-storyboard-tab.png | One editor window; Storyboard is a tab beside the scenes | — |
| 2026-10-07/storyboard-board-in-paul-editor.png | Storyboard Board renders live in the editor | — |
| 2026-10-07/storyboard-screenplay-in-paul-editor.png | Screenplay view: scene headings, NARRATOR dialogue | — |
| 2026-10-07/town-tab-hud-weather-grounded-houses-17ms.png | Town tab: HUD, live snow and embers, houses sitting on the ground; meter 17 ms · 60 fps (target 8 ms: not met) | 11, 48, 50 |
| 2026-10-07/town-tab-hud-text-crisp-after-overlay-fix.png | HUD text crisp after drawing it after the pixel upscale | 11 |
| 2026-10-07/queue-tab-in-paul-editor.png | Queue tab live in the editor | 35 |
| 2026-10-07/editor-host-profile-before-perf-round-2.txt | `sample` of Paul's editor: full repaints, per-frame text measuring, file polling | 50 |
| 2026-10-07/editor-host-profile-paused-combine-and-fire-24ms.txt | `sample` while paused: whole scene repainted 30×/s, colour conversion on present | 50 |
| 2026-10-07/local-engine-fuse-spells-names-qwen3.5-9b-1.4-3.4s.txt | The on-device model (local engine :8187) answers the fusion schema with legal choices and sensible names in 1.4–3.4 s | 19 |
| 2026-10-07/project-copy-ink-town-play-keys-fire-then-wind-model-names-blazing-curtain.png | Play keys (2, cast, 1, cast) arm then fuse in cast order; the cursor waits; the model's name "Blazing Curtain" types in (project copy, hidden host) | 19, 18 |
| 2026-10-07/project-copy-ink-town-play-keys-wind-twice-dud-model-names-zephyr.png | The same spell twice is a dud; the model gives it a humble name (project copy, hidden host) | 19 |
| 2026-10-08/reel-twin-moons-v1-contact-sheet-29s-1080p30.png | Reel v1 exists: 29 s, 1920x1080, 30 fps, offline engine renders (timelapse, pixel-to-ink, snow, embers, firefall, editor stand-in, title); no fusion shot yet; video at storyboard/exports/reel-twin-moons-v1.mp4 | 71 |
