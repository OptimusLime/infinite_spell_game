# Game design direction — 2026-10-08

This is the current design handoff, not a new screenplay. Today's explicit user decisions take precedence over historical proposals, implementation conveniences and AI-written storyboards. The existing storyboard and reel are **unapproved**; their files remain intact as evidence. The aim is to make the actual game work and use that work for r/AIGameDev posts.

## Decision language

**In** means an explicit current requirement or an established source rule without a current contradiction. **Maybe** means a candidate needing comparison or a still-open design choice. **No** means explicitly excluded. **Future** means deliberately deferred, not promised or approved for implementation. These are design decisions, independent of `todo/doing/review/done`, asset selection and test results. No feature is placed in Future merely because an assistant can imagine it.

| Object | Decision | What is established; what remains open |
|---|---|---|
| Physical mana | In | Energy is harnessed, stored and released. A cast receives finite funding; actual sensing, logic and effects consume energy. Exact units and prices remain open. |
| Wand/battery | In | A wand has limited storage/supply roles. It may combine caster and reservoir; a separate physical battery object is not mandatory. Capacity, refill mechanism and rate remain open. |
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
2. Show the wand's available energy and the cast allocation. Admit only a cast the source can fund; debit the source and fund the instance once. An unaffordable cast gives visible feedback and performs no unfunded action.
3. Observe one funded cast through its physical result. Record sensing/logic/effect charges, remaining energy and any dissipation separately. A launched instance retains its transferred budget if its former supply is disconnected.
4. Continue until the limited supply prevents another cast. Time passing alone must not secretly restore it. Reload/refill interaction is an open follow-up, not a specified cartridge count, capacity or cooldown.
5. Repeat the same selected spell and event in pixel/anime and day/night comparisons. The presentation can differ; the finite-energy rules must agree. Capture source revisions and ledger evidence, plus continuous playable footage before calling the slice accepted.

This procedure has not been run. The engine port has mana and lifecycle execution, but [the port record](spells.md) explicitly excludes the source project's Rapier, history/review and collaboration implementation. Do not infer end-to-end crafting from the presence of mana code.

## Compare current frames with references

The off-path `storyboard/design-review/` cards show two frames from reel v5, three official reference candidates, the historical moon-clearing target, and the battery/camera review questions. They have empty narration and are not film shots. `storyboard/path.toml` and the cut stay unchanged. The existing Board includes off-path cards; Slideshow follows only the active path. Markdown/HTML exports append off-path cards under “Not on the path”; construction export lists them separately, not as shots.

| Review question | Current evidence | Measure on the next comparable capture |
|---|---|---|
| Can the battlefield be read? | Pixel at 3.000 s is largely sky/roofs; anime at 10.000 s crops the foreground actor and ground action area. These are offline cinematic frames, not a new live-editor measurement. | Record visible ground/action bounds, caster/target/threat visibility and occlusion at the chosen play viewport. |
| Is small/large scale legible? | Current close framing gives little shared space for comparison. | Compare the same camera event with recognizable scale anchors; preserve readable actors while widening. Do not choose numerical lens targets without comparison. |
| Does night remain readable? | Current footage provides the baseline; Octopath13 is only a candidate for warm pools in blue night. | Compare silhouette separation and locally motivated light at the actual play size, with the same event and exposure. |
| Does spell energy read? | DBZ4 offers a candidate silhouette and value grouping, not our gameplay. | Identify caster, trajectory and impact at play size; ensure light/bloom does not erase the shape or hide targets. |

Media provenance and copyright attribution are in [design-review sources](../storyboard/sketches/design-review/README.md). No image is labeled approved by being displayed. No new film has been generated.

## Object/revision bridge

[concepts/library](../concepts/library) uses the existing applets `atelico_concepts::Concept` and `ConceptRef` formats. IDs are stable; the existing Store generated SHA-256 revisions and immutable snapshots. Parent/world/link references pin exact revisions. [pins.json](../concepts/pins.json) lists those existing-format references.

`traits["decision.state"]`, `decision.basis` and `decision.date` carry the design decision. `variant.phase` and `variant.look` identify comparisons. This uses the existing string-trait extension point; it is **not** a new typed decision UI. Asset reviews remain `draft/selected/rejected`. Pinned links state which inputs belong together; **compatibility remains unverified** until the comparison is performed. The four scene baselines bind day/night environment, one visual style, wand/battery and camera revisions without changing the old film or old concept library.

## Sources and open decisions

- Latest user direction, 2026-10-08: the decisions summarized above; it overrides older notes.
- [Creature spell structure](/Users/paul/coding/creatures/creature_3d_studio/docs/research/magical_physics/SPELL_STRUCTURE.md): finite transfer and executed-work accounting settled; physical form and tariffs explicitly open.
- [Magical-physics source lore](/Users/paul/coding/creatures/creature_3d_studio/docs/research/magical_physics/source_notes/magical_physics_world_lore.md): energy infrastructure and dark/moon imagery; explicitly exploratory.
- [MarkovJunior reference](/Users/paul/coding/creatures/creature_3d_studio/MarkovJunior/VENDORED.md): inherited generation fixtures, not a requirement for voxel presentation.
- [Concept implementation](/Users/paul/coding/atelico/atelico-applets/concepts/README.md) and [organization](/Users/paul/coding/atelico/atelico-applets/concepts/ORGANIZATION.md): identity, study, ancestry, selected anchors and provenance remain distinct.
- [Detailed source map](design-sources.md): repository locations, historical alternatives, existing artwork and versioning conventions.

Open: energy units/capacity, allocation controls, explicit replenishment, crafting UI and saved selection, camera framing, day/night lighting, how Markov-generated environments enter the game, and acceptance thresholds based on comparisons. Existing town/fusion/creature/storyboard choices remain implementation history unless explicitly retained; this handoff does not approve them.
