# Battlefield and night review

The same four current/day-control board IDs now show the **validated task 159 reserved viewport**. Both cel targets are visible above the control dock. The pre159 HUD-overlaid comparison remains below as evidence of the issue, alongside its original five findings. These are engineering and visual-review candidates; review 110 still owns art, camera and night-contrast decisions.

## Current layout — task 159

| Same launch within the new isolated run | Day, 10:00 | Night, 01:00 |
|---|---|---|
| Cel, 1100 × 800 | [Full native image](media/player-viewport/cel-day-1100x800.png) | [Full native image](media/player-viewport/cel-night-1100x800.png) |
| Pixel, 1100 × 800 | [Full native image](media/player-viewport/pixel-day-1100x800.png) | [Full native image](media/player-viewport/pixel-night-1100x800.png) |
| Cel, 1600 × 960 | [Full native image](media/player-viewport/cel-day-1600x960.png) | [Full native image](media/player-viewport/cel-night-1600x960.png) |
| Pixel, 1600 × 960 | [Full native image](media/player-viewport/pixel-day-1600x960.png) | [Full native image](media/player-viewport/pixel-night-1600x960.png) |

The controls occupy a separate 164px dock at 1100×800 and 122px at 1600×960, leaving a 1100×636 or 1600×838 World. The old top toolbar is also in that dock. Both cel targets now sit entirely inside the World rather than behind controls. The [open Inventory example](media/player-viewport/pixel-spell-inventory-1100x800.png) reserves a left drawer beside the World. It uses the scene's original restored hour, not the matched 01:00 / 10:00 control; direct drawer-switch behavior is covered by the native test.

Authored camera parameters, exposure 0 and the existing palette/LUT behavior are unchanged. **The World viewport/aspect changes**, so this is not pixel-identical framing versus the baseline. The new images come from a separate isolated run with the same fixture inputs and one 80→60 paid launch; canonical session and receipt IDs differ from the old run. Exact finite state stays unchanged within each run's view/hour/resize/drawer checks. Targets/projectiles still use debug forms, and physics remains flat-floor.

HUD overlap is corrected for these captured states. **Camera coverage and actor scale, weak night target separation, and the subtle near-caster launch point remain open.** No traversal, motion-readability or broad occlusion result follows from the layout fix. Native core 13/13 and UI 2/2 checks passed, including direct drawer transitions and missing-World controls. These are validated-source captures, not evidence of which native revision a currently running host has loaded. [Validation](media/player-viewport/validation.json), [source hashes](media/player-viewport/source-hashes.json) and [exact import receipt](media/player-viewport/import.json) pin engine `c606fa07`.

No new overview cards or review decisions were added. All four new revisions remain Unreviewed; previous card metadata, images and version-qualified provenance are preserved.

## Pre159 baseline — tasks 3 / 89

This is the preserved pre159 comparison from engine `43313f54` / game `96e1a5a`. Its original images, input hashes and five findings remain below. The first finding records the HUD occlusion corrected in the current layout above; it is not a description of the new cards. No camera, palette, outline, light or lens choice was adopted.

| Same paid event | Day, 10:00 | Night, 01:00 |
|---|---|---|
| Cel, 1100 × 800 | [Full native image](media/player-readability/cel-day-1100x800.png) | [Full native image](media/player-readability/cel-night-1100x800.png) |
| Pixel, 1100 × 800 | [Full native image](media/player-readability/pixel-day-1100x800.png) | [Full native image](media/player-readability/pixel-night-1100x800.png) |
| Cel, 1600 × 960 | [Full native image](media/player-readability/cel-day-1600x960.png) | [Full native image](media/player-readability/cel-night-1600x960.png) |
| Pixel, 1600 × 960 | [Full native image](media/player-readability/pixel-day-1600x960.png) | [Full native image](media/player-readability/pixel-night-1600x960.png) |

Within each style, only the camera's hour changes. Camera geometry, actor and targets, launch event, exposure 0, scene materials and the existing hour-dependent palette/LUT behavior stay intact. The styles use different existing geometry, projection and lighting; this is not a palette-only comparison between cel and pixel. The larger and smaller frames are actual native window sizes, not resized screenshots.

### Five pre159 findings — retained baseline

1. **The engineering HUD hides the action in cel.** The bottom panel alone covers 19.2% of the 1600×960 image and 22.8% of 1100×800; the top toolbar is additional. In the [small cel day frame](media/player-readability/cel-day-1100x800.png), most of both target spheres is behind the panel. They are already partly covered in the large frame. The controls are readable, but this is not an acceptable demonstration of unobstructed battlefield visibility. The [HLD wide-combat candidate](media/hld-wide-combat.png) keeps its HUD in the corner and leaves the actor-to-enemy corridor exposed. This does not prescribe copying that HUD.

2. **The current cel camera favors buildings over the action corridor.** Its central tower, roofs and fountain dominate the frame; the caster and targets sit low. The pixel view places the caster, both targets and side approaches in open ground above the HUD. The [TUNIC battlefield candidate](media/tunic-battlefield.png) keeps a small actor, nearby threats, the beam path and stairs visible together. Compare coverage and occlusion, rather than automatically enlarging the hero or moving the camera closer. This study does not change the existing square cameras.

3. **Actor scale alone is not a readability verdict.** The pixel actor is smaller in the image than the cel actor yet remains recognizable against the path. Its orange face and blue clothing form a clear cluster; the cel actor's green torso remains identifiable, but its lower action space is crowded by the fountain and HUD. The [HLD candidate](media/hld-wide-combat.png) demonstrates a small actor beside a much larger threat with open space between them. Our debug spheres are similarly sized targets, not a large-enemy scale test. No body-radius-derived visibility score is used.

4. **Night has local light already, but target separation weakens.** At the same exposure, windows and lanterns visibly light the ground in both styles. The actor retains a small local light pool. In the [pixel night frame](media/player-readability/pixel-night-1100x800.png), the green and amber targets become much darker than in the day control and lose separation from the path; the cel night view also has dark peripheral ground. The official [Sea of Stars day](media/sos-tod-day.png) / [night](media/sos-tod-night.png) pair shows the same deck and actors while a doorway becomes a stronger local source. Those 250×250 website frames are lighting references, not a prescribed clock, palette or acceptable peripheral darkness. Bright architecture by itself does not prove threat visibility.

5. **A live projectile is not yet a readable trajectory.** These frames contain the actual newly launched bolt with one immutable paid receipt, but it is a small point near the caster's feet in cel and is difficult to distinguish around the pixel actor. The [TUNIC candidate](media/tunic-battlefield.png) has long bright beam paths separated from the floor; that illustrates a visibility question, not permission to turn our bolt into a beam. This is one pinned launch instant, not motion-readability evidence or a complete charge/travel/impact comparison.

### Baseline scope

One explicit pointer fire spends 80→60 total energy; the loaded amber cartridge goes 40→20. The same receipt, authored revision, inventory custody, cartridge balances and exact physical body state survive both view changes, both sizes and both hours. The native test issues no model requests and ordinary drawing reads no GPU texture back; saving the proof images explicitly reads the finished frame. [Validation](media/player-readability/validation.json) and [byte/source provenance](media/player-readability/source-hashes.json) accompany the unmodified captures.

The earlier [task 156 town proof](/Users/paul/coding/atelico/atelico-app-engine/proof/editor/player-visuals/README.md) separately records ready, in-flight and contact states, including target health 12→9. Those captures use their original view/hour settings and are not part of this matched day/night matrix. Here only the newly launched event is pinned; matched-hour impact readability is still untested.

The actor uses the existing town hero; targets and projectile are still engineering debug forms. Physics uses the existing flat floor, not town collision geometry. There is no proof here of moving behind houses, staying visible through vegetation, traversing arbitrary light/dark terrain, edge-threat warning, or combat readability in motion. Those limits must remain visible when reviewing tasks 3/89. The test establishes a controlled current baseline and concrete deltas; it does not establish a satisfactory visual result.

The initial comparison replaced two reel thumbnails and added two daylight controls. Its [four card records](media/player-viewport/pre159-current-cards.toml), old reel images and [earlier reel-card metadata](media/player-readability/previous-current-cards.toml) remain available. The review journal is untouched. Each metadata/image revision gets a new review fingerprint; decisions are not transferred between revisions. Detailed frames stay linked here rather than becoming extra overview cards.

## Source candidates

All are existing official-source candidates already on the board. No reference was generated or recolored for this comparison.

- Heart Machine, [Hyper Light Drifter official gallery](https://www.heartmachine.com/hyper-light-drifter), screenshot 16, board ID `hld-wide-combat`: actor/enemy scale, quiet action floor and corner HUD. Not a mechanics or art commitment.
- TUNIC Team / Finji, [TUNIC Steam page](https://store.steampowered.com/app/553420/), board ID `tunic-battlefield`: actor/threat/path coverage. Not a mandate for its camera or material treatment.
- Sabotage Studio, [Sea of Stars official site](https://seaofstarsgame.co/), frames 076/026, IDs `sos-tod-day` / `sos-tod-night`: matched small lighting demonstration. Visual day/night labels are not verified in-game times.

Their exact source-image URLs, attribution and hashes remain in the board and [original provenance](provenance.toml). Task 110 is the human decision point. No review decision, film path, reel, production scene or live game state was changed.
