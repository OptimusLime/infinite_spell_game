# Focus while casting

Six exact native in-flight captures are imported into the References board: cel and pixel views, each with focus off, mild focus and a stronger stress case. These are unreviewed graybox mechanics studies. They are not final art, a proposed storyboard, or an approved gameplay view.

The source is engine commit `ca3b6bd0`, `proof/editor/casting-focus/`: its README, test log and 24 PNGs retain ready, in-flight and contact events, including equivalent world/camera focus planes. This folder contains the six board images, their [byte provenance](import.json), and a copy of the [complete state/comparison receipt](validation.json); the other 18 images and test source remain with the engine.

One actual native Fire action spends total cartridge power **80 → 60**, loaded power **40 → 20**, and changes target health **12 → 9** on contact. Each in-flight variant holds the same clock, cast receipt, aim, camera and geometry. The selected target, projectile and opaque HUD glyphs remain sharp. The stronger treatment reduces side-target readability, and even mild focus trades peripheral detail for emphasis. Focus remains off in production.

The effect uses a geometric focus plane with a single depth layer. It cannot reconstruct hidden backgrounds or independently focus transparent particles. Diagnostic single-frame timing is not the warmed renderer performance gate. No model calls or live game mutations were used.

[Native board verification](review-proof/validation.json) records category/drilldown/scroll navigation and side-by-side review at 1600×960 and 1100×800. All **67** board entries remain unreviewed with **zero** decisions. These screenshots show the native render target, not OS compositor acceptance.
