# Storyboard issues, round by round

Judges each round: two naive viewers (interested gamer G, skeptical AI-curious developer D) and three senior devs
(juice/game feel J, systems S, art director A). Fresh agents each round, card texts only. Intent: `round1-intent.md`.

## Round 1: the current 10-card path (game trailer)
| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 1 | Who made it and how: "not said"; no editor, no AI, no working day. Reads as "a game trailer posted in an AI dev subreddit with the AI part left out … an ad dressed up as content" | G D J S A | 🔴 | Rebuild the path as the working day: clock times, the Atelico editor, Claude |
| 2 | Built-from-parts absent; "I learned nothing useful … no lesson about keeping the code from falling apart" | D | 🔴 | New card: Claude installs marketplace parts that fit together; each later part names the pain it fixes |
| 3 | No name, no call to action: "nothing to click" | G D J S A | 🔴 | Close: every part is in the marketplace, ask your agent; the game's name on screen |
| 4 | The seed's stakes arrive at card 9 (~50 s) | G D J S A | 🔴 | "If the seed breaks, the town is gone" moves into the seed card |
| 5 | Card 1 shows a finished town, card 2 rewinds to a seed | G D J | 🟡 | (covered by 1: "At nine this morning, it didn't exist") |
| 6 | Forecast rings before we care; "fire in snow" named before it's set up | G D J S A | 🟡 | (covered by 1: the forecast becomes the dev asking the sky part, then skipping ahead) |
| 7 | Why do the townsfolk freeze? | G D J S A | 🟡 | Left for now |
| 8 | "The colour of the moon decides" contradicts two moons being up; only orange shown | G D J S | 🔴 | "Each moon raises its own creatures. Tonight both are up" |
| 9 | Spells come from nowhere: who casts, no link to anything | G D J S A | 🔴 | Everything is a spell; the tower is a spell (sensor + effect); fusion fuses fire into the tower |
| 10 | "Dark side" never defined | G D J S A | 🔴 | Dropped; the shadow is each moon's coloured shadow on the ground |
| 11 | Too many mechanics, one line each | G J A | 🟡 | Left for now (13 cards, but each mechanic now arrives with the part that makes it) |
| 12 | Fusion shown as HUD icons, no character casting, no hit | J S A | 🔴 | The hero casts in the world: the wall of flame stops a line; the fireball bursts in a crowd |
| 13 | Card 9 is a busy blob, no single readable hit | J A | 🟡 | (covered by 9/12: one hit, creatures in the tower's burning shadow) |
| 14 | Real or AI mock-up? | D J | 🟡 | (covered by 1: editor footage; shoot gate) |
| 15 | Ink must be truly flat or it reads as a filter | A | 🟡 | (written into card 5's shown text) |
| 16 | Opening sky pan is slow for Reddit | A | 🟢 | Kept: Paul's camera direction, word for word |
| 17 | Fire weather and fire spells never touch | S | 🟡 | Left for now |

Strongest beats by consensus: the pixel-to-ink cut (G J A), fire hissing in snow (J A), the tower's shadow burning a
creature (J S A: "make it the climax"). Systems: "Make the spells feed it … or cut them" → fusion now feeds the
shadow.

## Round 2: round1-draft (13 cards, the working day)
Reading vs intent: day/editor/Claude 🟢 (all five); the game 🟢; built from parts 🟡; everything is a spell 🟢
(S: "the best idea in the video"); fusion 🟡; call to action 🟢. R1 reds 1, 2, 3, 4, 8, 9, 10, 12 all read 🟢.

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 18 | Built or installed? The seed has no part; fusion is "the last part" yet five parts were installed and six are listed; what did the sessions write? | G D J S A | 🔴 | Card 3: "The one rule no part has, Claude writes" (seed.luau); card 10: Claude installs the Fusion tile on screen |
| 19 | Two clocks: build time 11:00 vs "day 33, just after midnight" vs "At midnight" | J D | 🔴 | Game time said only as "in four game days"; "At midnight" cut |
| 20 | "No server, costs nothing per fusion": why would it cost? A pitch line that breaks the story; recipe table or AI? | G D J S A | 🔴 | Say what it is: "A small AI model on the player's machine invents the fused spell"; "no server · $0" moves to an on-screen label |
| 21 | The seed is never in danger; the health ring set up in card 3 is never used; card 12 passive | J S | 🔴 | Card 12: the ring drains to a sliver; the burning shadow turns the last wave; the moons part |
| 22 | Three agents + file lists at the peak of the action: unreadable UI; no visible result | G J S A | 🔴 | Agents move to noon, before the night; they get three named jobs and one visible result |
| 23 | No failure, no diff: "an ad dressed up as a devlog" | D | 🔴 | Card 4: the first build lets monsters walk through walls; one line changes in monsters.luau; the others keep working |
| 24 | Editor and game alternate six times; the game never builds momentum | A | 🟡 | (covered by 22: editor until 15:00, then the night unbroken except one Fusion install beat) |
| 25 | "window" ambiguous | G D | 🟡 | (covered by 22: "one per editor window") |
| 26 | Townsfolk freeze: never paid off | G J S | 🟡 | Left |
| 27 | Fire in the snow: mood, no payoff | G J S A | 🟡 | Left (Paul: "it's raining fire in a snowy time") |
| 28 | Shadow and creature colours never matter | S | 🟡 | Left |
| 29 | "Adding the camera doesn't break the sky" unproven / ad copy | J S D | 🟡 | Left |
| 30 | The narrator is never named | G J S A | 🟡 | Left: Paul's call |
| 31 | "Nothing comes out off-style" assumes viewers know AI art drifts; "sensor/effect" jargon | D A | 🟡 | Left |
| 32 | Open on the eclipse instead of the sky pan | A | 🟢 | Kept: Paul's camera direction |

## Round 3: round2-draft
Reading vs intent: day/editor/Claude 🟢; the game 🟢; built from parts 🟡 (the split "five parts plus one file" is
inferred, not said); everything is a spell 🟢; fusion 🟡 (authored or generated?); call to action 🟢. R2 reds 18
(seed written, Fusion installed on screen), 19 (game days), 21 (ring to a sliver: "that last one is the thumbnail"),
22, 23 ("the bug fix is the most believable part") all read 🟢; 20 half-fixed (see 33).

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 33 | Fire+wind looks like a fixed recipe, then "a model invents" it: "which one is it?" The seam undercuts the most novel claim | G J S | 🔴 | Card 10 says it once: fusion's small model, on the player's machine, invents what two spells make; each result's name writes in. Card 11: "The model invents a new tower" |
| 34 | "Every part in this town is in the marketplace" contradicts seed.luau and monsters.luau; "say plainly what Claude wrote and what was prebuilt" | G S A | 🔴 | Card 13: "Today: six parts installed, four files written by Claude"; the four files shown beside the six parts |
| 35 | Real or game clock: "Ten p.m." over the night; "stays in the game" then a cut back to the editor at 18:00 | G | 🔴 | "Ten p.m." and the 22:00 clock cut; the Fusion install becomes a corner overlay over the ink night |
| 36 | The narrator is never named, the vendor shows only at the end: "an undisclosed vendor ad" | D S (G J A in R2) | 🔴 | Card 2: "I'm Paul, and I make the Atelico editor." (Paul to approve) |
| 37 | "On-device · $0" caption mid-fight reads like an ad | A | 🟡 | (covered by 33/34: moved to the end card) |
| 38 | Townsfolk freeze: why? (third round running) | G D J S A | 🟡 | Left: needs Paul's rule (Paul: "the good guys are frozen in place. And so that means that ... your things are vulnerable") |
| 39 | Fire in snow affects nothing; the weather session is never credited | G J S A D | 🟡 | Partly covered by 34 (weather is one of the four files); left |
| 40 | Purple and orange creatures and shadows never behave differently | S | 🟡 | Left |
| 41 | "Adding the camera doesn't break the sky" and "nothing comes out off-style" are claims, not pictures | G J A D | 🟡 | Left |
| 42 | Editor stretch (cards 2-5, ~45 s) front-loads the dull part; card 4 dense; forecast panel is UI on UI | J A G | 🟡 | Left |
| 43 | Climax is passive: the tower saves the seed, not the player | J | 🟡 | Left (S: "the defence works without player input, which is the right ending for a systems pitch") |
| 44 | "I ask it": Claude or the sky part? | J S | 🟡 | Left |
| 45 | Nothing checkable: no repo, no session log, no price | D | 🟡 | For the post text, not the narration |

Round 3 was the last allowed. Fixes 33-36 are applied to the cards but were not re-judged.
