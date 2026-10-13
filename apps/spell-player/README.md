# Spell player

A reusable native player that composes the shared Inventory, Equipment and Spellcraft Player. The scene contains a `SpellPlayer` node beside a scene-3d `World`; the component renders the actual physical bodies through that World's existing hooks. The small Luau renderer reads flat typed records through `data.object` and sends typed requests. It owns no game rules.

Open `scenes/spell-player.scene.luau` for the unreviewed engineering fixture. Equip the wand, load a cartridge, select an authored spell, aim at a target, then explicitly fire. Neither selection nor aiming spends energy. The bottom total comes from the core supply sum over owned item identities, including the loaded cartridge exactly once. Equipment and the shared Inventory grid open separately. A depleted cartridge remains an item and can be replaced; the previous cartridge returns to its explicit configured grid location.

The fixture's dimensions, capacities, spell allocations, body sizes and camera are test values, not accepted game balance or art direction. Targets are fixed selectable aim inputs: aiming captures the target's current direction; shots do not home. The aimed target is highlighted green. The current fixture has no walking, mouse-ground aiming or implicit firing. Production game integration remains separate (task104); this component does not replace the legacy game Town scene.

## Identity and state

`session::Registry` owns one `Player` per canonical configuration path and explicit session ID. Scene reloads, a different scene referencing the same identity, camera changes and backwards view clocks preserve Inventory, Equipment, selected authored sources, admission receipts and remaining cartridge energy. A new explicit session ID creates a new independent fixture. Closing a view does not refill it. Existing app unmount hooks remove only the closing view's rendered attachments; two views of the same session share the attachment until both close. The standard component inspection exposes these attachment counts for diagnostics.

Configuration initializes the session once. Subsequent configuration edits are detected and reported; changing it requires an explicit new session. Authored spell edits can be previewed and explicitly admitted into the existing session through Sources. Nothing silently replaces the running spell. Invalid edits preserve the current state. This is process-local persistence; it is not a durable save of the physical world across host restart.

`players/basic.player.toml` is a typed, checked file boundary: initial Inventory and Equipment, SupplySnapshot, bodies, fixed simulation step and allowed forms, source paths, and explicit return locations. No schema defaults invent capacities or physics parameters. Authored definitions are resolved relative to the config file and captured with their revisions.

## Typed binding

`view.rs` supplies `SpellPlayer`, `PlayerDashboard`, `PlayerRequest`, `PlayerUi` and `PlayerSpell`. Session paths encode canonical identity; children refer to existing normalized Inventory, Equipment, Supply and physical `BodyView` records. Schema installation occurs once per Store/session. The native callback retrieves the host's existing Store and Scope from Lua app data; it does not create another state store.

`PlayerRequest` has an increasing sequence and a rendered-state token. `Select`, `Equip`, `Load`, `Unload`, `Aim` and `Fire` are checked enums. Repeated sequences never replay; stale tokens reject. Only explicit Fire creates a cast admission, whose existing core identity checks guard custody, selected revision and expected remaining energy.

The generic Inventory controller's `handle_checked` hook validates the candidate against Equipment and Supply before publishing a move. Rejected moves preserve core Inventory and Store inventory fields, with typed interaction feedback. The physical world advances by bounded positive elapsed time. Rewinding the scene clock never reconstructs the session or replays funded requests.

`render.rs` draws low-poly engineering bodies for the core's Actor, Target, Bolt and Orb. It does not run the legacy Town, generated hotbar, fusion workflow or any model request.

## Verification

Run the isolated core suite with `cargo test -p component-spell-player --lib` and the native pointer proof with `cargo test -p atelico-editor --test spell_player`. The native test makes a private project copy and never edits a live queue or game inventory. Proof screenshots record their actual framebuffer dimensions under `proof/editor/spell-player/`.

## Keyboard commands

The caller may provide `[[keys]]` records in its player configuration: `key`, a checked `command`, and explicit `item_id`, `spell_id` or `target_id` where required. The fixture binds W to equip the wand, A/B to load its named cartridges, U to unload, 1/2 to select the named authored spells, Q/E to aim at the named targets, Space to fire, I to Inventory and G to Equipment. Each bound control displays its key; there is no inferred cycling, nearest-target aim or fusion action. Click the playing field or one of its controls to give it keyboard focus.

The editor supplies the existing held keys and authoritative focus as a transient native `KeyboardInput` snapshot through Script's Lua app data. Window keys stay under their existing Store/Scope; no second persisted input ledger exists. `KeyEdges` observes suppressed frames too: mounting while a key is held, returning focus, and releasing a command modifier do not become fresh presses. Text input, editor focus, agent focus and host-consumed shortcuts inhibit player actions. A physical release followed by a new press is required. The key map emits the same checked `PlayerRequest` as a button; keyboard Fire has the same funding, custody, revision and depletion checks. Multiple configured commands pressed together are rejected instead of silently ordering actions, and a pending pointer request is never overwritten.

The native keyboard proof uses `Editor::key`/`key_up` and verifies the finite loop, focus boundaries, repeat suppression, reload while held, pane shortcuts and exact selected target. A separate window regression verifies that one window's keys or focus loss do not mutate another's held keys.

## Reviewing an authored revision

Edit a configured YAML definition or optional Luau brain, open **Authored sources**, and preview its spell ID. The typed preview reports the current and candidate revisions, titles and energy allocations. **Admit this revision** installs that exact candidate for future casts; a file edit after preview rejects admission until it is previewed again. If that spell is selected, its next cast uses the new revision. Other configured spells remain unchanged.

`revision.rs` owns `PlayerRevisionRequest` and `PlayerRevisionView`. The dashboard supplies their normalized record references. A caller fills the operation (`preview` or `admit`), spell ID, expected current revision and, for admission, the rendered candidate revision and preview token; it advances `sequence` last. Duplicate sequences cannot replay. All source reads, parsing, world compatibility and stale-preview checks finish before any authored entry is replaced. The generic request path and finite cast checks remain unchanged.

Admission does not reconstruct Inventory, Equipment, Casting, the physical world or active instances. Earlier funded receipts retain their captured authored revision; a changed allocation applies only to a later explicit cast. The core regression compares the complete finite snapshots, equipment, physical bodies, tick and old receipts around preview, rejection and admission. The native pointer regression edits only a private fixture, rejects a stale preview, admits a 20→10 allocation change with 60 energy retained, then fires to 50. Screenshots are actual 1100×800 framebuffers.

The Sources capability is guarded by its typed schema so a source hot reload on an older running host keeps the previous player controls usable until its new producer is built. Scene panel wheel scrolling is not currently supplied by the host; the engineering fixture's revision controls fit without scrolling. Larger caller-defined lists need the shared scene scrolling support.

## Package checks and caller files

The package declares `cargo test --offline --quiet --lib`. Its test fixtures are source symlinks to the canonical engine fixture and Spellcraft authored sources; the existing `[files]` mappings materialize their bytes inside a staged package. Tests include only those package-local fixture paths and create a private caller configuration tree. No copied second source of fixture truth is maintained.

A consumer supplies its own scene and player configuration. `SpellPlayer.config` names the caller's file, `session` is a stable explicit identity, and `world` names the scene-3d World beside it. A configured definition/brain is resolved relative to that configuration's canonical parent directory. The engine fixture's `../packages/...` paths are examples for this checkout; a separately installed project should reference its own authored sources or its actual installed component paths. Installing the component does not rewrite the caller's game sources or silently seed a production inventory.

The HUD uses the engine's existing `style.crisp` layer, so source review, Inventory, Equipment and casting controls retain full-resolution text and pointer geometry when the scene world uses a pixel pass.

## Powered-capacity engineering fixture

Capacity is absent by default. A caller may explicitly use `[capacity] mode = "fixture_only"` with a nonempty list of fully specified `[[capacity.specs]]`; the package's `fixtures/capacity.player.toml` is an unreviewed mechanism fixture. The added pane shows the physical expander, existing Inventory grid, explicit source-cartridge activation buttons, paid world-tick boundary and the same account's cast/capacity debit history. It does not create another Inventory or another mana total.

Activation publishes the candidate grid, grant, shared debit and HUD supply rows in one checked Store batch before committing domain state. Missing/stopped/backwards clocks cannot make new capacity payments, and a stopped clock locks commands that could consume paid space. At expiry the fixture preserves occupied expanded cells and stops new Inventory/equipment/cast commands under `PolicyRequired`. This review interlock does not select the unresolved game no-power policy. No automatic renewal, offline billing, recharge, shrinking or item deletion is supplied. The optional state remains process-local with the Player session.

The normalized `PlayerCapacityRequest` is explicit: fill item/source identities, displayed configuration/balance/token, then increment `sequence` last. Refresh never submits a payment. The existing generic Inventory controller uses the aggregate's admission validation; there is no mana-specific grid implementation. Run `cargo test -p atelico-editor --test capacity_player` for the private native pointer fixture. See `docs/capacity.md` in the engine checkout for the core transaction and durability boundaries.

## Reserved battlefield and controls

`layout.rs` owns the presentation-only `PlayerLayout`, `PlayerRect` and `PlayerPane` contracts. Rectangles are normalized Data records keyed by the host's immutable `AppView` identity and exact SpellPlayer node path. The World and crisp HUD consume the same records; no geometry or inventory state is duplicated in Luau. `begin_view`/`end_view` retire layout records for removed nodes, and view release removes only its presentation records, not the funded Player session.

The control dock is outside the World rectangle. An explicitly opened Inventory, Equipment, Sources or Capacity drawer reserves a side rectangle; direct drawer switches display one pane at a time. The existing World renderer derives its aspect from that rectangle. Camera position, target, field of view, lighting and finite execution are unchanged. The dock retains the aggregate **Total energy**, separately labeled loaded cartridge, spell selection, aim, Fire and keyboard labels; errors remain visible in a scrollable status region. Large caller-defined control lists use the existing scene scrolling support.

The app matches one SpellPlayer to its configured exact sibling World ID. Missing or ambiguous matches leave controls available with a diagnostic and do not restyle any World. Older running hosts without the native layout callback retain the previous overlay presentation until rebuilt; script reload alone does not assume the new producer exists. The current pixel crisp pass still paints over the full design rectangle, so both the World placement and absolute crisp controls use the one computed layout rather than relying on hidden nodes to reserve flex space.

Run `cargo test -p atelico-editor --test player_viewport` for the isolated cel/pixel day/night proof at actual 1600×960 and 1100×800. It preserves one funded launch across view/resize/drawer changes, checks actual World placement and pointer controls, and exercises UI-only capacity activation and visible errors. See `proof/editor/player-viewport/` in the engine checkout. Layout and image evidence establish obstruction removal in these fixtures, not camera, art or gameplay acceptance.
