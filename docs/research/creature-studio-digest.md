# Creature Studio digest for Infinite Spell Game

Source repo: `/Users/paul/coding/creatures/creature_3d_studio` (read-only research, 2026-10-07).
Target: rebuild on the Atelico app engine (Rust host + Luau), `/Users/paul/coding/atelico/atelico-app-engine`.

All paths below are relative to the source repo unless absolute.

## 0. The short version

- The real spell work is `crates/spellcraft`. It is a Rust + Lua 5.4 library with **no Bevy dependency**. It runs a fireball from YAML, charges mana for every Lua VM instruction, and talks to physics through a `WorldAdapter` trait.
- A spell is YAML phases (`on_start`, `on_tick`, `on_contact`) made of steps. Each step uses one catalog component, such as `sense.nearby`, `logic.lua` or `force.impulse`.
- Composition ("arrow + bomb") exists only on paper: recipe S13 "Magical mortar" hands a payload to a carrier on contact. There is no `carrier`, `payload` or `rulebook` type in code.
- Collaboration is real: Yrs CRDT drafts, an immutable revision history mapped to Git, and a Cloudflare Durable Object coordinator. Persistence uses one Supabase table plus content-addressed archives. WebSocket sync is designed but not built.
- The sky has **no sun**. Two moons (purple and orange) orbit independently, and a colour LUT drives the cycle. There is no weather.
- NPCs, towns, enemies and "everything is a spell" are **ideas only**. The closest support: a caster, container and battery "can coexist in one physical object", plus a planned "Sky skull" fear spell that moves creatures.

## 1. Spell and item system

### 1.1 Key definitions (verbatim)

From `docs/research/magical_physics/SPELL_STRUCTURE.md`:

> "A **spell is one specific, atomic unit of authorship and casting**. A player sees and selects it much like a spell in a World of Warcraft spell slot. Examining that unit reveals its internal components."

> "Some components provide sensors or effectors. Some connect triggers directly to responses. Others contain conditional logic, such as 'if people are nearby, do this; otherwise, do that.'"

> "A wand can be both caster and battery. A stationary firing apparatus can receive mana over lines and store it before casting. A **spell container** holds a spell and determines its supported complexity and power level."

> "These roles can coexist in one physical object… without forcing a separate subsystem for every noun."

> "This corrects the earlier model that centered the spell on a moving physical object."

| Role | Definition (verbatim, `SPELL_STRUCTURE.md`) |
|---|---|
| Spell components | "Define nature, observations, possible actions and their composition. — A contact sensor; an effector; a trigger rule; a conditional logic component." |
| Spell container | "Holds a spell and limits its complexity and supported power." |
| Caster | "Initiates a cast through a held spell/container using an available mana supply. — A battery wand or a firing apparatus fed through mana lines." |
| Cast instance | "Owns transferred mana and tracks one execution, its origin, schedules, observations, rule state and spending." |
| Physical manifestations | "The bodies, particles, fields or other effects produced by that execution." |
| Mana source / storage / connection | Wands, batteries and mana lines. |

### 1.2 Mana pays for compute

> "Casting transfers a finite allocation from the caster to a new instance. That instance spends its own mana on sensing, rule evaluation, executed Lua and effects, and stops when exhausted."

> "Computation consumes mana: this is a requirement, not an optional design direction. Charge actual sensor activations, rule evaluations, effector work and executed Lua computation. An installed component that sleeps does not incur its execution cost merely for occupying a slot."

> "A large energy supply should not silently override a small container's complexity limit."

| Item | Value | Where |
|---|---|---|
| Lua VM instruction | `const INSTRUCTION_COST: f64 = 0.002;`, charged via an mlua hook every instruction | `crates/spellcraft/src/lib.rs:39`, hook ~1225 |
| `sense.nearby` read | 0.18 mana. Every tick at 60 Hz is 10.8/s; every 12th tick is 0.9/s | `SPELL_STRUCTURE.md` |
| Contact read / contact rule | 0.025 / 0.03 | `CAPABILITY_CATALOG.md:46` |
| `motion.accelerate` | "1 mana activation plus positive kinetic energy added at 100 joules per mana" | `CONVERSION_AND_CHAT.md` |
| Spend categories | `sensors, logic, rules, effectors, dissipation` | `lib.rs` |
| Exhaustion | `Instance::charge` moves the unaffordable remainder to `dissipation` and sets `status = "exhausted"` | `lib.rs:355` |
| Balancing plan | "Run 100 items, repeated queries, controllers and persistent effects… No new price coefficients are accepted here." | `design_sessions/COMPUTE_MANA_BALANCING.md` |

There are six separate quantities: component capacity, supported spell power, caster mana, instance mana, computation allowance, and physical/compute costs.

### 1.3 Lifecycle (`docs/research/magical_physics/LIFECYCLE_MODEL.md`)

| Phase | Purpose | Verbatim |
|---|---|---|
| `on_start` | Form and launch | "Create a sphere with explicit mass and publish its typed body handle; launch that same body with velocity." |
| `on_tick` | Observe and decide | "Execute due sensor acquisitions and Lua logic; skipped work incurs no execution charge." |
| `on_contact` | Respond to impact | "Receive explicit event body/target handles; apply ordered force and fire operations to their declared targets." |

- "File order is execution order; no arbitrary graph editor is required."
- Partial commit: "a paid force operation remains applied if later fire fails; executed Lua work is not refunded."
- Proposed extra hooks: "`on_end`, `on_exhausted`, `on_fault`" (`CAPABILITY_CATALOG.md:306`).
- Execution sequence (`SPELL_STRUCTURE.md`, "Validation and execution"): inspect, check container fit, request cast, transfer mana once, execute due work, apply accepted actions, complete or terminate.

### 1.4 Sensors, effectors and rules (proposal)

`docs/research/magical_physics/design_sessions/SHARED_COMPONENTS_AND_SPELL_RECIPES.md`:

> "The proposed working basis is **7 sensor families and 16 effector families**, with shared rules and execution services."

> "Each row means **input/observation → rule → paid action → output**. Outputs have handles: the body formed in one step is the body launched in the next."

| Kind | Families |
|---|---|
| Sensors (7) | R Read entity, Q Find entities, F Sample environment, T Trace space, V Receive event, C Check applicability, N Inspect network |
| Effectors (16) | M Move matter, B Shape matter, X Convert matter, E Transfer energy, P Apply force, J Bind parts, A Modify material response, D Set status, H Change vitality, L Change life state, K Claim control, G Give goal, I Emit signal, O Filter signal, W Connect energy ports, Y Relay observations |
| Rules / services | Sequence/branch, Select/rank/bounded loop, Chance, Schedule/cooldown/curve, Transaction/receipts, Instance/attachment ("Bind a recipe to a cast, item, status, child object or event participant") |

Sensor scope: "These are acquisition contracts. Installing `Read body motion` does not also grant access to minds…"

`CAPABILITY_CATALOG.md:301-320` keeps Lifecycle, Events, Selection, Sequencing, Conditions, State, Timing, Curves, Native math, Lua invocation, Transactions and Presentation mapping as distinct kinds. The reason: "Keeping their types distinct prevents a cheap selector or rule from bypassing a paid sensor."

### 1.5 How spells and items combine

There is no `carrier`, `payload` or `rulebook` type in code. Combination is designed through recipes (`SHARED_COMPONENTS_AND_SPELL_RECIPES.md`, S01–S26 and A01–A04):

| Recipe | Combination idea (verbatim) |
|---|---|
| S13 Magical mortar (l.226-233) | "Launch — X/B supply/form payload → P launch… V contact → call chosen payload recipe, e.g. S01 conversion." This is the arrow + bomb pattern: a carrier spell calls a payload recipe on contact. |
| S06 Poison (l.172) | "M load poison; B form carrier if needed; P lob → V ground contact → M deposit finite poison." |
| A03 Persistent enchanted item | "Item handle + R material/slots + C compatibility → bind installed recipe/grants through runtime; E fund its reservoir." Items are hosts that hold a recipe and a mana reservoir. |
| A04 Alchemy / crafting | "X exact recipe with supplied work, catalyst and loss products… recipes cannot declare arbitrary free valuable outputs." |
| S04 Reanimation | "K claim construct control; G follow/act. Q/R/T only at the installed brain's chosen cadence" |

Player-level combination is lineage, not item-on-item (`docs/architecture/shared-spell-sessions/CATALOG_LINEAGE.md`):
- Fork: "New spell identity and editable branch, with an explicit parent edge. Original unchanged."
- Mix: "Candidate with multiple source parents and component-level origin mappings."
- Example lineage: `Fireball@A ─mutate→ SpreadingFire@B`, `Ice@D ─mix→ SteamBurst@E`.
- v0.2 demo: "Keep the fireball. Make it leave poison when it hits." (`docs/versions/v0.2/plan.md`)

The older, superseded node-graph design is in `docs/research/magical_physics/prior_designs/spell_system.md`:
- `trait SpellNode { fn tick(&mut self, ctx: &mut TickContext); ... }`
- Lua combinators: `Sequential { Projectile {}, Gravity { strength = 9.8 }, GroundSensor { on_hit = Explosion { radius = 3 } }, Timeout { seconds = 10 } }`
- A CostTape for accounting.
- "Split Transformation — One spell becomes multiple spells (e.g., cluster bomb)" (l.1140).

Next build order (`design_sessions/TRIO_BUILD_PLAN.md`): "Fireball → Catch ward → Sky skull", three versions each.

### 1.6 Data format: YAML (live example)

`crates/spellcraft/assets/fireball.yaml`:

```yaml
version: 1
name: Fireball
mana: {source: wand, allocation: 60}
container: field
phases:
  on_start:
    - {id: form, use: body.sphere, output: projectile, params: {radius: 0.7, mass: 0.5}}
    - {id: launch, use: motion.launch, target: projectile, params: {speed: 6}}
  on_tick:
    - {id: nearby, use: sense.nearby, target: projectile, output: neighbors, every: 12, params: {radius: 12}}
    - {id: choose, use: logic.lua, input: neighbors, every: 6, params: {entry: full}}
  on_contact:
    target: projectile
    every: 1
    steps:
      - {id: push, use: force.impulse, target: event.other, params: {strength: 5}}
      - {id: burst, use: fire.burst, target: projectile, params: {radius: 4}}
```

Schema: `crates/spellcraft/src/definition.rs`, with `deny_unknown_fields` everywhere.
- `SpellDefinition { version, name, mana: Mana, container, phases: Phases, semantics: Option<SemanticMetadata> }`
- `Mana { source, allocation }`
- `Phases { on_start, on_tick, on_contact: ContactPhase }`
- `Step { id, use, target, input, output, every, params }`
- Limits: `MAX_SOURCE_BYTES = 64*1024`, `MAX_STEPS = 64`.

Other assets in `crates/spellcraft/assets/`:
- `components.yaml`: the catalog. Each entry has id, kind, phases, typed in/out ports, param bounds, cost, tags, aliases and an example.
- `scenario.yaml`: `max_ticks: 300, after_contact_ticks: 90, finish: reset`.
- `empty.yaml`.

### 1.7 Lua / Luau side

| Crate | Runtime | Use |
|---|---|---|
| `crates/spellcraft` | mlua 0.10.5, **Lua 5.4** vendored | Spell logic. `assets/fireball.lua` returns `{ full = function(ctx) ... end, lean = ... }`. The `ctx` API has `ctx:cached(requested?)` (returns `tick`, `count`, `nearby`, `distances`) and `ctx:schedule(id, every)` (`lib.rs` ~1236-1290). |
| `crates/spellcraft-media` | mlua **Luau** | Video/audio cue timelines only. |
| `crates/studio_scripting` | mlua via Bevy plugin | Old ImGui/scene scripting (`scene.spawn_cube`, `imgui.window`…). Not spells. |

- Limit, verbatim: "Current Lua executes a conditional decision over paid observations; it does not yet provide general conditional graph composition. Contact responses are ordered Rust effector steps, not a Lua `on_contact` function."
- Hot reload: `crates/spellcraft/src/server.rs` `poll_sources` (l.171) polls about every 250 ms. "Valid revisions apply to subsequent casts; invalid candidates retain the current definition/source. Active casts keep their definition and VM."

### 1.8 Capability catalog

Built (in `components.yaml`, 11 entries):
- `body.sphere`, `motion.launch`, `motion.accelerate`
- `sense.nearby`, `logic.lua`
- `force.impulse`, `force.lift`
- `fire.burst`, `matter.to_fire`
- `poison.coat`, `matter.to_poison`

Proposed (`docs/research/magical_physics/CAPABILITY_CATALOG.md`, plus `design_sessions/CAPABILITY_EXPANSION.md`): "90 sensor/effector backlog rows plus 54 additions give 144 design entries; only nine entries currently exist in the executable component catalog."
- Sensors: `sense.pose`, `motion`, `mass`, `bounds`, `contact`, `support`, `nearby`, `raycast`, `shape_cast`, `overlap`, `visibility`, `surface`, `path`, `material`, `temperature`, `heat_flux`, `fuel`, `oxidizer`, `wetness`, `pressure`, `flow`, `gravity`, `electric`, `light`, `mana`, `mana_flow`, `stored_work`, `integrity`, `stress`, `attachment`, `life`, `vitality`, `intent`, `owner`, `spell_state`, `threshold`.
- Effector groups:
  - matter: `body.form`, `matter.convert/split/merge/deposit/transmute`
  - motion and constraints: `force.apply`, `motion.steer`, `constraint.hold/tether/hinge/motor/break`
  - fields: `field.force/gravity/vortex`, `wave.pressure`
  - heat and phase: `heat.*`, `fire.ignite/sustain/extinguish`, `phase.freeze/melt/evaporate/condense`
  - fluids: `fluid.emit/pump`, `surface.coat`
  - structure and voxels: `structure.fracture/bond/repair`, `voxel.excavate/build`
  - electric: `electric.*`
  - living things and constructs: `tissue.*`, `construct.bind/command`
  - spell and mana: `spell.spawn`, `mana.transfer/connect`

### 1.9 Code layout (`crates/spellcraft`, its own isolated Cargo workspace)

| Area | Files |
|---|---|
| Core engine | `src/lib.rs` (1448 lines). Types: `Command` enum, `Observation`, `Body`, `Effect`, `trait WorldAdapter { sense, apply, effect_cost, body, conversion, environment }`, `Instance`, `Engine<W: WorldAdapter>`, `FixtureWorld`. |
| Physics | `src/physics.rs` (`RapierWorld`, rapier3d 0.31) |
| Definition | `src/definition.rs`, `src/scenario.rs` |
| Effects | `conversion.rs`, `destruction.rs`, `room_fire.rs`, `room_runtime.rs`, `surface_effects/`, `poison.rs` |
| Authoring | `semantics.rs`, `component_diff.rs`, `variants/`, `review.rs`, `draft.rs`, `library/` |
| Sharing | `history/`, `persistence.rs`, `collaboration/` (Yrs), `session.rs`, `shared_world.rs`, `cloud/`, `server.rs` |
| Binaries | `spellcraft-web`, `spellcraft-cloud`, `catalog`, `draft-proof`, `spellcraft-inspect` |
| Tests | `tests/`: 21 files, about 228 tests |
| Evidence | `docs/research/magical_physics/prototype/` |

Proposed refactor (`CAPABILITY_CATALOG.md:322-366`):
- Modules: `capabilities/`, `execution/`, `accounting/`, `world/`, `authoring/`.
- Shared types: `BodyId`, `CastId`, `Observation<T>`, `CostQuote`, `PreparedOperation`, `OperationReceipt`, `CapabilityDescriptor`, `CapabilityGrant`.

Design rule: extend `WorldAdapter` "rather than making Bevy entities the public spell handle."

### 1.10 Other spell docs worth reading

- `docs/research/magical_physics/README.md`: overview, plus the Aquarium precedent (see section 4).
- `IMPLEMENTATION_ROADMAP.md`, `NATIVE_WORLD_DIRECTION.md`, `CONVERSION_AND_CHAT.md`, `VISUAL_REVIEW.md`, `UI_HANDOFF.md`.
- `design_sessions/`: `SPELL_BRAINSTORM.md`, `RUST_CAPABILITY_STRUCTURE.md`, `CAPABILITY_EXPANSION.md`, `COMPUTE_MANA_BALANCING.md`, `TRIO_BUILD_PLAN.md`.
- `fire-study/`: Hades breakdown, ground patches, world integration.
- `prior_designs/`: `spell_system.md`, `spell_system_phase0.md`, `player_fantasy_demo.md` (superseded).

## 2. Day/night and sky

Verbatim, from `docs/CONTINUATION_SKY_RENDERING.md`:

> "**There is no sun. There is no daytime.** This world has TWO MOONS that orbit independently … The sky is always night. Lighting comes from the moons."

### 2.1 Cycle

- Code: `crates/studio_core/src/day_night.rs`. All 7 phases are marked "COMPLETE" in `docs/DAY_NIGHT_CYCLE_PHASES.md`. The design is in `docs/DAY_NIGHT_CYCLE_PLAN.md` (974 lines).
- Time: `time: f32`, "Current time in cycle (0.0 - 1.0, wraps automatically)". Speed: "1.0 = one cycle per second … Typical values: 0.01 (100 sec/cycle)". There is also `paused`.
- The phases are colour keyframes (`ColorLutConfig::dark_world()`), not named states:

| t | Label | Ambient (rgb, intensity) | Fog density | Exposure |
|---|---|---|---|---|
| 0.0 | Deep Night | (0.02,0.01,0.04) 0.05 | 0.6 | 1.0 |
| 0.2 | Pre-Dawn | (0.04,0.02,0.03) 0.08 | 0.5 | 0.9 |
| 0.3 | Dawn Peak | (0.15,0.05,0.08) 0.15 | 0.7 | 1.1 |
| 0.4 | Twilight | (0.08,0.04,0.02) 0.1 | 0.5 | 1.0 |
| 0.5 | Night | (0.03,0.015,0.01) 0.05 | 0.6 | 1.0 |
| 0.7 | Second Transition | (0.12,0.04,0.1) 0.12 | 0.65 | 1.05 |
| 0.85 | Late Night | (0.02,0.01,0.05) 0.05 | 0.6 | 1.0 |

- Interpolation: `Linear | CatmullRom | Step`. "CatmullRom" is really smoothstep.
- Wiring: ambient and fog are "stored in LUT (shader integration partial)"; "saturation/contrast/tint stored but not yet shader-applied".

### 2.2 Two moons (`MoonCycleConfig` in `day_night.rs`)

| Moon | Period | Phase offset | Inclination | Zenith colour | Horizon colour | Intensity (zenith/horizon) | Disc size |
|---|---|---|---|---|---|---|---|
| Purple | 1.0 | 0.0 | 30° | (0.5,0.2,0.9) | (0.8,0.3,0.5) | 0.6 / 0.15 | 0.12 (~7°) |
| Orange | 0.8 ("Faster than purple moon") | 0.5 | 15° | (1.0,0.5,0.15) | (1.0,0.3,0.05) | 0.5 / 0.1 | 0.08 (~5°) |

Orbit maths (`calculate_position`), all portable:
- `moon_time = (cycle_time / period + phase_offset).fract()` and `angle = moon_time * TAU`.
- Position: `x = cos`, `y = sin·cos(incl)`, `z = sin·sin(incl)`.
- Colour lerps from horizon to zenith over `(h+1)/2`.
- Intensity fades in from `set_height` up to 0.3.

### 2.3 Sky rendering

- Main path: a fullscreen sky dome in `assets/shaders/sky_dome.wgsl` (517 lines). Config is in `crates/studio_core/src/deferred/sky_dome.rs`, the render node in `sky_dome_node.rs`.
- Textures are generated by MarkovJunior: `textures/generated/mj_clouds_001.png`, `mj_moon_purple.png`, `mj_moon_orange.png`, `mj_stars.png`.
- Clouds: Rayleigh/Mie scattering, per-moon cloud lighting, edge glow.
- Stars: texture plus twinkle, `0.4 + twinkle * 0.6`.
- Alternative: a sky sphere mesh material (`crates/studio_core/src/sky_sphere.rs`).
- Terrain lighting (`assets/shaders/deferred_lighting.wgsl`):
  - two moon shadow maps
  - `moon_altitude_factor = smoothstep(-0.15, 0.1, alt)`
  - horizon warmth `vec3(0.15,0.05,-0.1)`
  - darkness factor "1.0 when at least one moon is visible, drops toward 0.25 when both are below"
- Leftover sun code is marked for deletion (`SunAppearance`, `render_sun`, `SunOrbit`).

### 2.4 Plans and status

| Doc | Status |
|---|---|
| `docs/plans/markov_cloud_sky_alternative.md` | ACTIVE. "ALL sky elements (clouds, moons, stars) are generated by MarkovJunior". Phases 0-4 and 8-11 are done. Cloud UV flow and wind are planned. |
| `docs/plans/moon_environment_lighting.md` | ACTIVE. Phases A-F: dynamic uniforms, altitude, zenith darkness, dynamic ambient, dawn/dusk scatter, env LUT. A-C look partly done in the shader, but the doc is not updated. |
| `docs/plans/seus_sky_techniques.md` | Technique ranking. Aurora is "FUTURE"; volumetrics are deferred. |
| `docs/plans/dm_skybox_comparison.md`, `markov_sky_textures_research.md` | Research |
| `docs/DEBUG_SKY_CLOUDS.md` | World-space cloud sampling fix. FIXED. |

**Weather: none.** No rain, snow or wind system exists.

## 3. Collaboration, multiplayer and persistence

Main docs: `docs/architecture/shared-spell-sessions/`. These include README, COLLABORATION, PERSISTENCE, CATALOG_LINEAGE, PLAYER_WORKFLOW, DELIVERY_PLAN, ACCEPTANCE, CLOUD_TEST_ENVIRONMENT, PR3_PROGRESS and V0_2_DEMO.

### 3.1 Model (verbatim, `README.md`)

> "craft and test a spell, leave with that spell and its saved session, return to improve it, then do the same together and reuse spells through a catalog. Rust owns capabilities, persistence and execution; Lua composes permitted behavior; the native and HTML UIs call the same APIs."

| State | Definition |
|---|---|
| Draft | "live Yrs edits; may temporarily contain invalid YAML/Lua." |
| Accepted spell revision | "validated immutable source, parentage and semantic diff." |
| Crafting session save | "revision pins, draft/history/decisions, authored test setup and recorded results. It preserves the work, not an in-flight projectile." |

Two more rules, verbatim:

> "World positions and mana are authoritative simulation state, not conflicting CRDT values. A CRDT merge can converge while producing an invalid spell, so convergence never bypasses validation or authorizes execution."

> "the first version explicitly trusts one chosen simulation host. Arbitrary client-authored physics is out of scope."

### 3.2 Channels (`COLLABORATION.md`)

| Channel | Durability |
|---|---|
| Draft (Yrs update, epoch, op ID) | Durable acknowledgement |
| Presence | Ephemeral |
| Command (join, begin test round, cast, pause, save, accept proposal) | Ordered by session/host |
| World ("Only the leased simulation host emits truth") | Resync if baseline missing |
| Publication (outbox) | Durable |

The Yrs doc is `draft { files: map<path, text>, metadata: map }`. "YAML and Lua text are the editable source of truth."

### 3.3 What is built

| Piece | Where | Status |
|---|---|---|
| Yrs CRDT drafts, sessions, history (Git-mapped) | `crates/spellcraft/src/collaboration/`, `session.rs`, `history/` | Built and tested |
| Coordinator state machine: `Command { Append, Claim, Start, Frame, Finish }`, host lease by epoch | `crates/spellcraft-coordinator/src/lib.rs` | Built |
| Cloudflare Worker + Durable Object `SessionCoordinator`, HTTP POST | `crates/spellcraft-coordinator/src/edge.rs`, `wrangler.toml` | Built. "No response escapes the output gate before the journal and state are durable." |
| Cloud persistence: one table `spellcraft_pr3.heads(project, generation, payload sha256)` plus a private Storage bucket | `crates/spellcraft/src/cloud.rs` (feature `cloud`) | Built |
| Full Postgres schema (projects, membership, revision edges, outbox) | `PERSISTENCE.md` | Design only |
| WebSocket `WS /v1/craft-sessions/:id/sync` | `CATALOG_LINEAGE.md` | **Not built** |
| `supabase/` folder | only `supabase/.temp/cli-latest` | Empty |
| Semantic catalog search (embeddings, Worker `/embeddings`) | `library.rs`, `edge.rs` | Built |

Delivery (`DELIVERY_PLAN.md`, `PR3_PROGRESS.md`):
- PR1 (solo craft and history): merged.
- PR2 (craft together; 20 concurrent edit pairs, 10 paired casts): merged.
- PR3 (find, fork, combine): merged as #10 on 2026-09-29. The plan doc is stale.
- v0.2 film "Borrow fire. Keep poison.": in progress.

Player workflow (`PLAYER_WORKFLOW.md`):
- "one shared conversation, named bubbles, one composer… no source editor."
- Decision policy: "**Owner decides** (default), **Any editor decides**, or **Everyone agrees**."

### 3.4 Messaging (`docs/messaging/`)

- Approved hook: "**A world where magic and physics collide. Make, mix, mutate your spells—together.**"
- Goal, wording not approved: "**Magic as a creative medium deep enough that entirely new styles of magic can emerge.**"
- What the game is: "A physics-driven magic sandbox in which players develop spells through an iterative collaboration with an in-game partner." "Forking and reassembly are central to the concept, not incidental distribution features."
- Also: "Mana is physical."
- Tone rules:
  - No "You don't X; you Y" formula.
  - No trailer narration.
  - Don't lead with AI or tech terms.
  - Avoid the mental-model errors listed: "one request producing a finished spell; the assistant mindlessly obeying; the player having to program everything alone; a conventional ingredient menu."

## 4. NPCs, buildings, towns, enemies, "everything is a spell"

**Nothing is implemented.** There is no NPC, town, enemy, faction or quest code. The phrase "everything is a spell" does not appear. v0.2 says: "NPC health gameplay and an autonomous in-app model are outside this version." (`docs/versions/v0.2/plan.md:67`)

| Idea | Source | Status |
|---|---|---|
| Stationary firing apparatus fed by mana lines (a building as a spell host) | `SPELL_STRUCTURE.md` | Concept |
| Item as recipe host plus reservoir (A03) | `SHARED_COMPONENTS_AND_SPELL_RECIPES.md` | Proposal |
| Creature sensors: `sense.vitality` ("No omniscient private brain access"), `sense.intent`, `sense.owner`, `construct.bind/command` ("controller spends its own allocated mana") | `CAPABILITY_CATALOG.md` ~l.207 | Unchecked backlog |
| Sky skull: Fear ("Compatible creatures who can see it flee… claim limited movement control → give a flee goal"), Pursuit | `design_sessions/TRIO_BUILD_PLAN.md` | Planned, third in build order |
| Factions: "frozen people vs. river people who hate each other"; Tundra Necromancer, Ice King boss; castles along the river | `docs/plans/map_editor/VISION_NOTES.md` §5, §21 | Vision |
| "Playing God": describe story, build world, populate, run simulation, drop in | `VISION_NOTES.md` §22 | Vision |
| RTS automation: "Lua scripts that operate these guys"; "code a lot of the logic directly into the game, directly into the objects" | `VISION_NOTES.md` §23 | Vision |
| "Harvester Brain… doesn't need to run A* every frame and cost a bajillion mana… Costs proportional to compute costs." | `VISION_NOTES.md` §24 | Vision |
| "Everything has a cost. Everything follows rules." | `VISION_NOTES.md` §10 | Key insight |
| MarkovJunior building generators (`Apartemazements.xml`, `ModernHouse.xml`, `LostCity.xml`, dungeons) | `MarkovJunior/models/`, port in `crates/studio_core/src/markov_junior/` | Port works (Apartemazements matches the reference at 84.7%; ModernHouse fails to parse). Not used for towns. |
| Sensor → brain → effector creatures, implemented | External: Godot Simulation Aquarium, `guppy.lua` (`/Users/paul/coding/atelico/godot_simulation/...`, cited in `docs/research/magical_physics/README.md` ~l.122) | Built elsewhere. Note: "effector costs are zero" there. |

**Take for Infinite Spell Game:** the parts for "everything is a spell" are already in the model:
- the caster/container/battery roles that "can coexist in one physical object"
- the `Instance/attachment` service ("Bind a recipe to a cast, item, status, child object or event participant")
- K Claim control and G Give goal effectors
- mana-metered Lua brains

A town, NPC or building would be a long-lived spell instance attached to an object. It has a mana reservoir, installed sensors, rules and effectors, and its brain cadence is paid in mana. This synthesis is new; the source repo does not state it.

## 5. Reusable vs Bevy-specific

| Item | Bevy? | Reuse for app engine |
|---|---|---|
| `crates/spellcraft` (engine, YAML schema, `WorldAdapter`, mana metering, Rapier adapter, history, Yrs collab, cloud) | No. "a reusable, renderer-independent Rust library in an isolated Cargo workspace" | **High.** Port the concepts and types. Swap Lua 5.4 for Luau (mlua supports both; spellcraft-media already uses Luau). Implement `WorldAdapter` over the app engine's world. |
| `crates/spellcraft-coordinator` (Rust/WASM Durable Object) | No | High, as-is |
| `crates/spellcraft-media` (Luau timelines) | No | Medium (trailer tooling) |
| YAML spells, `components.yaml`, recipes S01-S26, capability catalog | No | High: data and design |
| Day/night LUT values, moon orbit maths, MarkovJunior sky PNGs | Light (`Resource` derive, `Color::srgb`) | High: copy numbers and formulas |
| WGSL maths (altitude factor, horizon warmth, cloud scatter, twinkle) | Shader only | Medium: port the maths |
| MarkovJunior Rust port | Core logic is not Bevy (lives in `studio_core`) | Medium: buildings and dungeons |
| `studio_core` deferred renderer, sky dome node, `DayNightCyclePlugin`, sky sphere | Yes | Low |
| `studio_scripting` (Bevy + ImGui Lua) | Yes | None |
| `renderers/bevy` (bevy 0.17.3 spell viewer) | Yes | None. `renderers/html` is a browser reference. |

Process rules worth keeping:
- `AGENTS.md`: "UI and assistants call the same Rust services; Lua composes behavior/UI."
- `AGENTS.md`: "documentation proposals are not evidence of implementation."
- `AGENTS.md`: no parallel JS spell engine.
- `docs/HOW_WE_WORK.md`: the Library-Centric Rule.
