# Weather and moons as gameplay

Research for a game where two sky bodies (a sun and a moon, or two moons) cycle at different speeds. A forecast shows when they overlap. Each overlap makes weather or an event (fire falling during snow, say) and brings monsters up out of the ground, coloured by the moon. The player defends a town seed.

Researched 7 October 2026, using about 60 web searches and page fetches. Every number has a source and date. **(est.)** marks a third-party estimate or my own inference. Known gaps are listed at the end.

---

## 1. The game you half-remembered: it is **Icarus**

You described "a Rust-like survival game with storms, one-word name, Ichabod or similar". That is **Icarus** (RocketWerkz, 2021). It is a survival-crafting game from Dean "Rocket" Hall, who made DayZ. Storms are its signature feature.

- **How it works.** Each biome has its own storm: rain and lightning in forest, blizzards in arctic, sandstorms in desert. Storms run in phases: Start, Damage, Chaos, End. From medium strength up, lightning sets thatch and wood on fire and knocks trees over. Heavy rain wears buildings down over time ([GamePretty, 9 Sep 2021](https://www.gamepretty.com/icarus-weather-guide-all-you-should-know/); [TheGamer, 15 Dec 2021](https://www.thegamer.com/icarus-tips-tricks-surviving-storms/)). Out in the open, an Exposure meter fills up and gives you worse and worse ailments until you reach shelter.
- **How it's telegraphed.** A Weather Event Timeline sits in the top right under the temperature. A red "STORM INCOMING!" bar names the storm type. Coloured bars slide from right to left, and the storm starts when a bar reaches the left edge ([TheGamer](https://www.thegamer.com/icarus-tips-tricks-surviving-storms/)). The **Week 60 update** replaced this with a forecast meter. It predicts storm strength over the next 2 in-game hours "with some inaccuracy". It has 7 strength levels and **no longer tells you the storm type** ([eip.gg, Week 60 notes, early 2023 (est.)](https://eip.gg/icarus/news/icarus-week-60-update-overhauled-weather-forecasting-system-and-30-minute-backups/)). In other words, the studio made the forecast *less* precise on purpose, to keep tension.
- **What players loved.** A storm is something you plan your build around. Stone versus wood, caves, carving shelters into boulders: the weather decides your architecture.
- **Numbers.** It hit #1 on Steam's global top sellers at launch (Dec 2021) and passed 1M copies soon after ([PCGamesInsider](https://www.pcgamesinsider.biz/news/72704/charts-dayz-creators-icarus-debuts-at-no1-on-steam)). Hall reported **984,000 base-game copies sold in the late-2025 Steam Winter Sale alone**, plus 243,000 DLC buyers. Concurrent players went from about 9k (Nov 2025) to 35k+ (Dec 2025) ([GameGeeker summary of Hall's post](https://gamegeeker.com/games/icarus)).

---

## 2. Game-by-game survey

### Rust (Facepunch)
- **Cycle.** Weather is driven by artist-made presets (Clear, Dust, Fog, Overcast, RainMild, RainHeavy, Storm) that the game blends between. Server owners can change the odds of each ([Facepunch wiki](https://wiki.facepunch.com/rust/Weather)). Weather mostly sets mood and sight lines, and makes you wet or cold. It does not drive events.
- **Lesson.** Fog and storms in a PvP game act as *cover*, letting raids happen under weather. But Rust has no forecast, so players can't plan around it.
- **Numbers.** 16M+ copies sold on all platforms ([Destructoid, Jan 2024](https://www.destructoid.com/rust-has-sold-more-than-16-million-copies-since-releasing-10-years-ago/)). 20M+ on Steam alone ([KitGuru, 7 Jul 2025](https://www.kitguru.net/desktop-pc/mustafa-mahmoud/rust-has-sold-over-20-million-copies-on-steam-alone/)).

### Valheim (Iron Gate)
- **Cycle.** Weather is rolled for the whole world every 666 seconds. Each biome reads that same roll as its own weather, so light rain in Meadows means a thunderstorm in Black Forest at the same moment ([r/valheim data-mine thread, Oct 2021](https://redlib.groet-infra.nl/r/valheim/comments/qepzf0/weather_system/hhvh5i5/?context=3)). Weather gives Wet, Cold and Freezing debuffs. In Ashlands, glowing cinders rain from the sky and hurt anyone outside ([Supercraft wiki](https://supercraft.host/wiki/valheim/ashlands/)).
- **Raids** are announced by a text line and music, e.g. "The ground is shaking" means trolls are coming ([TechRaptor](https://techraptor.net/gaming/guides/valheim-forest-is-moving-base-raids-guide)). This is close to your "monsters rising" idea, and players remember these lines word for word.
- **Numbers.** 10M copies in about 14 months ([Game World Observer, 25 Apr 2022](https://gameworldobserver.com/2022/04/25/valheim-surpasses-10-million-units-sold-in-14-months-since-its-launch)).

### Don't Starve / Don't Starve Together (Klei)
- **Cycle.** Four seasons, each with its own threat: winter freezing, summer overheating, spring rain and wetness. Each season has a giant boss: Deerclops (winter), Moose/Goose (spring), Bearger (autumn). In DST, Deerclops comes on about day 30.8 by default ([Don't Starve wiki, Seasons](https://dontstarve.wiki.gg/wiki/Seasons); [PlayWithTools](https://playwithtools.com/post/seasonal_giants_driving_you_mad_don_t_starve_spawn_mechanics_simplified)).
- **Hounds.** Hound attacks come every 6–13 days early on, shrinking to every 3–8 days after day 100. You get **120 seconds** of growling that gets louder, falling to **30 seconds** later in the game. Your character also says out loud: "Did you hear that?" The hound's colour matches the season: red hounds in summer and autumn, blue in winter and spring ([wiki, Periodic hound attacks](https://dontstarve.wiki.gg/wiki/Periodic_hound_attacks)). This is the clearest example out there of **season colour → enemy variant**.
- **Numbers.** 1M+ in its first year (2013) ([MCV](https://www.mcvuk.com/business/dont-starve-surpasses-1m-players-klei-mulling-vita-and-mobile)). About 2.5M (DS) and about 9.6M (DST) on Steam (est., [Raijin](https://raijin.gg/app/322330/Dont_Starve_Together), 2026).

### Terraria (Re-Logic): the richest moon menu
- **Moon phases.** 8 phases, one per night. They change shops, some spawns and fishing ([wiki](https://terraria.wiki.gg/wiki/Moon_phase)).
- **Blood Moon.** A 1-in-9 chance each night, once a player has more than 120 max health, and never on a new moon. Text in the lower left reads "The Blood Moon is rising...". **All water in the world turns red, rain included.** The surface gets a red filter, enemies spawn near towns, and zombies can open doors ([wiki via search](https://terraria.wiki.gg/wiki/Event)).
- **Solar Eclipse.** A 1-in-20 chance each day in Hardmode, or you can summon one. The sun turns into an annular ring and the day goes dark. Every enemy is a nod to a 20th-century horror film (Mothron, Reaper, and so on) ([wiki](https://terraria.wiki.gg/wiki/Horrors)).
- **Pumpkin Moon / Frost Moon.** You summon these with an item. Each has 20 scored waves until dawn. **The moon itself turns into a jack-o'-lantern** ([wiki, Pumpkin Moon](https://terraria.wiki.gg/wiki/Pumpkin_Moon)). Each has a progress bar; Blood Moon and Eclipse don't.
- **Lunar Events.** Four Celestial Pillars, each with its own enemy family. Then "Impending doom approaches...", and the Moon Lord arrives one minute later ([wiki](https://terraria.wiki.gg/wiki/Moon_Lord)).
- **Lesson.** **A recoloured moon tells you which monster set is coming.** That is your "monster colour = moon colour" idea, already proven at huge scale.
- **Numbers.** 70M copies (39.6M PC, 10.7M console, 19.7M mobile) ([Inven Global, 17 May 2026](https://www.invenglobal.com/articles/21948/15-year-old-terraria-surpasses-70-million-cumulative-sales)). About 90 hours average play per player (same source).

### Minecraft (Mojang)
- **Cycle.** 8 moon phases change one per night. A fuller moon means more slimes in swamps and better-equipped zombies and skeletons. Phantoms spawn if you haven't slept for 3 days ([ProGameGuides](https://progameguides.com/minecraft/full-minecraft-moon-phases-guide/)). Most players never notice the moon effects, so they don't count as telegraphing. The useful lesson is that **a modifier the player can't read doesn't feel like a mechanic.**
- **Numbers.** 350M copies ([Guinness/Mojang, Apr 2025](https://twistedvoxel.com/minecraft-surpasses-350-million-copies-sold-worldwide/)). 425M+ ([VGChartz quoting Xbox's Asha Sharma, 26 Sep 2026](https://www.vgchartz.com/article/469182/minecraft-sales-top-425-million-units/)).

### Zelda: Breath of the Wild / Tears of the Kingdom: Blood Moon
- **Cycle.** At midnight on a Blood Moon night, the sky goes red, a short cutscene plays (with Zelda's voice-over the first time), and **every defeated monster and picked-up weapon comes back** ([Zelda Dungeon wiki](https://www.zeldadungeon.net/wiki/Blood_Moon)). In TotK the sky starts turning red *before* midnight, and red gloom drifts up out of the ground ([GGRecon](https://www.ggrecon.com/guides/zelda-tears-of-the-kingdom-blood-moon/)). The game also uses unscheduled "panic" blood moons to clear internal state when memory runs low ([ZeldaMods](https://zeldamods.org/wiki/Blood_moon)).
- **Lesson.** **The sky warns you, and the ground answers.** Particles rising from the soil before monsters appear is your exact image.
- **Numbers.** BotW 34.32M, TotK 21.55M as of March 2025 ([Nintendo figures via search; Nintendo Life](https://nintendolife.com/news/2024/08/random-zelda-breath-of-the-wild-sold-better-than-tears-of-the-kingdom-last-quarter)).

### Majora's Mask: the moon as a clock
- **Cycle.** The game loops over three days. The moon has a face, gets bigger every day, and hits Clock Town at 6:00 on the third night. Title cards ("Dawn of the First Day") open each day. At noon on the final day, the clock UI turns into a **6-hour countdown**. Earthquakes become more frequent. The tower bell rings every 10 minutes, then every 5 minutes from 5:00, then every 3 minutes from 5:30 ([Zelda Wiki, Final Day](https://zeldawiki.wiki/wiki/Final_Day)).
- **Design note.** The team first planned a 7-day cycle. They cut it to 3 because a week of NPC schedules was too much for players to track ([Aonuma via Zelda Dungeon](https://zeldadungeon.net/?p=11216)). **Keep your cycles short enough to hold in your head.**
- **Numbers.** 3.36M on N64. Majora's Mask 3D reached 2.03M by May 2015 ([Zelda Dungeon](https://www.zeldadungeon.net/?p=11819)).

### Bloodborne (FromSoftware)
- **Cycle.** Time moves forward with the story (evening, then night, then blood moon). It isn't a loop. **Insight**, a stat you build up by seeing horrors, changes what you can see: hidden enemies, new attacks, and eventually a red moon low in the sky ([TechRaptor](https://techraptor.net/content/bullet-points-bloodborne-insight)).
- **Lesson.** The moon can track *how far things have gone* as well as *when*. Its colour and size can show how bad the world has become.
- **Numbers.** 7.46M sold through by fiscal year 2020 (leaked Sony data, [GamingBolt](https://gamingbolt.com/bloodborne-sold-nearly-7-5-million-copies-as-of-fiscal-year-2020)).

### 7 Days to Die (The Fun Pimps): the best-known horde timer
- **Cycle.** A Blood Moon Horde arrives every 7th day, from 22:00 to 04:00. Players can change the frequency and add random extra days ([wiki](https://7daystodie.wiki.gg/wiki/Blood_Moon_Horde)).
- **Telegraph ladder.** At 08:00 **the day number in the HUD turns red**. At 18:00 thunder starts and the sky begins to redden. At 21:00 the sky is deep red with a full storm. At 22:00 a scream from no particular direction starts the horde. Waves grow with your "game stage".
- **What players loved.** The whole game is "build up for 6 days, then survive the 7th". The game is literally named after its cycle.
- **Numbers.** Steam concurrent peak of 125,419 at its 1.0 release ([PCGamesN, 29 Jul 2024](https://www.pcgamesn.com/7-days-to-die/steam-success)). 16M+ copies around April 2024, 20M+ in later reports (est., [80.lv](https://80.lv/articles/7-days-to-die-is-finally-out-of-early-access-after-11-years/)).

### Risk of Rain 1 & 2 (Hopoo): time is the enemy
- **Cycle.** A difficulty bar in the top right climbs steadily: Easy → Medium → … → "HAHAHAHA". It's driven by `coeff = (playerFactor + minutes × timeFactor) × 1.15^stages` ([RoR2 wiki](https://riskofrain2.wiki.gg/wiki/Difficulty)). In single-player Rainstorm it rises about 0.1 per minute.
- **Lesson.** **The clock is the boss.** One visible bar turns every minute spent into a choice.
- **Numbers.** 1M in its first month of Early Access (Apr 2019). 2M+ by its 1.0 release, 11 Aug 2020 ([BusinessWire](https://www.businesswire.com/news/home/20200811005309/en/Open-Floodgates---Risk-Rain-2-Officially)). Up to 9.6M (est., VGI via [SteamPulse](https://www.steampulse.org/game/632360)).

### Frostpunk (11 bit studios): the forecast as the main tension
- **Cycle.** A temperature forecast for the **next five days** sits next to the thermometer at the top of the screen. A freezing-thermometer icon warns you of a cold snap ([indienova critical play](https://lab.indienova.com/indie-game-review/critical-play-report-frostpunk/)). Early drops follow a fixed script so new players learn the pattern; later they are random. The big storm hits around day 15.
- **Lesson.** **A forecast turns the weather into your research and build plan.** You see −50 °C coming in 4 days, so you rush heaters. That's the loop.
- **Numbers.** 250k in 66 hours (Apr 2018). 5M+ in six years. 400k+ bought in the 2024 Steam summer sale ([Wikipedia](https://en.wikipedia.org/wiki/Frostpunk); [Mezha, 12 Jul 2024](https://mezha.ua/2024/07/12/frostpunk-sold-400k-copies/amp/)).

### Dome Keeper (Bippinbits / Raw Fury): the wave-timer loop
- **Cycle.** Dig underground for resources, return to the dome before the wave timer runs out, defend, repeat. It started as a 72-hour Ludum Dare jam game. The wave timer is an early upgrade you can buy ([GamingOnLinux, Oct 2022](https://gamingonlinux.com/2022/10/dome-keeper-is-a-nice-twist-on-base-tower-defense)).
- **Lesson.** **"How far can I dig before I must run home?"** is your town-seed tension: going out to gather against a timer.
- **Numbers.** $1M+ gross on day one (Sep 2022) ([WN Hub](https://wnhub.io/news/analytics/item-393)). 1M players by 27 Sep 2024 ([GamesMarket](https://www.gamesmarket.global/saxonian-success-dome-keeper-by-bippinbits-reaches-one-million-units-ce406e3b6b945e16f91de158b285dcf8)).

### Dredge (Black Salt Games): day safety, night fog
- **Cycle.** Time only moves when you act. At night, fog rolls in, a panic meter rises, and strange events start. The art director said: "During the day, you're seeing lots of open vistas, and then at night, everything becomes incredibly claustrophobic once the fog comes in." The team **centred the fog on the boat, not the camera**, and **held back monster reveals** because "once you actually start seeing the monsters, they almost lose their impact" ([Game Developer, GCAP talk, 4 Oct 2023](https://www.gamedeveloper.com/production/leveraging-the-unseen-to-turn-players-worst-fears-against-them-in-dredge)).
- **Numbers.** 100k in its first 24 hours. 1M+ by Oct 2023 ([Wikipedia](https://en.wikipedia.org/wiki/Dredge_(video_game))).

### Kingdom series (Noio / Raw Fury): blood moons on a 2D side-scroller
- **Cycle.** Greed attack your walls every night. Blood moons bring bigger waves, and they don't end until every greed is dead. Destroying a portal usually brings a blood moon from that side. Blood moons come with warning sounds. In the Dead Lands DLC, the **moon phase** hints at when the next one is due ([Steam discussions](https://steamcommunity.com/app/701160/discussions/0/1743355067112584456)).
- **Lesson.** The closest match to your premise: **a tiny kingdom defended against nightly waves, with the moon as the scheduler.**
- **Numbers.** 4M+ for the series by May 2019 ([Kingdom blog](https://www.kingdomthegame.com/news/2019/5/27/raw-fury-acquires-the-kingdom-series)). Two Crowns about 759k on Steam (est., [Raijin](https://raijin.gg/app/701160/Kingdom_Two_Crowns)).

### Noita and Spelunky: weather as a rare surprise
- **Noita.** Rain is rolled once at the start of a run (1 in 15). **Snow uses your real computer clock**: in Dec–Feb there's a 1-in-12 chance ([Noita wiki](https://noita.wiki.gg/wiki/Weather)). It's mostly flavour, though rain interacts with fire and the physics.
- **Spelunky 2.** "Dark level" feelings (you need flares), and the ghost appears after 3 minutes on a level ([Spelunky wiki](https://spelunky.fandom.com/wiki/Ghost_(HD))). The ghost is a **timer you can feel instead of read**: it shows up when you dawdle.
- **Lesson.** One run-wide or level-wide condition, announced *on entry* ("I can't see a thing!"), is cheap to make and makes each run memorable.

### Outer Wilds (Mobius Digital): the 22-minute loop with planets on their own clocks
- **Cycle.** The sun goes supernova every 22 minutes. **Each planet has its own timeline inside the loop**: sand pours from one Hourglass Twin to the other, and Brittle Hollow falls apart into its black hole. Some places can only be reached at certain minutes ([Wikipedia](https://en.wikipedia.org/wiki/Outer_Wilds)). The music grows toward the end of the loop, and the sun visibly swells.
- **Lesson.** **Several clocks inside one master clock** is the deepest version of "two cycles". Players learn *when* to be *where*.
- **Numbers.** 2M+ by August 2021 (est.). Won the 2020 BAFTA for Best Game.

### Other games with forecasts and event calendars
- **Stardew Valley.** The TV shows tomorrow's weather. Rain means you don't need to water crops, and a Rain Totem changes the forecast ([wiki](https://stardewvalleywiki.com/Television)). Sales: 41M+ (Dec 2024, [PSU](https://www.psu.com/news/stardew-valley-has-surpassed-41-million-copies-sold-across-all-platforms-since-launch/)).
- **Final Fantasy XIV.** Weather follows a fixed formula, and NPC Skywatchers forecast 8/16/24 hours ahead. Players built apps (Eorzean Watch, EorzeaEnv) to set alarms for rare-weather hunts ([Console Games Wiki](https://consolegameswiki.com/wiki/Skywatcher); [PyPI EorzeaEnv](https://pypi.org/project/EorzeaEnv/2.0.0)). This is the strongest evidence that **predictable sky windows create community planning**.
- **Persona 3.** Each full moon brings a boss. The calendar counts down the days, and you split those days between social links, stats and dungeon grinding ([Wikipedia](https://en.wikipedia.org/wiki/Persona_3)). **The moon is the deadline that gives every other choice its weight.**
- **Monster Hunter Wilds.** Each region cycles Fallow → Inclemency → Plenty. Apex monsters show up during Inclemency, and you can choose the weather at camp ([Sportskeeda](https://www.sportskeeda.com/esports/how-weather-work-monster-hunter-wilds)).
- **Oxygen Not Included.** Meteor showers recur (about every 14 cycles in the base game). **Space Scanners give early warning, and more sky coverage gives earlier warning.** A telescope is needed to tell what kind of shower is coming ([ONI wiki](https://oxygennotincluded.wiki.gg/wiki/Space_Scanner)). Forecast quality is something you build. That is directly usable for you.
- **RimWorld.** Eclipse (0.75–1.25 days, dims outdoor light), solar flare, cold snap, toxic fallout, volcanic winter. **They hurt most when stacked** ([RimWorld wiki](https://rimworldwiki.com/wiki/Events)). Sales: 4.7M on Steam (est., July 2026, [PCGamesN](https://www.pcgamesn.com/rimworld/rimworld-sales-numbers)).
- **Into the Breach.** Before each turn, the board marks the tiles where Vek will **burst up out of the ground**, and you can block them by standing on those tiles ([prototypr](https://blog.prototypr.io/into-the-breachs-ux-makes-you-feel-smart-a9cb03210757)). This is the cleanest UI precedent for "monsters rising from the ground".

---

## 3. Why forecasting creates tension and planning

1. **Knowing the threat beats a surprise for strategy.** Frostpunk's 5-day forecast, 7DtD's red day number and Persona's full-moon count all show the threat early. The game becomes "how do I spend the time I have left?" instead of "react fast". Tension comes from *anticipation*, and a countdown makes every tick count ([Gnome Stew, countdown mechanic](https://gnomestew.com/tick-tock-the-countdown-mechanic/); [CGM, anticipation](https://www.cgmagonline.com/articles/how-games-to-build-player-tension/)).
2. **Uncertainty on purpose keeps the dread alive.** Icarus cut the forecast down to strength only, with some error, and hid the storm type. ONI lets you see *that* a shower is coming before you can see *which* kind. A good recipe: **certain about when, fuzzy about what, or the other way round.**
3. **Warnings in steps.** 7DtD goes HUD colour, then sky, then storm, then a scream. Majora goes moon size, then countdown, then faster bells. Don't Starve's growls get louder over 120 seconds. Each step is a new chance to change plans.
4. **Forecast quality as something you upgrade.** ONI's scanner coverage, Dome Keeper's wave-timer upgrade and Frostpunk's research all make *seeing further* a reward.
5. **Short cycles.** Majora dropped from 7 days to 3. Outer Wilds is 22 minutes. 7DtD is 7 days, and the game is named after it. Players have to be able to remember the rhythm.
6. **A safe phase sets up the threat.** Dredge's calm days are what make the nights frightening.

## 4. Two cycles overlapping: beats, tides and examples

- **The physics.** Two cycles with periods A and B line up every `lcm(A, B)`. Their "beat" (how often they go in and out of step) is `1 / |1/A − 1/B|`. **Tides are the model**: when the sun and moon line up, at new or full moon, their pulls add up into **spring tides**. At right angles they partly cancel into **neap tides**. That alternation repeats every **~14.8 days**, half of the 29.5-day lunar month ([NOAA](https://oceanservice.noaa.gov/facts/springtide.html); [LibreTexts](https://geo.libretexts.org/Bookshelves/Oceanography/Our_World_Ocean%3A_Understanding_the_Most_Important_Ecosystem_on_Earth_Essentials_Edition_(Chamberlin_Shaw_and_Rich)/03%3A_Voyage_III_Ocean_Physics/14%3A_Ocean_Tides_and_Sea_Level_Rise/14.06%3A_Spring_and_Neap_Tides)). Your game can show this directly: a **"spring" overlap** (both peaks together) is the big event, and a **"neap"** stretch is a quiet window to expand.
- **Elder Scrolls (Masser and Secunda).** In Skyrim and Oblivion the two moons line up every 5 nights, and the full pattern repeats every 120 days ([UESP / ESO forums](https://forums.elderscrollsonline.com/en/discussion/comment/1156000/)). In the lore, Khajiit have 16 forms depending on the moons. But it's **mostly cosmetic**. Two moons is an old fantasy image that has hardly been used as a mechanic, which makes it an **open space for you**.
- **Terraria** layers several cycles: an 8-night moon phase, random Blood Moon and Eclipse rolls, and summoned moons. They mostly don't interact, apart from "no Blood Moon on a new moon". That one rule is a small taste of what two cycles can do.
- **Outer Wilds** nests planet clocks inside the master 22-minute clock, so where you are matters as much as when.
- **Valheim** rolls weather once for the whole world and lets each biome read it differently. That's a cheap way to make one global "beat" mean different things in different places. It works with your "moon colour decides monster type" idea.
- **Design maths, worked example (illustration, not sourced).** Sun cycle 4 days, moon cycle 7 days → full overlap every 28 days, with partial overlaps in between. Two moons on 5 and 8 days → they line up every 40 days. Pick prime-ish periods so the near-overlaps keep changing and the forecast is always worth reading.

## 5. How stylised games show moons, eclipses and weather

- **Recolour the light source.** Terraria's red Blood Moon, black-ring eclipse and jack-o'-lantern moon. A Zelda Blood Moon tints the whole screen red. **One colour change to the moon plus a full-screen tint is the cheapest, clearest signal there is.**
- **Make the world agree with the sky.** Terraria turns all water red, rain included. TotK sends red gloom up from the ground. 7DtD adds thunder and a red sky. The sky change should show up in at least one ground-level system: water, fog, particles or grass.
- **Size and face.** Majora's moon grows and has a face. Bloodborne's moon hangs lower and bigger as Insight rises. Size can stand for *how close*.
- **Eclipse corona.** Stylised eclipses usually have a thin bright rim plus a wider soft glow behind a flat disc (e.g. the free Unity "Oborozuki" moon shader's two-layer halo, [Booth](https://booth.pm/en/items/8320267)). Godot has free stylised skies where light 1 is the sun and light 2 is the moon, and the moon phase follows the sun angle ([godotshaders](https://godotshaders.com/shader/stylized-sky-with-procedural-sun-and-moon/)). Two directional lights is how you'd build two bodies.
- **Fog centred on the player** (Dredge) keeps sight lines short without hiding the character.

---

## 6. Eight design principles for a dual-sky weather system

1. **Every overlap is read the same way: sky → ground → monster.** First the sky changes colour, then something at ground level answers (water, fog, embers, cracks), then monsters rise. The player learns this grammar once and can read any combination.
2. **Moon colour = enemy family. Sun state = weather.** Keep the two roles separate so players can read the mix. Terraria and Don't Starve show colour-coded enemy sets work.
3. **Short, prime-ish periods.** Each cycle should be a few in-game days, with no common factors, so overlaps move around and stay memorable (Majora 3 days, 7DtD 7).
4. **Forecast "when" clearly and "what" fuzzily, and let players buy better forecasts.** Exact timing, but strength and type firm up as the event gets closer. Watchtowers and scrying stones in the town sharpen the forecast (Icarus, ONI).
5. **Warnings in steps, each one a chance to act.** Forecast icon days ahead → sky tint at dusk → ground signs a minute before → a sound cue at the start (7DtD, Don't Starve).
6. **Spring and neap: big overlaps paid for with calm stretches.** After every major overlap comes a quiet window for expanding the town and gathering. That's the Dredge contrast and the Dome Keeper dig-then-defend loop.
7. **Overlaps give something back as well as taking.** Rare drops, fire that thaws frozen ground, eclipse-only resources (Terraria's event loot, Blood Moon fishing), so players *look forward* to the scary nights.
8. **One unforgettable picture per combination.** Fire falling into snow should be a single readable screenshot. If an overlap can't be captioned in four words, cut it (Minecraft's invisible moon modifiers are the warning).

## 7. Ten combinations

Bodies: **Sun** (Bright / Dim / Eclipsed). **Moon A, "Ember"** (cycles through white → amber → red). **Moon B, "Tide"** (cycles through blue → green → violet). Snow, rain and so on come from the season or base weather layer.

| # | Overlap | Weather | Visual | Enemies that rise |
|---|---------|---------|--------|-------------------|
| 1 | **Red Ember at full + winter snow** | **Firefall**: burning motes fall into the snow; drifts hiss and melt into slush | Orange sky over blue-white ground, steam columns | **Cinderlings** claw out of the melt holes; they can't cross standing water |
| 2 | **Sun eclipsed by Ember** | **Black noon**: daylight drops to dusk, temperature falls fast | Black disc with an orange corona, long shadows, stars at noon | **Shade-moles** come up wherever shadows overlap; lanterns block their tunnels |
| 3 | **Ember and Tide line up (spring alignment)** | **King tide**: rivers flood the low parts of town for one night | Both moons stacked, ring halo, water glowing purple | **Drowned wardens** wade out of the flood line; seed-shrine on high ground is safe |
| 4 | **Ember and Tide opposite (neap)** | **Still air**: no wind, perfect visibility | Pale sky, both moons small at opposite horizons | None. A **calm window** with bonus harvest; the forecast shows it in gold |
| 5 | **Green Tide + summer sun** | **Bloom storm**: warm rain, plants grow 10× faster | Green-tinted rain, flowers opening in time-lapse | **Thorn-sprouts** burst out of crops; good for harvest, bad for walls |
| 6 | **Violet Tide + Dim sun** | **Mirror fog**: thick fog that shows a ghost copy of the town | Violet fog, reflected buildings slightly out of place | **Doppels**: shadow copies of your own defenders rise from their reflections |
| 7 | **White Ember + Blue Tide + snow** | **Glass frost**: everything freezes, rivers become bridges | Twin silver moons, ice-crystal sparkle, frozen waterfalls | **Rime crawlers** cross the frozen river from the far bank; normal paths change |
| 8 | **Amber Ember + Bright sun (day overlap)** | **Heat shimmer**: mirages, crops dry out, wildfire risk | Wavering air, a double sun in the haze | **Dust-hounds** spawn from sun-baked earth; firebreaks matter (an Icarus-style fire threat) |
| 9 | **Red Ember + Violet Tide + Eclipse (triple, rare)** | **Starfall**: meteors with real impact craters, all weather layers at once | Sky split red and violet, black sun, falling stars with trails | **The Hollow King** (boss) climbs out of the biggest crater; forecast weeks ahead like Persona's full moon |
| 10 | **Tide at new moon (invisible) + rain** | **Blind rain**: the forecast goes fuzzy and the town's scrying stones crackle | Heavy rain, no moon, only lightning shows the field | **Unknown family** (rolled on the night) rises; tests how ready you are without exact information |

## 8. Forecast UI sketch, in words

- **The sky ribbon (top of the screen, always visible).** A horizontal strip about 7 days long that scrolls left as time passes, like Icarus's timeline and Frostpunk's 5-day bar. Day and night bands alternate in light and dark. **Two thin tracks** run along it: the upper one shows Moon Ember's phase as a row of small coloured discs, the lower one shows Moon Tide's. The sun's state (bright, dim, eclipsed) sits as a soft glow on the band behind them.
- **Overlap markers.** Wherever the tracks line up, a **diamond pin** appears between them. Its colour mixes the two moon colours and its icon shows the weather (snowflake + flame for Firefall). Pin size shows strength: small for a minor overlap, large for a spring alignment, crowned for the rare triple.
- **Forecast certainty.** Pins further than about 3 days out are **blurred**, showing only "something red, medium". They sharpen as they get closer. Each watchtower you build pushes the sharp zone one more day out (Icarus, ONI).
- **Hover or tap a pin** to open a small card: the name ("Firefall"), the time window, the expected enemy family as a **silhouette in the moon's colour** (left as a silhouette until you've met it, as Dredge holds back its monsters), the weather effect in one line, and what you get out of it ("cinders melt frozen ore").
- **Now-cursor.** A thin vertical line marks the present. As a pin reaches it, the **HUD day number turns that pin's colour** (7DtD's red day number) and the pin starts to pulse.
- **Calm windows** (neaps) show as **gold stretches** on the ribbon, so the player sees when it's safe to go out.
- **Full calendar view** (key press): a grid of a whole cycle (e.g. 28 or 40 days) with every overlap marked, Persona-style, for long-range planning and so players can share "Starfall on day 37" with each other.

## 9. Five trailer and short-clip hooks

1. **"It's snowing fire."** A quiet snowy town at dusk → the moon turns red → the first ember falls and hisses into a drift → claws break through the melt hole. Cut to the forecast pin: snowflake + flame. 6 seconds; works without sound.
2. **The forecast scrub.** Close-up on the sky ribbon as two moon tracks slide toward each other. The pin fills in, the camera pulls up to the real sky and the moons stack into a ring. "Day 37." Smash cut to the King Tide flood.
3. **Same town, three skies.** One locked-off shot of the town seed, cross-fading through three overlaps: Glass Frost, Bloom Storm, Mirror Fog. Each has its own monsters rising from the same ground. Caption: "The sky decides who comes."
4. **The eclipse at noon.** First-person at midday. The sun gets bitten away, stars come out, shadows stretch. In every patch of shadow the ground starts to move. Ends on black, then the orange corona.
5. **Spring vs neap.** Split screen: on the left, a calm gold night where the player builds; on the right, the same town 14 days later at full alignment. Text: "Plan for the tide."

---

## Sources (main)
Icarus: [eip.gg Week 60](https://eip.gg/icarus/news/icarus-week-60-update-overhauled-weather-forecasting-system-and-30-minute-backups/), [TheGamer](https://www.thegamer.com/icarus-tips-tricks-surviving-storms/), [GamePretty](https://www.gamepretty.com/icarus-weather-guide-all-you-should-know/), [GameGeeker](https://gamegeeker.com/games/icarus), [PCGamesInsider](https://www.pcgamesinsider.biz/news/72704/charts-dayz-creators-icarus-debuts-at-no1-on-steam) · Terraria: [wiki Events](https://terraria.wiki.gg/wiki/Event), [Pumpkin Moon](https://terraria.wiki.gg/wiki/Pumpkin_Moon), [Inven Global](https://www.invenglobal.com/articles/21948/15-year-old-terraria-surpasses-70-million-cumulative-sales) · 7DtD: [wiki](https://7daystodie.wiki.gg/wiki/Blood_Moon_Horde), [PCGamesN](https://www.pcgamesn.com/7-days-to-die/steam-success) · Don't Starve: [hounds](https://dontstarve.wiki.gg/wiki/Periodic_hound_attacks) · Zelda: [Zelda Dungeon](https://www.zeldadungeon.net/wiki/Blood_Moon), [Final Day](https://zeldawiki.wiki/wiki/Final_Day) · Dredge: [Game Developer](https://www.gamedeveloper.com/production/leveraging-the-unseen-to-turn-players-worst-fears-against-them-in-dredge) · RoR2: [wiki](https://riskofrain2.wiki.gg/wiki/Difficulty) · ONI: [Space Scanner](https://oxygennotincluded.wiki.gg/wiki/Space_Scanner) · RimWorld: [events](https://rimworldwiki.com/wiki/Events) · Tides: [NOAA](https://oceanservice.noaa.gov/facts/springtide.html). Other links are inline.

## Gaps and caveats
- **Icarus Week 60 date**: the patch-note pages (eip.gg, SteamDB) blocked fetching. "Early 2023" is my estimate from the weekly update count after the Dec 2021 launch. The 984k Winter Sale figure comes from a secondary summary of Hall's post, not the original post.
- **Steam estimates** (Raijin, VGI, SteamPulse) are model-based, not reported sales. They're marked (est.).
- **Kingdom's blood moon warnings** came only from Steam forum threads. I found no wiki page.
- **Bloodborne's moon** is tied to story progress and Insight. I didn't find a primary source on the exact moon visuals per Insight level.
- **No official sales numbers exist** for Outer Wilds, RimWorld or 7DtD after 1.0. The figures above are estimates or old milestones.
- **"What players loved"** comes from reviews and design talks, not from player surveys.
- **Dome Keeper's 1M** is "players", which may include Game Pass and other subscription players, not only copies sold.
