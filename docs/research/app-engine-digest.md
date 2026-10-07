# App engine digest for Infinite Spell Game

Source: `~/coding/atelico/atelico-app-engine`, branch `universal-ai-app-engine`, read on 2026-10-07. Read-only survey.
All engine paths below are relative to that checkout. `E=~/coding/atelico/atelico-app-engine`.

## Strongest findings

1. `app init <dir>` makes a project outside the engine. It builds its own host from the engine's sources and shares the engine's `target/`.
2. The local registry (`~/.atelico/remote/index.git`) holds only editor panels and applets. **None of the 3D packages are published there.** The 3D packages need publishing first, or a path source.
3. Time of day is already data: `TimeOfDay`, `Sky`, `Palette`, `Look` nodes, a sun, **one** moon, stars, clouds and graded dawn/day/dusk/night. There is **no weather** of any kind.
4. Spells exist as `items/*.item.toml`. They are made of phases and steps that use sensor and effector capabilities. A local model merges two items under a strict schema, and Rust builds the result. Only one way to merge exists today: strap the payload on the carrier and fire it on contact.
5. There is **no multi-user or networked editing**. There is one store per process, the server listens on localhost only, and the last write wins. Several windows can share one store in one process.
6. Each host has one Claude pane. Multi-session work is by convention only (worktrees, ports, lanes in the handoff docs). The only real lock is `.atelico/recording.lock`.
7. Video already plays inside the engine. ffmpeg decodes it live into a layout node `{kind="video"}`, and `video` window mode edits `*.cut.toml`. There is no audio during playback.

## 1. A new project outside the engine

### Commands

| Step | Command | What it does |
|---|---|---|
| Build the CLI | `cd $E && cargo build -p atelico-cli` | Builds `$E/target/debug/app`. |
| Make the project | `$E/target/debug/app init ~/coding/infinite_spell_game [--port 7890] [--index <url>]` | Writes `atelico.toml`, `app/main.luau`, `scenes/`, the editor shell (`editor/`), `.atelico/editor.json` and `.gitignore`. Installs what the editor needs. Fails if `atelico.toml` already exists. |
| Run it | `cd ~/coding/infinite_spell_game && $E/target/debug/app run` | Syncs packages, builds the host, then launches the host, the control server and the Claude pane. |
| Install a package | `app install <name>` | Adds the package and its requirements, rebuilds and restarts. With no name, installs everything the project needs. |
| Find a package | `app search "<what it does>"` | Semantic search over the index, using engine embeddings, with FTS5 as a fallback. |
| Publish to the registry | `app publish <dir>...` | Validates, runs tests, packs a tar.gz (sha256), uploads the blob and pushes the index entry. |
| Registry admin | `app registry init\|serve --addr 127.0.0.1:8830\|sync\|status` | The index is a bare git repo at `~/.atelico/remote/index.git`. Blobs are served on :8830. |
| Recipes | `app recipe apply\|fork\|suggest\|accept\|show` | A recipe is a set of packages plus layout, theme and settings. |
| Live control | `app get\|set\|list\|watch <store.path>`, `app open <doc>`, `app play\|stop`, `app shot`, `app capture`, `app verify`, `app ui list\|find\|click` | Every UI element has an address `ui:<window>/...`. |

### Generated `atelico.toml` (from `crates/cli/src/registry.rs` `init_project`)

```toml
[host]
port = 7890
width = 1600
height = 960
app = "app/main.luau"

[agent]
session = "atelico-<dirname>"
command = "claude"
resume = "new"
drawer = 520

[registry]
sources = []
index = "file:///Users/paul/.atelico/remote/index.git"

[editor]
dir = "editor"
requires = ["layout", "ui-kit", "shader-forge"]
```

Typed schema: `crates/config/src/lib.rs` (`Config`, `Host`, `Agent`, `Registry`, `Installed`, `Source`, `Manifest`).
- `registry.sources` holds package folders: a path relative to the project, or `{git, rev, path}`.
- `registry.home` gives the project its own registry state.
- `[components.<name>] path=..., source=...` records each installed package. Versions are pinned in `atelico.lock`.

### How an outside project builds

- The CLI checks whether the project is "external", meaning it is not the engine checkout (`crates/cli/src/install.rs` `external`).
- For an external project, it writes `.atelico/host/Cargo.toml`. This is the engine's `crates/host` with the project's native packages added as dependencies (`crates/registry/src/workspace.rs`).
- It builds with `CARGO_TARGET_DIR=$E/target`. The binary is `atelico-host-<hash of project path>`, so engine crates compile only once.

### Getting the 3D packages in (the gap)

- `~/.atelico/remote/index.git` holds: ai, app-editor, audio, the editor-* panels, faces, layout, my-video-editor, phone-kit, pocket_oracle, read_aloud, ui-kit, video and video-editor.
- It does **not** hold scene-3d, world, cel-shading, sky, terrain, third-person, item-synthesis and the rest.
- **Proven path:** `app proof-team-video` (`crates/cli/src/team_video.rs`, functions `catalog` and `publish_registry`) does this for a fresh project. It publishes `$E/packages/{timeline, video, node-graph, shader-forge, skill-tree, scene-3d, world, third-person, cel-shading, cel-shaded-island, pixel-rendering, pixel-island, ...}` into a registry of its own. It then runs `init_project` with that index.
  - To do the same by hand: `app publish $E/packages/scene-3d $E/packages/world ...`, in dependency order.
- **Untested shortcut:** set `registry.sources = ["../atelico/atelico-app-engine/packages"]`. Path sources are relative to the project. The engine itself uses `"packages"` this way.

### Packages and component.toml

Example: `packages/cel-shading/component.toml`.

```toml
name = "cel-shading"
requires = ["scene-3d", "world"]
provides = ["look:cel-shaded"]
lua = """usage doc shown to Claude/search"""
project_files = ["scenes/cel-shading.scene.luau", "looks/dawn.cube"]   # copied into the project on install
[files]  "scenes/cel-shading.scene.luau" = "../../scenes/cel-shading.scene.luau"
[package] version="0.1.0"  title  description  keywords  data_types  permissions
[package.capture] image  clip  scene          # `app package-capture packages/x` renders them
[package.dependencies] scene-3d = "^0.1"
```

- **Luau package:** `component.toml` plus `lua/` (for example `lua/ui/*.luau` and `lua/apps/*.luau`). It installs live, with no restart.
- **Native package:** adds `Cargo.toml` and `src/`. Installing one rebuilds the host and restarts it.
- Some native crates (`sky`, `water`, `fire`, `particles`, `toon-character`, `island-props`, `arrow`, `pixel-sprites`) have **no** `component.toml`. They are workspace crates pulled in by `scene-3d` and the others, not packages you install.
- **Scenes:** `scenes/**/*.scene.luau` return a node tree, Godot style: `instance = "scenes/parts/...scene.luau"`, with overrides. Example: `scenes/cel-shaded-island.scene.luau`.

### Hot reload

- **The store** (`crates/store`) is the only write path. The GUI, file edits, MCP `set`, the CLI and Claude all go through it, with no restart.
- **File sync** (`crates/sync`) is two-way. A file that fails to parse keeps its last good values.
- Luau reads are dependency-tracked, so only the affected units re-run.
- `app reload` reloads scripts. `app restart` restarts the host.
- Open gaps: a race between TOML and scene sync, and graph files lose comments when written.

## 2. Rendering and visuals

Paths are under `packages/` unless they start with `crates/`.

| Part | Kind | What it does today |
|---|---|---|
| `scene-3d` | native | The 3D starter kit. Provides World, Camera (with `hour` override), Grade, probes, pixel lens and scene files. Node types: Environment, Grade, Look, Palette, Sky, TimeOfDay, Sea, Terrain, PixelArt, Material, Part, Camera. |
| `crates/render-3d` | native | The wgpu renderer: shadows, MSAA HDR, GTAO, toon-PBR, LUT blend. It knows nothing about content. |
| `sky` | native crate | Time of day as data (`src/lib.rs`) and the sky shader (`src/sky.wgsl`). |
| `water` | native crate | `Sea`: swell, distance field, foam, wakes. |
| `fire` | native crate | The one fire. Its look is a Shader Forge graph. Also bombs and explosions (`explosion.rs`, `bomb.rs`). |
| `particles` | native crate | CPU particles in the style of bevy_hanabi, drawn as flipbook sprites. Nodes in `scene-3d/nodes/particles.nodes.yaml`. |
| `cel-shading` | manifest + scene | Wind Waker look: 2 bands, ink outline, and `.cube` grades for day, dawn, dusk and night. |
| `cel-shaded-island` | manifest + scene | The cel-shaded island seen from the air. |
| `pixel-rendering` | manifest | 2.5D pixel art: sprites, an ortho camera snapped to pixels, sun-direction shadows. |
| `pixel-island` | manifest | A pixel islet, from afternoon to dusk. |
| `shader-forge` | native | Compiles node graphs to WGSL and renders them offscreen. Timeline params become uniforms, so there is no recompile. |
| `terrain-generation` | native | A seeded volcanic island heightmap, with `height_at`. |
| `world` | Luau | The app that runs scene-3d scenes. |
| `worlds` | Luau | An atlas of worlds painted by the local image model. |
| `toon-character` | native crate | `Character`: a big-headed adventurer. |
| `character-animation` | native | KayKit rig with CPU skinning. The Animator blends idle, walk and run. |
| `third-person` | native | Character controller, follow camera and rails. |
| `island-props` | native crate | Hut, Dock, Boat, Campfire. The campfire's light switches on at night. |
| `emotion-grid` | Luau | Emotion grid widget (eyes × mouth) bound to a row and column in the store. |
| `timeline` + `crates/timeline` | Luau + native | Typed timeline, keyframes and playback. Cut files are parsed here. |
| `node-graph`, `skill-tree` | native/Luau | A node canvas. The same canvas is reused as an in-game skill tree (`skill-tree/data/skills.graph.yaml`). |

### Day, night, moons and weather

- `TimeOfDay { hour 0-24, day_length s, running, hours }` sits on the World.
  - Its props are at `scene.<id>.nodes.root/world/time.props`.
  - It is derived every frame into `runtime.environment.<world>`: sun and moon direction, how much day it is, colours, and the two looks to blend.
- **Sky shader:** gradient, golden-hour haze, twinkling stars, wisps, cloud layers, sun disc, and a moon with craters and a halo.
- **Moon:** only one. It is placed opposite the sun (`sky/src/lib.rs:212`) and has no orbit or phases. At night the moon casts the shadows.
- **Proof:** `app proof-day-and-night` (`crates/cli/src/proof_day_night.rs`) runs a full day in 40 s. Evidence: `proof/scene-3d/day-and-night-with-a-look-per-hour/`.
- **Weather, rain, fog and seasons:** none. Timeline tracks driving the hour are still open (`docs/architecture-review-3d.md` §8).
- **Reading:** `docs/launch-video/3d-starter-kit.md`. Its sky section is titled "Sky dome, moons", from `creature_3d_studio` examples p32-p35 and `studio_core/src/day_night.rs`. A source for more moons.

## 3. Spells and items

| File | Role |
|---|---|
| `crates/items/src/lib.rs` | `Item {version, name, doc, composed_from, phases}`. `Phases {on_start, on_tick, on_contact: ContactPhase{target, every, steps}}`. `Step {id, use, target, input, output, every, params}`. `Capability`, `Port`, `Param` (min, max, default). `Item::validate`. |
| `crates/items/src/compose.rs` | Merge: `Compose {carrier, payload, fires_on, steps, name}`. `schema(base, addition)` builds a per-pair JSON schema of legal choices. `build()` is the deterministic builder. |
| `crates/items/src/rulebook.md` | The fixed system prompt. |
| `crates/items/src/runtime.rs` | `World` adapter trait. `Cast::start/tick/done`. `Exec`, `Event`. |
| `crates/items/src/physics.rs` | `SimWorld`: spheres, gravity, a terrain trait, sub-stepping, straps, contact detection, traces. |
| `packages/arrow/src/lib.rs` | Registers the capabilities (`capability!` macro, linked through `linkme`). |
| `packages/arrow/src/bin/merge-items.rs` | CLI merge against `http://127.0.0.1:8187/v1`. |
| `packages/item-synthesis` | Async in-game merge (`Task`, `Combined`, `Card`, icon). Model: `Qwen/Qwen3.5-9B-Q4_K_M`. |
| `packages/combine-items/lua/apps/combine_items.luau` | Luau UI: `engine:items()` and `engine:combine(a, b)`, which returns a task with `:poll()`. |
| `items/*.item.toml` | arrow, bomb, bomb-arrow (merged), seeker (tick: find → trace → steer). |

### Sensors and effectors

They are capability families, lettered as in spellcraft:
- **Sensors:** `sense.contact` (V), `sense.find` (Q), `sense.trace` (T), `sense.read` (R).
- **Effectors:** `body.arrow|bomb|orb` (B), `motion.launch|throw|steer` and `force.impulse` (P), `bind.strap|stick` (J), `matter.explode` (X), `status.stun` (D).

### Merging

- The request uses `response_format: json_schema, strict: true`, with constrained decoding by llguidance.
- The model only picks from enums. Names are 1-3 capitalised words.

### Firing

- In a scene: an `Arrow` node with `props = {item = "bomb-arrow", fired, aim_*}`. Examples: `scenes/bomb-arrow.scene.luau`, `scenes/island-shoot.scene.luau`.
- There is no Luau call that starts a cast directly.

### Tests and proofs

- `crates/items/src/tests.rs`
- `packages/arrow/tests/world.rs`
- `scripts/proof-merged-item-fired`, evidence in `proof/items/merged-item-fired/`
- `scripts/proof-character-shoots-a-bomb-arrow`

### Gaps

- Only one merge shape exists (strap the payload on the carrier, fire on contact).
- About 15 capabilities and 4 items.
- No mana or cost.
- No Lua "brain" (spellcraft's `logic.lua`).
- `scripts/measure-models-merging-items` still uses the old trigger/effect model.

Spellcraft origin: `~/coding/creatures/creature_3d_studio/crates/spellcraft/`. It is a port, not a dependency, because spellcraft's Lua 5.4 cannot share a binary with Luau.

## 4. Collaboration

| Part | What it is |
|---|---|
| `crates/store` | One in-memory typed tree per process. Every change has a revision and an origin: Gui, File, Mcp, Cli, Host or App. No user or session id. Last write wins. |
| `crates/sync` | Two-way sync between the store and files. Project files on disk are the only persistent state. |
| `crates/control` | HTTP and MCP (`POST /mcp`) on `127.0.0.1:<host.port>`. Localhost only. |
| `crates/registry/src/serve.rs` | Blob server for packages. Not for editing. |
| `app window open\|list\|mode\|close` | Several windows of **one** host share one store (`crates/host/src/shared.rs`). Proof: `scripts/proof-two-windows-edit-one-project-live`. |

- There is no websocket, CRDT or OT, no remote origin, no presence, no auth, and nothing binds `0.0.0.0`.
- "Shared live editing" is item 3 of the ambition list in `docs/start-here.md`, and is not built.
- **What it would need:**
  - A transport.
  - Changes tagged with a user or session.
  - Conflict handling.
  - Presence (`crates/control/src/pointer.rs` and `spotlight.rs` exist for the local agent only).
  - Auth.
  - A fix for the file-sync race.

## 5. Several Claude sessions

### The agent pane

- `crates/agent/src/lib.rs` runs `claude` in an rmux pane and mirrors it into the drawer.
- The rmux daemon outlives the host, so a restarted host reattaches.
- `[agent] resume = new|continue|last|<id>`.
- `crates/host/src/agent.rs` writes `<state>/agent/mcp.json`, which points Claude at the host's `/mcp`. It also writes `settings.json`, whose hooks post activity to `/agent/activity` for the event bar.

### Talking to Claude

- `app send "<text>"` types into the pane.
- Claude calls the MCP tools `say` (one line on the chat card) and `propose`. `propose` shows an approval card and blocks until the person answers.

### Splitting work

- Splitting is by convention only. See `docs/handoff-agent-a.md`, `docs/handoff-agent-b.md` and `docs/handoff-agent-c.md`.
  - Each agent has a lane and a worktree (`../engine-b`, `../engine-c`).
  - Rules: commit with explicit paths, pull with `--no-rebase`, never `rm`, one recording at a time (`pgrep -x app`).
- **Locks:** only `.atelico/recording.lock`, a `lockf` lock held by `app video make` while rendering (`crates/cli/src/video/make.rs:37`).
- **Proof isolation** (`crates/cli/src/isolate.rs`): each proof runs in a copy at `.atelico/proof-runs/<name>/`, with a free port and its own agent session.
- **For this game:** give each Claude session its own `--port` and `[agent] session`, assign files to lanes in a doc, and serialise recordings with the lock.

## 6. Video inside the engine

| Part | Role |
|---|---|
| `packages/video/src/stream.rs` | Layout node `{kind="video", path, time}`. One ffmpeg process per clip decodes frames at the drawn size into a tiny-skia pixmap. It loops, or holds the last frame with `once`. **No audio.** |
| `packages/video/src/lib.rs` | `engine:video(path)` extracts stills once (4 fps, 480 px) into `.atelico/video/<key>/`. |
| `packages/video/src/production.rs` | ffmpeg encode jobs, with progress reporting. |
| `packages/editor-video-viewer` | Editor part for `cut` documents. Plays the shot under the playhead from `runtime.cut.<name>` and `playback.<name>`. |
| `packages/editor-clips`, `packages/editor-timeline` | Clip lanes and track lanes. |
| `crates/cli/src/video/` | `app video render <cut>`, `app video make` (runs recordings, renders under the lock, writes `.atelico/<cut>/checklist.md`, `--publish`), `app video stills <file>`, `app video diagram <name>`. |
| Cut files | `docs/launch-video/launch-video.cut.toml`, `docs/team-video/team-video.cut.toml` and others. Parsed by `crates/timeline` (`cut`). |
| Video mode | `app window mode <id> video` on a cut. Proof: `app proof-launch-cut-edited-in-the-engine`. Scrub, trim, swap and reword are store writes, and only the changed shots re-render. |

### Showing a video in an in-engine UI instead of the HTML hub

1. **For a cut:** open it in a `video` window. This is already proven.
2. **For any mp4:** emit `{kind="video", path=..., time=t}` from a Luau layout, and drive `t` from a `playback.*` store value.
3. **For a review panel:** a small new editor part, like `editor-video-viewer`. It lists the mp4s, shows the stream node, and binds play and scrub controls to the store.
- **Missing:** audio during playback, and remote viewing (the hub is on the network, but an engine window is only local).
