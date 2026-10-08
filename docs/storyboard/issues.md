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

## Round 4: the applied cards (re-judging round 3's fixes; queue task 12)
Round 3 fixes: 34 installed vs written 🟢 (all five state "six parts, four files"); 36 narrator named 🟢 (all five:
"Paul, who makes the Atelico editor"); 33 recipe vs AI 🟡 (G D read "the model invents"; J S: names only? bounded?);
35 clocks 🔴 (the 18:00 overlay mid-fight; dawn at 19:00). Queue task 15 makes three yellows into work, so they are
scored red here.

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 46 | Real vs game clock: "the clock says 18:00, with a part being installed. Is he installing it while the fight is going on?" | G J S | 🔴 | Card 5: "I skip ahead and play that night myself"; card 10: "still playing … while the night runs" |
| 47 | Passive climax: "the player watches a health ring drain, and the moons part on schedule"; peaks at card 11 (task 15) | J S | 🔴 | Card 12: the player's own fusion saves the seed |
| 48 | "Invents" reads as a name typing in; no bounds ("a demo that only works once") | J S A | 🔴 | Card 10: "invents what two spells make, as a new sensor and effect"; results write in as typed spell entries |
| 49 | Fire in the snow affects nothing; the weather session is never credited (task 15) | G J S A D | 🔴 | Card 7: "the weather session's fire … every burning flake I catch is a free fire spell" |
| 50 | Purple and orange creatures behave the same; the orange shadow does nothing (task 15) | S A | 🔴 | Card 8: "purple ones climb over walls, orange ones dig under them" |
| 51 | Why the townsfolk freeze (queue task 14) | G J S A | 🟡 | Three options written into the frozen-ink card's sketch_note for Paul; spoken unchanged |
| 52 | Card 2 dense; "adding the camera doesn't break the sky" unproven | G J S A D | 🟡 | Left (card 2 is held for Paul, task 13) |
| 53 | "The one rule no part has" awkward; "I ask it" ambiguous | G J D | 🟡 | "I ask Claude" folded into 46's edit; rest left |
| 54 | "Repo or it didn't happen"; price, model size | D | 🟡 | For the post text |
| 55 | "It didn't exist at nine" vs prebuilt parts | D A | 🟡 | Left |

## Round 5: round4-draft
Reds 46-50 all read 🟢: "I start with … every flake adds fire" (G: "burning flakes made me stop, in a good way");
climbers and diggers (all); "typed spell: name, sensor, effect" (S); the player casts the save (G J S A).

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 56 | The climax repeats card 10's combo ("tells me nothing new"); a ring of flame vs diggers underground; it should be "won by the thing just invented" | J S | 🔴 | Card 12: wind cast into the burning tower, "a pair I never tried"; the model invents a turning shadow that sweeps round the seed |
| 57 | Wind is never obtained: "the payoff feels cheated" | S A | 🔴 | Card 7: "I start with wind; every burning flake I catch adds a fire spell" |
| 58 | "Plays live" + real-time clock = one night lasting four hours: "reads like an error" | J S A | 🔴 | Card 5: "play that night, again and again, all afternoon"; the clock ticks between replays; card 10: "without restarting the night" |
| 59 | Card 10 overloaded | S A | 🔴 | "as a new sensor and effect" cut from the spoken line (the typed entry on screen carries it) |
| 60 | Same result every time? Cherry-picked? Show an uncut fusion with latency | S J D A | 🟡 | Partly covered by 56 (an untried pair, live); rest for the shoot |
| 61 | Card 4's bug (walking through walls) vs card 8's feature (climbing over, digging under) | J | 🟡 | Left |
| 62 | Moon town part vs the seed growing the town | J S | 🟡 | Left |
| 63 | Editor beats 2-5 are ~45-60 s before the first payoff; flash the ink cut in card 1 | J A | 🟡 | Left (card 1 is Paul's camera direction) |
| 64 | Townsfolk freeze, seed loss never shown | G S A | 🟡 | Options in the card note (51) |

## Round 6: round5-draft
Reds 56-59: the climax is now a new fusion the player makes (🟢: "the watchtower mutating live into a lighthouse beam
of fire", G; A: "the right climax image"); wind's source 🟡 (logic there, but all five still ask "when did I get
wind?"); card 10 trimmed 🟢; the clock 🔴 again (18:00 in card 5's replays and in card 10).

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 65 | 18:00 appears twice ("time goes backwards"); replays or one continuous night? | D J S A | 🔴 | Card 5's replay clocks are 15:30, 16:40; card 10: "Six p.m., mid-replay" |
| 66 | "Again and again" does nothing: no failed attempt, the stakes are never felt; the seed is never seen breaking | J A S | 🔴 | Card 5: "All afternoon I lose the seed, and replay it"; two quick losses on screen (the seed shatters, the town greys out) |
| 67 | Climbers vs diggers never matter (the beam kills both alike); card 12 crammed; the player casts once then watches | S J | 🔴 | Card 12 split: the beam burns the diggers (spells/lighthouse); the climbers are left on the roofs, and the player's last flake + wind puts a wall of flame along the rooftops (town/hold-the-seed) |
| 68 | "I start with wind": when? All five asked; "everything is a spell" arrives after it | G D J S A | 🔴 | Fire-in-snow moves after the watchtower card; "My own spell is wind"; the wind icon is on screen "all along"; set in the town square |
| 69 | Wall of flame and fireball look hand-authored; lead with the tower | A | 🟡 | Left |
| 70 | Card 4's panes unreadable; card 5's forecast won't read in 2 s; ~60 s of UI up front | A J | 🟡 | Left |
| 71 | Fused visuals vs "nothing off-style"; useless fusions? | S | 🟡 | Left |

## Round 7: round6-draft
Reds 65-68: clocks 🟢 (S: "the timeline works … the win needs Fusion, and Fusion arrives at 18:00"); climbers vs
diggers need two counters 🟢 (S: "card 13 pays that off well"); loss on screen 🟢; wind 🟡 (four still ask).

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 72 | Card 5's "replayed" / loss montage breaks the one night being followed; losses come before the threat is known; "the worst card" | G D J S A | 🔴 | Card 5 ends at the skip; the two losses move to card 10 as flashes: "I have lost this night twice" |
| 73 | Wind "all along" never set up | G D J S A | 🔴 | Card 8: "My wind is one; the watchtower is another"; card 9: "Every burning flake I catch is a fire spell" |
| 74 | The finish repeats card 10's wall of flame; the beam is the stronger image; the moons part as a rescue | J A | 🔴 | Order swapped: rooftop wall first (town/rooftop-wall), the lighthouse beam last, landing at the sliver as the moons part (town/hold-the-seed) |
| 75 | Orange is moon, diggers, fire and fireball at once in ink: fire on orange creatures turns to mush | A | 🟡 | Left (note for the look: fire white-hot) |
| 76 | "Ask your agent for it": for what? | G | 🟡 | Left |
| 77 | Orange shadow unexplained; snow, freeze, ink are decoration | S | 🟡 | Left |

## Round 8: round7-draft
Reds 72-74: card 5 now ends at the skip 🟢 (no judge lost there); the beam as the finish 🟢 (A: "the beam is the right
climax"; S: each creature type "has a clear answer"); wind 🔴 still.

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 78 | "Lost this night twice" arrives after we have watched the night go well; card 10 overloaded | S A J | 🔴 | The two losses move to the top of the night (card 6: "Third try at this night", flashes at 15:30, 16:40, then 17:50); card 10 keeps only the install and the two fusions |
| 79 | "My wind is one": what wind? | G J S A | 🔴 | Card 3: "You start with one spell: wind." (the wind icon in the player's one slot) |
| 80 | "Without restarting it": the game or the night? | G D J S | 🔴 | "while the night keeps running" |
| 81 | The model invents "sweep" exactly on cue; "a fusion I never tried" reads scripted; show a dud or a limit | S A D J | 🟡 | Left for Paul / the shoot (A: cut "a fusion I never tried") |
| 82 | The climax reads "the AI saved me" plus a timer | J | 🟡 | Left: the invented fusion saving the night is the point of the video |
| 83 | Climbers over walls vs the noon wall fix | S G | 🟡 | Left |

## Round 9: round8-draft
Reds 78-80 all read 🟢: no judge lost the night at card 10; "you start with one spell: wind" lands (wind no longer
"from nowhere" for D S A; G J ask only what wind's own effect is); "while the night keeps running" 🟢 (A: say "while
the game runs" 🟡). Every judge reconstructs the whole point: one day, six parts installed and four files written,
order-dependent fusion by an on-device model into typed spells, buildings fusable, the night won by the beam.

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 84 | Who wins: "the model's invention and the clock win, and the player is barely involved"; "was it skill, or a dice roll?" (queue task 15) | J S | 🔴 | Card 13: the player's reason and aim: "The tower's shadow can't reach them, so I cast wind into it to make it move … The last digger burns a step from the seed, and then the moons part." Applied; not re-judged |
| 85 | Fusion bounds: what stops a useless or unbalanced invention? Show one dud | S J D | 🟡 | Left for Paul / the shoot |
| 86 | Why do both moons make fire? | G J A | 🟡 | Left (weather agent's rule, queue task 5) |
| 87 | Townsfolk freeze | all | 🟡 | Three options in the frozen-ink card's sketch_note (queue task 14) |
| 88 | Noon wall fix vs climbers and diggers | J S | 🟡 | Left |
| 89 | Cards 2, 4, 5 ≈ 50 s of UI; cold-open on the ink flip | A J | 🟡 | Left (card 1 is Paul's camera direction; card 2 held, task 13) |
| 90 | "Ask your agent for it" + "$0" read as an ad; show line counts / raw footage | D J A | 🟡 | For the post text and the shoot |

Final: 14 cards, 434 spoken words (cap 555). Cards updated; path.toml reordered.

## Round 10: round10-draft (queue task 44)
Taken on before judging (coordinator's list): a dud fusion in card 10 ("fire then fire, a warm glow that does
nothing"); the weather rule in one line (card 9, the weather agent's rule: purple snow, orange embers, aligned
firefall); the noon bug is now monsters spawning inside houses (no wall contradiction); cards 4 and 5 are game-first
(panes as a strip, the forecast as a 3-second overlay), card 2's installs take two seconds (its spoken line
untouched).

Readings: card 13 🟡 (G: "'I' (Paul) plays it"; A: "the right image"; J: the clock and the AI still look like the
winner; S: "the one real skill moment is deciding to cast wind into the tower"); dud 🟢 (D: "a nice honest touch";
S: "shows the model can produce duds"); fire rule 🟢 (no one asked); wall fix 🟢 (no one asked); editor time 🟡 (G
J A still feel cards 2-5 as setup, ~40 s by A's count).

| # | Issue | Who | | Proposed fix |
|---|---|---|---|---|
| 91 | Card 10 too dense: install, model, dud, two orders, typed entries | J A | 🔴 | Split: spells/fusion-arrives (install, model, the dud) and spells/cast-order ("Order matters", big readable icons, entries flash for half a second) |
| 92 | Fire then fire needs two fire spells; we saw one flake caught | G S | 🔴 | Card 9 shown: the player catches two |
| 93 | The ring stops as the moons part: "rescued by a timer" | J S | 🔴 | Card 13 shown: the ring stops when the last digger burns; only then do the moons part, "the night's reward" |
| 94 | Open on the ink night | J A | 🟡 | Card 1 keeps Paul's direction; adds a one-second flip to ink and back after the walk |
| 95 | "Creature in my shadow" reads as the player's | A | 🟡 | "in its shadow" |
| 96 | The model invents the perfect counter on the first try; show a failed fusion near the climax | S D | 🟡 | Left (the dud is in card 10) |
| 97 | Wind and the tower: who made them? Moon town vs seed | D S | 🟡 | Left |
| 98 | Ad framing at both ends; "$0" into the voice | G D J A | 🟡 | Left for Paul / post text |

Applied: 15 cards, 439 spoken words. Fixes 91-95 not re-judged.

## Rounds 11-12: task 45, a failed fusion near the climax (15-card version, now history)
Round 11 put a dud (Emberheart) into card 14: 🔴 J S A ("repeats Hearthglow", "makes the player look clumsy", card 14
"carries too much"). Round 12 moved the failure to card 13 as a rushed wrong-order cast (wind then fire, the fireball
sails over the climbers): 🟢 J ("helps. It proves order matters under pressure"), S ("the best beat in the video");
A 🟡 (make the wrong order visible). Card 14 now one decision, one payoff: A "clean now". Left 🟡: the spell economy
(are spells used up?), the win reads as the model's luck (S), "Try 1/2" labels (A). Applied to the history cards.

## Reel v1 (Paul, 2026-10-08: "focus on ujique artistic choices and PUNCHY moments ... watch out for this corny shit")
The path is now a 10-shot, 36 s reel with no narration; the 15-card path is in docs/storyboard/history/.
Judges: an r/aigamedev regular (R) and a senior art director (A), on reel-r1/cards.md.

| # | Issue | Who | | Fix |
|---|---|---|---|---|
| R1 | Slow hook: a 7 s locked-off timelapse, nothing in the first 2 s | A (R: "if the first frame isn't static") | 🔴 | 1.5 s cold-open flip pixel->ink->pixel; timelapse cut to 5 s |
| R2 | Shot 7 overloaded (three casts, two names in 5 s); ends on the deflating Hearthglow | A | 🔴 | Hearthglow first (the gag, 2 s), then Wall of Flame as the payoff (4 s); Fireball cut |
| R3 | "made in the Atelico editor, with Claude" reads as a sponsor tag; the editor shot too small to read | R A | 🔴 | No burned-in tagline; a readable Claude prompt and the town flipping live is the shot |
| R4 | Two shadows too subtle at 3 s | R A | 🟡 | 4 s, closer push |
| R5 | Style flips faster and faster: trailer cliché | R | 🟡 | Left |
Liked: Hearthglow ("killed me lol", R), the burning flakes into snow ("the shot people will screenshot", A), the
two-shadow tint and the flip (A), the lamp ripple (R). Both: would upvote / share.
