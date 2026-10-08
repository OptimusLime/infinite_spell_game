# Concept baseline

These files use the existing `atelico_concepts::Concept` schema from atelico-applets; no parallel concept model was introduced. They were created and resolved through the existing `atelico-concepts` executable, with one isolated library root.

The parent/world/link fields contain immutable `ConceptRef` revisions. `metadata/revisions/` preserves their exact canonical definitions. `../pins.json` is an index of the existing `ConceptRef` format, not a second revision system. No image assets are selected or generated.

The following existing string traits are the temporary design-review bridge:

- `decision.state`: In / Maybe / No / Future. Independent of task state and asset review.
- `decision.basis`, `decision.date`: why the design state was assigned.
- `variant.phase`, `variant.look`, `variant.identity`: compatible comparison intent.
- `compatibility.verification`: currently unverified. Pinning is not a compatibility test.

In means the requirement is accepted, not implemented. Maybe references remain unapproved even when their ancestor style is In. No excludes the named direction. Future is reserved for explicit deferral; none is silently invented in this baseline.

Read with the existing tool:

```sh
/Users/paul/coding/atelico/atelico-applets/target/debug/atelico-concepts --library /Users/paul/coding/infinite_spell_game/concepts/library resolve isg-pixel-night
```

Definitions should evolve by the existing branch/revision workflow. Do not overwrite immutable snapshots or equate reference relationships with approved gameplay.
