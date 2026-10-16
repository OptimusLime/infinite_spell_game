# Design review media — internal references, not a film

There are six comparison inputs. Four JPEG originals are retained alongside PNG copies for the editor. Displaying them does not select a target, establish gameplay, or grant approval to the existing reel. `SHA256SUMS` records exact image files.

| File | Origin and purpose | Decision/provenance |
|---|---|---|
| current-pixel-reel-v5-03.000s.png | Exact frame extracted with ffmpeg at 3.000 s from `storyboard/exports/reel-twin-moons-v5.mp4` | Current stored offline render; not a fresh live-editor capture or approved frame |
| current-anime-reel-v5-10.000s.png | Exact frame extracted with ffmpeg at 10.000 s from the same reel | Current stored offline render; same limitation |
| octopath-04.jpg | [Official Square Enix page](https://www.jp.square-enix.com/octopathtraveler2/about/), [source image](https://www.jp.square-enix.com/octopathtraveler2/assets/img/about/img04.jpg) | Maybe: dock/environment scale; do not inherit unreadably small actors |
| octopath-13.jpg | [Official Square Enix page](https://www.jp.square-enix.com/octopathtraveler2/about/), [source image](https://www.jp.square-enix.com/octopathtraveler2/assets/img/about/img13.jpg) | Maybe: blue night/warm light pools; strong focus blur is not approved |
| dragon-ball-4.jpg | [Official Bandai Namco page](https://www.bandainamcoent.com/games/dragon-ball-sparking-zero/media), [source image](https://us-east-1-bandai.graphassets.com/AXzioIclSWilEjFtsMJPwz/pGC8ObSQkKqYCjtMXE8d) | Maybe: broad energy silhouette and light/dark separation; close framing is not our camera target |
| historical-moon-clearing-target.jpg | Creature Studio `docs/research/magical_physics/art-direction/moon-clearing-target.jpg`, with `TARGET.md`/`PROPOSAL.md` | Historical AI concept art, not gameplay. September general direction was accepted; today's no-voxel instruction overrides conflicting details |

Official candidates downloaded 2026-10-08. Octopath: © SQUARE ENIX CO., LTD. Dragon Ball: © Bird Studio/Shueisha, Toei Animation; © Bandai Namco Entertainment Inc. Retained for internal reference and attribution, not presented as our artwork or licensed game assets.

Reel source SHA-256:

```text
cf8b689e8711ff97cbd2eea7a4caa6ee86a17544101da2f87f9199d532a81752  storyboard/exports/reel-twin-moons-v5.mp4
```

Reproduce the frames with `ffmpeg -ss 3 -i storyboard/exports/reel-twin-moons-v5.mp4 -frames:v 1 current-pixel-reel-v5-03.000s.png` and `-ss 10` for the anime frame. Original downloaded references were copied unchanged. No image generation or repainting occurred.

## Editor PNG copies

The layout image loader currently uses `tiny_skia::Pixmap::decode_png` (`components/layout/src/lib.rs`). The four JPEG references therefore have same-size PNG display copies. Cards name the PNG files; source JPEGs and their original hashes are retained. Conversion used `ffmpeg -i SOURCE.jpg -frames:v 1 -pix_fmt rgba DISPLAY.png`, with no generation, crop, resize or new artwork.

| Preserved original | Editor display copy | Size |
|---|---|---|
| `octopath-04.jpg` | `octopath-04.png` | 798×447 |
| `octopath-13.jpg` | `octopath-13.png` | 798×447 |
| `dragon-ball-4.jpg` | `dragon-ball-4.png` | 1200×675 |
| `historical-moon-clearing-target.jpg` | `historical-moon-clearing-target.png` | 1586×992 |
