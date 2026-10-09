# Reserved-viewport review images

These are exact committed PNG/JSON copies from engine `c606fa07b14721013b30000bb202c031f9d573fc`, `proof/editor/player-viewport`. Eight day/night frames and one explicitly open Inventory drawer are imported; other native diagnostics remain in the engine proof directory. No image was resized, recolored or composited.

`import.json` identifies each image hash and the unchanged four board IDs. `validation.json` records the exact camera/Grade, actual World/dock rectangles and finite state for the eight day/night controls. `source-hashes.json` is the original source receipt. `pre159-current-cards.toml` preserves all four preceding card records; their images remain in `../player-readability/`.

The source owner validated core 13/13 and native 2/2, including direct drawer transitions, lifecycle and UI-only/missing-World behavior. See the engine's source proof README and logs for those checks. The Inventory image uses the restored original scene hour; it is not a matched day/night control.

The new World and controls are disjoint, but the viewport/aspect differs from the old overlay presentation. This is a separate isolated run, with one 80→60 paid launch and exact finite state held within the run. Camera/night/art decisions remain open. No review journal, live game or queue was changed by this import.
