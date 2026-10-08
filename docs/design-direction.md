# Game design direction — 2026-10-08

This is the current design handoff, not a new screenplay. Today's explicit user decisions take precedence over historical proposals, implementation conveniences and AI-written storyboards. The existing storyboard and reel are **unapproved**; their files remain intact as evidence. The aim is to make the actual game work and use that work for r/AIGameDev posts.

## Decision language

**In** means an explicit current requirement or an established source rule without a current contradiction. **Maybe** means a candidate needing comparison or a still-open design choice. **No** means explicitly excluded. **Future** means deliberately deferred, not promised or approved for implementation. These are design decisions, independent of `todo/doing/review/done`, asset selection and test results. No feature is placed in Future merely because an assistant can imagine it.

| Object | Decision | What is established; what remains open |
|---|---|---|
| Physical mana | In | Energy is harnessed, stored and released. A cast receives finite funding; actual sensing, logic and effects consume energy. Exact units and prices remain open. |
| Generic grid inventory | In | A reusable game system holds item instances within finite grid capacity. It must not be a mana-specific inventory. Grid dimensions, item footprints and stacking rules remain open. |
| Equipment | In | A separate reusable system equips an item such as a wand. Item identity and ownership must remain consistent with inventory; equipping must not duplicate an item or its energy. |
| Wand and physical cartridges | In | Equip a wand, then load and replace physical mana cartridges carried as inventory items. Mana is finite. Cartridge capacity, charging sources and transfer pricing remain open. |
| Total power display | In | The bottom panel reads generic inventory and sums the remaining energy of its cartridge items. A loaded/equipped reference must not count the same cartridge twice. Exact transfer/containment behavior needs an explicit contract. |
| Magical inventory expanders | In | Expanding carrying capacity consumes mana. Extent, rates and empty-power behavior remain choices; no values are specified yet. |
| Unlimited automatic recharge | No | Do not hide an infinite supply behind the mana display. Any later replenishment requires an explicit source and rule. |
| Craft and fire spells | In | The player crafts a spell, selects it and fires that selected spell. Neither auto-casting nor a fixed pair table establishes the requested crafting experience. |
| Pixel world | In | Keep pixel presentation; its implementation and exact resolution remain choices. |
| Anime/cel world | In | Anime-style cel shading with more Dragon Ball Z influence. A specific commercial game's frame is still only a reference candidate. |
| Voxel world | No | Historical voxel art is not the target. Useful generation or simulation algorithms can still be reused without adopting that appearance. |
| Battlefield framing and scale | In | Current views are too close. Show the battlefield and contrast small and large forms, including the user's zord-like scale reference. No new giant-enemy mechanic follows from this. |
| Tilt-lens settings | Maybe | Compare the requested sense of scale; focal length, tilt, focus and blur are not selected. |
| Day/night variants | In | Keep separate variants of the same objects, with pinned inputs. Palette/light parameters are not approved merely because a variant exists; “day” does not decide new sun lore. |
| Octopath II screenshots | Maybe | Assistant-selected references for spatial depth and local night light. Do not inherit tiny actors or heavy blur that hides play. |
| Sparking! ZERO screenshot | Maybe | Candidate energy silhouette/light-dark separation; Dragon Ball Z direction is In, this exact image and its close camera are not. |
| Historical moon-clearing image | Maybe | September general-direction acceptance is context, not approval of every detail today. Its voxel geometry conflicts with today's No. |

MarkovJunior generation and dark, brooding moon landscapes are important inherited work. Keep the generation experiments and motivated pools of light available for comparison. Do not silently promote old no-sun biology, intelligent energy, gods, broken moons, or newly invented enemies into current rules.

## First playable battery-casting slice

This is a proposed acceptance procedure for the requested behavior, not invented balance or a film narrative:

1. Craft or modify one spell through the existing authoring path; save the exact spell revision and select it. Firing must use that selected revision, not substitute a demo spell.
2. Put cartridge item instances in the generic grid inventory, equip a wand through Equipment, and load a physical cartridge. Show the bottom-panel sum of remaining inventory-cartridge energy, the loaded source and the cast allocation without counting an item twice. Admit only a cast the source can fund; debit that source and fund the instance once. An unaffordable cast gives visible feedback and performs no unfunded action.
3. Observe one funded cast through its physical result. Record sensing/logic/effect charges, remaining energy and any dissipation separately. A launched instance retains its transferred budget if its former supply is disconnected.
4. Continue until the limited cartridge prevents another cast. Time passing alone must not secretly restore it. Replace the depleted cartridge with another inventory cartridge and deliberately cast the selected spell again. Preserve the depleted item and its remaining energy; cartridge count, capacity, charging and cooldown values are not specified.
5. Repeat the same selected spell and event in pixel/anime and day/night comparisons. The presentation can differ; the finite-energy rules must agree. Capture source revisions and ledger evidence, plus continuous playable footage before calling the slice accepted.

The complete procedure has not been run. The reusable spell runtime now admits an immutable authored spell from a finite physical cartridge and executes an explicitly aimed cast in the existing physics world. Its tests cover exact debit, depletion, replacement, aim, contact and replay prevention. Native controls and the inventory/HUD connection are still being verified; neither these tests nor the engineering fixture establish end-to-end crafting or game acceptance. [The runtime contract](/Users/paul/coding/atelico/atelico-app-engine/docs/spell-supply.md) records these boundaries. The earlier [port record](spells.md) remains historical context.

The current town entry point makes the missing supply connection concrete: `packages/spellcraft/src/town.rs::start_definition` supplies `definition.mana.allocation` afresh to `runtime::cast`, without debiting a finite player reservoir there. `runtime::cast` then initializes and meters the funded instance. Instance accounting and player battery admission are separate requirements; the former does not establish the latter.

## Shared-system boundary

Inventory owns item identity, placement, grid capacity and transfers. Equipment owns equipped item references and equip/unequip rules. Both are independent reusable game systems. Cartridge energy and wand loading compose those systems; they do not define the inventory API. The game HUD reads authoritative inventory state. Any loaded-cartridge ownership transition must preserve identity and prevent duplicate energy totals. This is a required boundary, not a new bespoke game implementation.

The older immutable concept snapshots remain historical revision inputs. Their earlier wording that left battery form open is superseded by the cartridge decision above; retain their hashes rather than rewriting them in place. A new concept revision and compatibility review is tracked work. Reusable Inventory (98), separate Equipment (99), the native grid component (123), and finite cartridge admission/aimed execution (127) are implemented and tested. Native player controls (109), inventory/equipment/HUD composition (101), and the game-project finite casting proof (104) are now technically verified. The game proof preserves one finite session through debit, depletion, cartridge replacement and explicit saved-revision admission. Native authoring (135) has a passing create/edit/save/admit fixture; its published package and live installation are being completed. Funded-instance completion accounting (138) remains active work. The queue database is authoritative for subsequent status changes. Component tests are not a playable-game acceptance.

## Compare current frames with references


The native References tab loads [the research board](../references/visual-direction.references.toml): **31 external still images, 4 exact-frame motion studies with local clips, 9 project images, and 17 existing concept records**. Categories separate pixel environments, cel shading, casting, effects, inventory/UI, lens/framing, moon/day/night, and camera experiments. All candidates remain unreviewed. [Motion observations](../references/motion-study.toml), [byte provenance](../references/provenance.toml) and [technical comparison questions](../references/technical-study.toml) accompany the images. Current-versus-reference inspection and explicit versioned decisions use the shared, typed engine References component (107).

The six [camera experiments](../references/media/camera-studies/validation.json) hold time, actor pose and within-look geometry constant: close, wide, and wide with a focus band, for pixel and cel presentation. Wider framing exposes more ground and side approaches while shrinking the actor. The focus-band variant preserves the central half of the image exactly; it is a screen-space effect, not a physical tilted focus plane. These are framing studies without live aiming, spells or HUD. Pixel and cel still have separate geometry, and the cel scene has large dark areas that need readability work. No camera or lighting choice has been accepted.

The older off-path `storyboard/design-review/` cards show two frames from reel v5, three official reference candidates, the historical moon-clearing target, and the battery/camera review questions. They have empty narration and are not film shots. `storyboard/path.toml` and the cut stay unchanged. The existing Board includes off-path cards; Slideshow follows only the active path. Markdown/HTML exports append off-path cards under “Not on the path”; construction export lists them separately, not as shots.

| Review question | Current evidence | Measure on the next comparable capture |
|---|---|---|
| Can the battlefield be read? | Pixel at 3.000 s is largely sky/roofs; anime at 10.000 s crops the foreground actor and ground action area. These are offline cinematic frames, not a new live-editor measurement. | Record visible ground/action bounds, caster/target/threat visibility and occlusion at the chosen play viewport. |
| Is small/large scale legible? | Current close framing gives little shared space for comparison. | Compare the same camera event with recognizable scale anchors; preserve readable actors while widening. Do not choose numerical lens targets without comparison. |
| Does night remain readable? | Current footage provides the baseline; Octopath13 is only a candidate for warm pools in blue night. | Compare silhouette separation and locally motivated light at the actual play size, with the same event and exposure. |
| Does spell energy read? | DBZ4 offers a candidate silhouette and value grouping, not our gameplay. | Identify caster, trajectory and impact at play size; ensure light/bloom does not erase the shape or hide targets. |

Media provenance and copyright attribution are in [design-review sources](../storyboard/sketches/design-review/README.md). No image is labeled approved by being displayed. No new film has been generated.

## Object/revision bridge

[concepts/library](../concepts/library) uses the existing applets `atelico_concepts::Concept` and `ConceptRef` formats. IDs are stable; the existing Store generated SHA-256 revisions and immutable snapshots. Parent/world/link references pin exact revisions. [pins.json](../concepts/pins.json) lists those existing-format references.

The historical concept snapshots carry decisions in the original string-trait extension point (`decision.state`, `decision.basis`, `decision.date`) and comparisons in `variant.phase` and `variant.look`. They remain readable provenance, not the domain model for the new UI. The native References component uses typed Rust records/enums and observable Data/Store bindings for review state, pinned revisions and exact-pair compatibility. An explicit save records a new decision; opening, comparing or capturing a card records no approval. Asset reviews remain distinct from concept decisions. **Compatibility remains unverified** until reviewed. The four earlier scene baselines pin day/night environment, visual style, wand/battery and camera revisions without changing the old film or concept library; they do not approve separate worlds or a gameplay look-switch mechanic.

## Sources and open decisions

- Latest user direction, 2026-10-08: the decisions summarized above; it overrides older notes.
- [Creature spell structure](/Users/paul/coding/creatures/creature_3d_studio/docs/research/magical_physics/SPELL_STRUCTURE.md): finite transfer and executed-work accounting settled; physical form and tariffs explicitly open.
- [Magical-physics source lore](/Users/paul/coding/creatures/creature_3d_studio/docs/research/magical_physics/source_notes/magical_physics_world_lore.md): energy infrastructure and dark/moon imagery; explicitly exploratory.
- [MarkovJunior reference](/Users/paul/coding/creatures/creature_3d_studio/MarkovJunior/VENDORED.md): inherited generation fixtures, not a requirement for voxel presentation.
- [Concept implementation](/Users/paul/coding/atelico/atelico-applets/concepts/README.md) and [organization](/Users/paul/coding/atelico/atelico-applets/concepts/ORGANIZATION.md): identity, study, ancestry, selected anchors and provenance remain distinct.
- [Detailed source map](design-sources.md): repository locations, historical alternatives, existing artwork and versioning conventions.

Open design choices: energy units/capacity and prices, replenishment, depleted-expander behavior, camera framing, day/night lighting, how Markov-generated environments enter the game, and acceptance thresholds based on comparisons. Allocation editing and saved selection now have technical proofs; that does not settle their game design or presentation. Existing town/fusion/creature/storyboard choices remain implementation history unless explicitly retained; this handoff does not approve them.
