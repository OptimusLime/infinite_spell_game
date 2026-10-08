# In-game UI (HUD and game screens)

The game's own UI, drawn over the game in Play and in the scene view. Not the editor. Paul: "We are missing in game
UI elements, which is why the game looks so baren and werid" and "why would the forecast panel be in the editor?
thats part of the game bud."

Built as one engine package, `packages/game-ui` (scene node `Hud`, Luau kit `game_ui`), in two looks: **pixel** (the
cosy town) and **ink** (the frozen other side). The HUD swaps look on the same frame as the world does.

## Rules and budget

- **One question per element.** Each element answers one question the player asks every few seconds. If it doesn't,
  it isn't on screen.
- **At rest: 4 elements** (seed, sky dial, spell bar, forecast chip). **At most 6** with contextual ones (one
  building card, plus the alignment banner; the chip hides while the banner shows; the fusion line is part of the
  spell bar). World-space marks (creature pips, numbers, the prompt, edge arrows) don't count; they live on the thing
  they describe.
- **Screen:** the HUD stays in a 6% border band and the bottom 18%; the middle 60% of the picture is the game. The
  hero is never covered (third-person's keep-in-frame bands already reserve the bottom).
- **Colours. Pixel: 8.** Ink outline `#0b0a14`, frame dark `#2a2340`, frame light `#6a5a8c`, paper `#f0e6d2`,
  purple `#9a74d0`, orange `#f4a440`, seed blue `#a8c8ff`, danger `#e04848`. **Purple and orange mean the two moons
  and nothing else, in both looks**: never danger, never an element, never urgency. **Ink: 4.** Black `#0b0a14`, paper
  `#f4efe4`, purple, orange (the card says purple and orange are the only colours on that side). **Danger in ink is an
  inversion**: the plate turns paper with black marks and flips black/paper 3 times in 180 ms; danger in pixel also
  carries a glyph (a cracked pip), so it never relies on red vs orange alone.
- **Elements are never colour-coded.** Fire, wind and frost are told apart by silhouette (flame, swirl, crystal) in
  paper; the moons own the only two hues.
- **Type.** Pixel: VT323, sizes at whole multiples of the pixel grid, no anti-aliased scaling. Ink: a heavy clean sans
  (Atelico Type) for numbers and names, regular for small labels.
- **Motion budget.** At rest nothing moves except the seed's slow pulse and the moons along the dial. Motion means
  something changed. Pixel motion scales by whole pixels (2× for 2 frames, then 1×), never a blurred 1.4×; ink has
  no fades (flat 4 colours): it snaps or brush-wipes out in 80 ms.
- **Same anchors in both looks.** Each element has a fixed box and right-aligned tabular figures, so the look swap
  never reflows a number.
- **Readable at 720p and on a phone:** smallest text 18 px at 720p; every icon on an 8 × 8 (pixel) or 16 × 16 (ink)
  grid, drawn as crisp cells, never a scaled bitmap.

## The two looks

| | Pixel world | Ink world |
|---|---|---|
| Frame | 9-slice pixel frame: 1-cell outline with notched corners, 1-cell light bevel top-left, dark fill at 85% | Flat black plate, 3 px paper keyline inset, plus a 3 px paper keyline outside so it never melts into the black-lined world |
| Fill | Frame dark, slightly transparent so the warm town shows through | Solid black (the world is flat, so the HUD is flat) |
| Text | VT323, paper, 1-cell drop shadow in outline colour | Atelico Type, paper on black; key numbers in purple or orange |
| Icons | 8 × 8 cell sprites | 16 × 16 cells, read as clean shapes |
| Motion | Steps (frame-by-frame, 8–12 fps feel), no easing blur | Snaps and smears: overshoot, ink-brush wipe-ins |

## Elements

| # | Element | Shows | When | Where | Pixel | Ink | Animated |
|---|---|---|---|---|---|---|---|
| 1 | **Town seed** | The seed's health as 3 arcs of 8 pips round a crystal icon, and "18/24" | Always, once a seed is planted | Top-left | Square pips in seed blue on a pixel frame; lost pips go frame dark | Round paper pips; lost ones hollow | Slow pulse (1.6 s) of the crystal, faster as pips fall (6 left 0.8 s, 3 left 0.55 s, 1 left 0.4 s). A hit: the frame shakes 140 ms (2, 1, 1 cells, at most once a second), lost pips flash danger, hold 350 ms, then drain 50 ms apart (a ghost bar). Crossing 25% is its own beat: 2-frame flash and a warning sound. The last pip: heartbeat and a 6% danger vignette. The seed breaks: 300 ms freeze, world at 0.25× for 1.2 s, the ring shatters into its 24 pips |
| 1b | **Seed arrow** | An arrow on the screen edge pointing at the seed; one small arrow per brood heading for it (purple or orange) | Seed off screen and a creature within 12 m of it, or 3 s after a hit | The 6% edge band | 2-cell arrow, danger with a crack glyph | Paper arrow, inverted plate | Pulses at 2 Hz. When the seed is on screen, a world-space ring of pips over it mirrors the HUD ring instead |
| 2 | **Sky dial** (two-moon clock) and night counter | The horizon with the sky above and a dimmed band of ground below, each moon on its own arc where it is now (real `runtime.environment`: `moon_turns`, heights); a moon under the horizon sits as a dim pip on the lower arc. The hour and "NIGHT 3" | Always | Top-centre | Arc of dots, two 3-cell moons in purple and orange, a horizon line; hour in VT323 | Two thin arcs, the moons as flat discs with black outline | Moons move with the world clock. Within 25° the gap between them glows (no number: the chip owns the time); in the last 10 real seconds before an alignment the glow ticks from 1 Hz to 6 Hz |
| 3 | **Forecast chip** | Now's weather icon and "ALIGN 2d 4h" (time to the next alignment), with the key [F] | Always, except while the banner shows | Top-right | Small pixel frame | Black tab | The countdown ticks; under one game hour its plate inverts once and the countdown slides into the banner (300 ms) |
| 4 | **Forecast panel** | The coming days: each moon's hours in the sky, each night's weather, the alignments and what they bring (content from the weather lane), and a fusion log page | Opens on [F]. In calm hours the game pauses; during an alignment or an attack it opens read-only at full speed | Centre, 70% of the picture, over a darkened game | Big pixel frame, ruled day columns | Full ink plate, one column per day | Opens with a 3-step frame grow (pixel) or a brush wipe (ink); the next alignment's column pulses twice, then holds highlighted |
| 5 | **Spell bar** with cast-order fusion | 3 slots (keys 1 2 3) and a 4th **fusion slot** (always the last fused spell, replaced each fusion, never your own slots). Each slot shows the spell's **shape** (bolt, wall, orb) with its **element** badge (flame, swirl, crystal) in the corner. Above, one fixed row: the cast order as it is built ("FIRE →", then "FIRE → WIND"), and the result "= Wall of Flame" | Slots always. The fusion row from the first cast of a pair: a ghost "FIRE → ?" as soon as one element is armed; with the second selected, it previews "= ???" for an unknown pair or the name for a known one | Bottom-centre | 8 × 8 icons, the selected slot's frame in paper; the fusion slot has a double frame | 16 × 16 icons, selected slot's keyline in paper, the fusion slot a double keyline | Cast: slot punches 2 frames down, 4 back with a 1-cell overshoot. Chip 1 pops the moment the first element is cast (2× for 2 frames); the arrow wipes in 120 ms as the second starts; casting the other order flashes the arrow the other way. **The reveal** (first time a pair is found): the world holds 80 ms and bursts at the cast point; the chips slide together (300 ms) and "= ▮" blinks at 4 Hz while the model on the machine names it (never less than 350 ms; at 1.5 s a fallback name from shape + element is stamped and the model's name swaps in when it comes); the name types at 18 letters/s (names capped at 20 letters), holds 900 ms, then flies on a 380 ms arc into the fusion slot and lands with a 2-frame squash and paper flash. A known pair: chips then the name stamped whole, 400 ms |
| 6 | **Creature marks** | A row of up to 5 pips over each hurt creature, in its moon's colour, on a black backing | Only after it is first hit; hidden at full health | World space, above the head | Square pips | Round pips with black lozenge backing | Pop in over 2 frames on the first hit; drain 60 ms apart with a paper flash |
| 7 | **Damage and effect numbers** | The number dealt, or the effect word (SLOW, BURN) the first time it lands on a creature | On hits | World space, at the hit | VT323 in paper | Heavy sans in paper; big on crits | Pixel: 2× for 2 frames then 1×, rise 24 px in steps, gone at 0.7 s. Ink: rise 16 px in 2 steps, snap out. Hits on one target within 300 ms merge into one rolling number; at most 6 on screen |
| 8 | **Interaction prompt** | Key + verb + target: "[E] Plant seed", "[E] Enter Bakery", "[E] Cast into Watchtower"; the target building gets an outline in the world | When the hero is within reach of something usable | World space, over the target | Key cap as a 2-cell bevelled square; paper outline on the target | Key in a paper circle; thick keyline on the target | Slides up 4 px over 120 ms on appear, then holds still. Hidden during a fusion reveal |
| 9 | **Building card** | The building's name and its spell, one line each: "Sensor: creature in my shadow", "Effect: slow", "Action: …" (paper; no moon colour unless the line names a moon's shadow); its sensor area drawn on the ground while selected | On select (click, or the pad's face button on the targeted building); opens by itself when a spell is cast into the building | Bottom-left, above the bar's line | Pixel frame with a header strip | Ink plate with keylines | Slides in from the left in 4 steps. When a building's sensor fires: its sensor icon flashes over the roof (3 frames) and a 1 px line in its shadow's moon colour reaches the target (150 ms). Cast into it: the world holds 80 ms, the outline pulses twice, the new effect line plays the fusion reveal |
| 10 | **Alignment banner** | "THE MOONS ALIGN · FIRE IN THE SNOW · BOTH BROODS RISE", then "ALIGNED" | One game hour before an alignment and while it lasts | Full-width ribbon under the sky dial for 3 s, then a slim tab under the dial | Purple and orange chequered ribbon edge | Hard black band, the two moon colours as stripes | Unrolls from the centre; the text flashes twice, then holds and shrinks to the tab. Last 10 real seconds: a big number stamps each second (2× for 2 frames, tick). **The switch:** 1 frame paper, 2 frames black, 120 ms freeze, then a brush stroke from where the moons meet on the dial wipes world and HUD to ink together (350 ms); the banner re-inks in place and flashes once |
| 11 | **Dawn summary** | Night N survived: seed health kept, creatures stopped, spells fused (with names), buildings that fought | At dawn after a night with a wave, after the thaw (pixels spread out from the seed, 800 ms) | Right third, clear of the hero | Pixel scroll card | Ink plate (the last ink frame before the world thaws) | Each line counts up in at most 300 ms (1.5 s in all), fused names retype at 18 letters/s; holds at least 3 s, any input skips; dissolves in 8 steps. Never blocks casting |

**Sound sync points** (frame-locked events the audio lane hooks): the alignment impact frame, each typed letter of a
fusion name (pitch ±2 semitones), landing in the fusion slot, a seed hit, crossing 25%, each dial tick inside 25°,
each second of the last 10, each dawn tick and a resolving chord at the end.

## What each element is for (why it stays)

1. Seed and seed arrow: the lose condition, and where it is. If the player can't see it they can't defend it.
2. Sky dial: the two cycles are the game; the player has to read both moons at a glance to plan.
3. Forecast chip: the one number that matters (time to the next alignment) without opening anything.
4. Forecast panel: planning ahead is the core loop between nights; it lives in the game, not the editor.
5. Spell bar and fusion line: the key feature (fusion by cast order) must be visible: what went in, in what order,
   and what came out, and the order must be learnable before the cast (the ghost line).
6–7. Creature marks and numbers: the player needs proof a spell worked.
8. Prompt: the only verbs outside combat.
9. Building card: buildings are spells; the player must see the sensor and effect to place and fuse them.
10. Banner: the alignment is the big event; it must not be missed.
11. Dawn summary: the reward beat and the moment fusions are remembered.

## Not in the HUD (on purpose)

No minimap (one town, the camera shows it), no XP bar, no quest log, no mana bar until the spell core decides on
mana (then it joins the spell bar as a thin strip, not a new element).

## State it reads

| Element | Source | Today |
|---|---|---|
| Sky dial, night counter | `runtime.environment.<world>` (hour, moon_days, moon_turns, purple/orange lights, aligned) from the World's Sky | Real |
| Forecast chip | the twin moons' schedule (packages/sky) via the moon-forecast / weather lane | Real when its Luau API is linked; otherwise derived from the environment, marked "stub" |
| Weather icon | the weather lane (`weather.at`) | Real when linked; otherwise from the moons' heights |
| Seed health | the town seed (TownSeed) | **Stub**: `Hud` prop `seed_health` |
| Spells, fusion | spell core (not ported) | **Stub**: `Hud` props `spells`, `fusion` |
| Creatures, numbers, prompt, building card, dawn | game logic (not written) | **Stub** props, shown only when set |

## Judging the description (round 1)

Two fresh judges, no files, everything in the prompt: a senior game UI/UX designer (UX) and a juice/game-feel
designer (J). Each red is fixed in the element rows above; the yellows that were cheap are folded in too.

| # | Complaint | Judge | Was | Now |
|---|---|---|---|---|
| 1 | Orange means danger in ink (and urgency on the chip): breaks "purple and orange are the moons" | UX1, J6 | 🔴 | 🟢 danger is an inversion (paper plate, black marks, 3 flips in 180 ms) plus a crack glyph; the chip inverts instead of going orange |
| 2 | Element badges have no colours that fit: fire reads as the orange moon, frost as the seed | UX2 | 🔴 | 🟢 elements by silhouette only, in paper; rule stated |
| 3 | Building card colours effects by moon with no rule | UX3 | 🔴 | 🟢 paper; moon colour only on a line that names a moon's shadow |
| 4 | Fused spell overwrites slot 1 | UX4, J4 | 🔴 | 🟢 a 4th fusion slot, replaced each fusion |
| 5 | Cast order can't be discovered before casting | UX5, J3 | 🔴 | 🟢 ghost "FIRE → ?", preview "= ???" or the known name, chips pop at cast, the reversed order flashes the arrow |
| 6 | No sense of where the seed or the threat is | UX6, J7 | 🔴 | 🟢 seed arrow and brood arrows on the edge band; a world ring over the seed when on screen |
| 7 | Fusion line and prompt collide above the bar | UX7 | 🔴 | 🟢 prompt moves to world space over the target; fusion row fixed |
| 8 | Fusion reveal too fast (430 ms) and away from where the eyes are | J1 | 🔴 | 🟢 80 ms hitstop + burst at the cast point, "=" beat, 18 letters/s, 900 ms hold, 380 ms arc; repeats stamp in 400 ms |
| 9 | The on-device model's naming delay isn't designed for | J2 | 🔴 | 🟢 swirl + blinking cursor, 350 ms minimum, fallback name at 1.5 s, quiet swap |
| 10 | Alignment switch: no build-up, no impact | J5 | 🔴 | 🟢 10 s ramp (tick 1→6 Hz, big countdown), paper/black impact frames, 120 ms freeze, brush wipe from the dial |
| 11 | Losing the last pip has no beat | J8 | 🔴 | 🟢 heartbeat + vignette at 1 pip; break: freeze, 0.25× world, the ring shatters |
| 12 | Target of "[E] Cast into tower" ambiguous | UX8 | 🟡 | 🟢 names the building, outlines it |
| 13 | Budget doesn't add up at an alignment | UX9 | 🟡 | 🟢 chip hides under the banner; fusion row counts as the bar |
| 14 | Full-width banner stays up all fight | UX10 | 🟡 | 🟢 3 s full, then a tab |
| 15 | Countdown shown twice (dial and chip) | UX11 | 🟡 | 🟢 the chip owns the number; the dial only glows |
| 16 | Half-circle dial hides a moon below the horizon | UX12 | 🟡 | 🟢 ground band with the moon as a dim pip |
| 17 | 24 pips in a ring won't stay crisp | UX13 | 🟡 | 🟢 3 arcs of 8 |
| 18 | Slowing time on the forecast is abusable | UX14 | 🟡 | 🟢 pauses only in calm hours |
| 19 | Ink HUD melts into the ink world | UX15 | 🟡 | 🟢 outer paper keyline; creature pips on a black backing |
| 20 | Look swap reflows text | UX16 | 🟡 | 🟢 fixed boxes, tabular figures, same anchors |
| 21 | Numbers spam; can't "never overlap" | UX17, J13 | 🟡 | 🟢 merge within 300 ms, cap 6, effect word first time only |
| 22 | Card misses the spell's action; sensor area invisible | UX18 | 🟡 | 🟢 Action line; sensor area drawn on the ground |
| 23 | Dawn card covers the hero, competes with the thaw | UX19, J18 | 🟡 | 🟢 right third, after the thaw, skippable |
| 24 | Seed hit and under-25% under-specified | J9, J10 | 🟡 | 🟢 ghost bar, timed shake, escalating pulse, 25% beat |
| 25 | Chip-to-banner handover vague; banner doesn't change at the switch | J11, J12 | 🟡 | 🟢 countdown slides in; banner re-inks, "ALIGNED" |
| 26 | Pixel 1.4× scale blurs; ink fades break the flat look | J14 | 🟡 | 🟢 whole-pixel 2× steps; ink snaps |
| 27 | Prompt bob breaks the motion budget | J15 | 🟡 | 🟢 no bob |
| 28 | Building spells firing are invisible | J16, J17 | 🟡 | 🟢 sensor icon over the roof, a moon-coloured line to the target; card opens on a cast-in |
| 29 | No audio sync points | J19 | 🟡 | 🟢 listed |
| 30 | Slot keys, "white" flash not in palette, shake spam, red/orange colour-blindness, long names, forecast blink | UX20, J20 | 🟢 | 🟢 keys 1 2 3, paper flash, one shake a second, crack glyph, 20-letter cap, pulse twice then hold |
