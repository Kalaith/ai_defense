# Last Assembly — Design Review

*Senior design / systems review. Reviewed: source (`src/`), content JSON (`assets/`), design doc (`game_design.md`), owner feedback (`feedback.md`), and a freshly captured gameplay screenshot. Date: 2026-07-07.*

> **Headline:** This is a real, working game with one genuinely original idea (the beacon-as-bait sacrifice loop) buried inside a competent-but-conventional tower defense. The recent work has fixed the *legibility* crisis the owner flagged. The next crisis is **motivation**: the unique fantasy is mostly flavor text, and the one novel decision the game owns is currently too low-stakes to carry it. Continue development, but redesign the meta-loop so the sacrifice actually costs something.

> **Addendum (2026-07-07), on owner insight — survival infrastructure:** The original review framed the factory as a rack of *combat* upgrades and treated the population/food system as a minor idle subsystem. That under-weights the sharpest expression of the game's core pillar: **enabling factory subsystems should be a survival necessity that costs beacon volume and adds no combat strength.** The owner's example — light up Water Filtration and the scream grows, but skip it and your people die of thirst — is the "grow louder to keep your *own* people breathing" tension, and it is *stronger* theme-per-mechanic than the combat-buff framing the rest of this review leans on. Crucially, the skeleton already exists in the build (see the new **Survival Infrastructure** system section below), and one piece of it — **water** — is currently a dead stake: it is produced and it costs beacon volume, but nothing ever consumes it. This addendum threads that concept through the sections that follow.

---

# 1. Project Overview

## Project Name
**Last Assembly** (repo: `ai_defense`)

## Genre
Tower Defense + light base/resource management, with a survival/attrition framing. Single-player, top-down 2D (Rust + macroquad).

## Core Concept

**What the player does:** You reclaim a dead megafactory one section at a time — repair machines, power them, place turrets along fixed circuit-board routes — then "raise the beacon" to draw machine waves. While the beacon burns, scavenger teams are out in the field returning scrap, food, and survivors. You decide how long to hold before shutting the beacon down and recalling them.

**The fantasy on paper:** *"You are not building a fortress to survive. You are building a scream so others don't have to."* The factory is a deliberate decoy — you make yourself impossible to ignore so humanity can breathe elsewhere. Every upgrade makes you stronger **and** louder, so growth is the source of danger.

**What actually makes it different (as built):** One decision — *how long do I keep the beacon up before I lose the teams I sent out?* Everything else (turrets, routes, upgrades, power budget) is well-executed but familiar. That single risk/greed dial is the intellectual property.

**The second, under-realized axis (see Addendum):** *how much of the factory do I dare bring online just to keep my own people alive?* Every life-support system you enable (water, food) makes the beacon louder and gives no combat benefit — but neglect it and the holdout dies. This "survival-vs-volume" decision is a distinct, equally-ownable hook, currently only ~20% wired.

**Target player:** TD players who like a management/economy layer (Mindustry, They Are Billions, Rusted Warfare crowd). Not the pure-optimizer Bloons audience; someone who enjoys tension and theme.

## Current State

**Feature-complete-but-thin, transitioning from "make it legible" to "make it matter."** This is *not* a prototype. The implemented systems are substantial: full wave/combat sim, five tower types, six factory sectors, a 13-node upgrade tree, threat/awareness scaling, beacon phases, scavenger recall + salvage reports, a repeatable beacon cycle, progressive map reveal, a 4-step onboarding coach, save/load, and a headless screenshot harness. The May→July commits (`readable map`, `declutter HUD`, `progressive disclosure`, orphan-slot fix) directly resolved the owner's "UI is a mess" complaint — the current screenshot is clean and guided.

What's *missing* is not plumbing, it's **content depth and stakes**: one map, one enemy roster of five, no persistent meta-progression, no audio, and a narrative that lives entirely in the readme + one intro card. The engine is ready for content; the design needs to prove the loop is fun before that content is worth building.

---

# 2. Core Gameplay Analysis

## Main Gameplay Loop

As built, per beacon cycle:

> **Repair a machine → Power it (reveals more map) → Build/upgrade towers → Raise beacon (waves + scavengers out) → Survive & bank loot → Shut down & recall → Read salvage report → Rebuild louder → Repeat**

## Evaluation

- **Is it clear?** *Now, yes.* The coach walks the four opening beats, the map legend explains every glyph, and the "NEXT STEP" strip always names one action. This was the project's biggest liability three commits ago and it's largely solved.
- **Is it satisfying?** Partially. The moment-to-moment TD (place, watch, panic at a breach) works — the owner confirms "placing towers, seeing how many you can take" is the best part. The satisfying beat is real.
- **Meaningful decisions?** *One* strong one (beacon duration vs. scavenger safety) and several weak ones (which turret, which upgrade). The strong one is undercut — see §3 Beacon.
- **Enough variety?** No. One map, five enemies, five towers, linear upgrade paths. A second cycle plays almost identically to the first.
- **Long-term motivation?** *This is the gap.* Because the beacon cycle is repeatable and scavenger loot just feeds the same run's economy, there is no persistent reason to push further than "comfortable." The game gives you a farming loop where it promised a sacrifice.

**The core tension the theme sells — "grow louder to survive longer, risk dying faster" — is only half-wired.** Beacon strength does scale wave size (`update_beacon` → `phase_from_strength` → wave budget/tier floor), so growth *does* raise danger. But growth is optional and reversible, and nothing forces the player up the risk curve. The escalation is a slider the player rarely has a reason to push.

---

# 3. Existing Systems Review

## Beacon / Scavenger System

### Purpose
The signature system. Converts "time survived under pressure" into external reward, and forces the greed-vs-safety decision that is the game's whole identity.

### Current Implementation
Beacon strength = `sectors×2 + power/10 + aiVault×5 + pop/20`, driving four phases (Warm → Sustained → Screaming → Terminal Howl). Each phase raises wave budget/tier and improves scavenger yields (`update_scavenger_recall`). Teams go out on beacon start; recall/shutdown resolves them with a per-phase loss chance; a salvage report tallies the cycle; then you rebuild and raise again (`finish_beacon_cycle` / `dismiss_salvage_report`).

### Strengths
- The **recall-before-shutdown timer** is the one genuinely novel, tense decision in the game. The owner independently named it as the thing with potential.
- Cleanly data-driven (yields/loss chances in `constants.json`), easy to tune.
- Phase escalation gives the run a natural arc and dramatic flavor lines.

### Weaknesses
- **The stakes are local, so the tension is fake.** Scavenger loot (scrap/food/pop) is spent inside the *same* run. There's no persistent settlement, campaign map, or meta-currency that the sacrifice feeds — despite the design doc's entire premise resting on "humanity does better *elsewhere*." Losing a team costs you a little economy this run and nothing after it. The emotional payload ("you are bait so others live") has zero mechanical representation.
- **The repeatable cycle contradicts the fantasy.** The design doc says "collapse is expected... there is no clean choice." The build turns it into a safe, farmable rebuild loop. You can raise a Warm-Signal beacon, bank easy loot, shut down before anything threatens you, and repeat forever. Risk is opt-in and trivially avoided.
- **Greed isn't rewarded steeply enough to tempt.** Terminal Howl yields ~5× Warm scrap but with a 20% per-tick loss chance — a fine ratio, but since the loot has no lasting use, there's little reason to reach for it.

### Improvement Ideas
- **Give the sacrifice an external ledger.** A persistent "survivors evacuated / colony strength" meter that ONLY grows from banked scavenger returns and beacon time, carries across cycles/missions, and is the actual win condition. Now every extra second of Screaming Beacon means something.
- **Make shutdown a real cost, not a reset.** Either escalate difficulty permanently each cycle (machine awareness rises and *stays* risen — the doc's "the AI learns you are the bait"), or make sectors take lasting damage, so you can't farm the same easy beacon indefinitely.
- **Add a ratchet that forces the risk curve up** (e.g. food drain that outpaces low-phase yields, so you *must* push to Screaming to feed the holdout). Turn the escalation slider into a treadmill.

**Impact: Game-changing. Cost: Medium** (the systems exist; this is re-pointing reward flow + a persistence layer, not new tech).

---

## Tower Defense / Combat

### Purpose
The minute-to-minute play and the reason the "best part" feedback exists.

### Current Implementation
Five tower archetypes (Ballistic/Laser/EMP/AreaDenial/Subversion) with per-type combat quirks (EMP slow, flak AoE, subversion chain-kills, laser vs. armor multipliers) in `tick_towers`. Fixed slot placement on a circuit-board map with two-step unlock (clear debris → power). Power budget auto-disables towers on brownout. Five enemy types incl. commanders with a speed aura.

### Strengths
- Type matchups are real, not cosmetic (`laser_vs_heavy_mult`, `ballistic_vs_heavy_mult`, commander death-splash). There's genuine counter-play.
- The **power-budget constraint** is a nice twist — towers fight for a limited grid, so "build more" isn't free. This is underexploited but good.
- Fixed slots + multiple unlockable enemy routes is a smarter framing than free-placement mazing.

### Weaknesses
- Only one map means the spatial puzzle is solved once and never again.
- Tower upgrades are flat stat bumps (`apply_upgrade_levels`); no branching or identity change at higher levels.
- Enemy roster is small and behaviorally shallow — scouts dodge, saboteurs skip, commanders buff, the rest walk. The design doc's "enemies adapt to what you build" is not implemented (threat only scales *quantity/health*, not tactics).

### Improvement Ideas
- Lean into the power budget: make it the central spatial-economic puzzle (overclock towers for more power draw + more heat/threat).
- 2–3 more enemy behaviors that *punish specific builds* (shielded-front units that beat ballistic, fast swarms that beat single-target laser) to make tower choice situational.
- One branching upgrade per tower at max level (identity fork, not +10%).

**Impact: High. Cost: Medium.**

---

## Factory / Sector Progression + Map Reveal

### Purpose
Progression spine; the "awaken a dead god" fantasy; the gate that controls information and difficulty pacing.

### Current Implementation
Six sectors with power costs and passive bonuses (assembly=+dmg, robotics=+fire rate, ai_vault=+range, logistics=+scrap/food). Repair→power→awaken tiers. Map sections stay hidden until you power the frontier (`update_section_visibility`, `reveal_all_sections`), camera grows to match. 13-node upgrade tree gated by sector.

### Strengths
- **Progressive reveal is the single best UX decision in the project** — it converts the "overwhelming map" problem into a drip-fed sense of discovery *and* paces difficulty. Excellent.
- Sector bonuses are legible and each meaningfully changes combat math.
- Repair-then-power two-step gives progression a satisfying physical texture.

### Weaknesses
- Sector bonuses are passive global multipliers — you unlock them, then forget them. No ongoing decision.
- The "awakening tiers / factory becomes organic / UI distorts at high corruption" from the design doc is not present; awakening is currently just a label.
- Every sector is strictly good to unlock, so there's no real *choice* in expansion order beyond affordability.

### Improvement Ideas
- Tie each sector's power draw and threat contribution together so unlocking is a real trade (louder beacon, bigger target) rather than pure upside — this also directly feeds the risk-curve fix in §Beacon.
- **Split the factory into two categories: military/economy (towers, damage, scrap) vs life-support (water, food, medical).** Today every sector is a combat/economy buff, so "enable = strictly good." Life-support subsystems should give *zero* combat value — you enable them only to keep people alive, and each one raises beacon volume. That converts "which buff next" into the far better "can I afford to be this loud just to survive?" (see §Survival Infrastructure).
- Small: let sector *integrity* matter more visibly (a damaged Power Core should brown out your grid, creating a defend-the-core priority).

**Impact: Medium (High if it carries the life-support split). Cost: Small–Medium.**

---

## Threat / Machine Awareness

### Purpose
The reactive-enemy fantasy; the mechanical link between "growth" and "danger."

### Current Implementation
Six hidden signatures (energy/heat/data/corruption/noise/territory) → an awareness level that scales wave budget and enemy health, with decay over time. Breaches and buildings add noise; combat adds heat.

### Strengths
- Good bones: the feedback loop (build/fight louder → harder waves) is wired and data-driven.
- Decay gives a reason to sometimes go quiet.

### Weaknesses
- **Invisible.** The player feels waves get bigger but can't see *why* or steer it. A hidden system that drives difficulty but isn't legible is just difficulty that feels arbitrary.
- Only affects *quantity/health*, never enemy *composition or tactics* — so it reads as "numbers go up," not "the machines are reacting to me."

### Improvement Ideas
- Surface it: a simple "what's making noise" readout (the THREAT panel already exists — put the top 2 contributors in it). Let the player make informed noise/quiet decisions.
- Let awareness tiers unlock *qualitative* enemy responses (saboteurs appear once corruption is high, commanders once territory is high) so it feels like adaptation, not inflation.

**Impact: High. Cost: Small** (surfacing) **to Medium** (qualitative tiers).

---

## Population / Food / Economy

### Purpose
Survival pressure; the reason scavenger food matters.

### Current Implementation
Population with morale/health/food; starvation drains morale/health and eventually kills; productivity feeds scrap income. Food comes from waves + logistics + scavengers.

### Strengths
- Closes the loop: scavenging → food → population → productivity → scrap → defense.
- Starvation death spiral is a genuine fail state distinct from being overrun.

### Weaknesses
- In practice food is rarely a binding constraint (starting 100 + per-wave rewards keep it comfortable), so this whole subsystem currently idles. It's a survival pressure that doesn't press.
- Morale/health are tracked but their gameplay consequence (productivity) is subtle enough to ignore.

### Improvement Ideas
- **Make food the ratchet that forces beacon greed** (see §Beacon): if the holdout out-eats low-phase yields, the player *must* push to riskier phases to feed them. This retasks an idle system into the engine of the core tension — high value, near-zero cost. *(Implemented 2026-07-07: food drain now scales while the beacon is up.)*
- Food is only the first of several life-support demands — water is already produced but unused. Treat this system as one half of **Survival Infrastructure** below, not just a food counter.

**Impact: High (as the greed-forcing lever). Cost: Small.**

---

## Survival Infrastructure (Life Support) — *the under-weighted axis*

### Purpose
The factory is not only a weapon rack; it is **life support** for the human holdout. Enabling a subsystem should keep people alive *and* make the beacon louder, with **no combat value** — a survival-vs-volume decision that runs parallel to, and independent of, the defense. This is the sharpest, most on-theme expression of the core pillar ("grow louder, draw more danger"), and the review's original framing missed it.

### Current Implementation
The **skeleton is already built.** The map is divided into named sections you repair→power (`Intake & Scrap`, **`Water Filtration`**, `Power Spine`, `Assembly`, …). Every building carries a **`threat_per_sec`** (0.12–0.35 — it makes the beacon louder) *and* a `boon` (scrap/food/water/power). So the *cost* side of "enable a subsystem → louder beacon" is fully wired. The **demand** side is almost entirely missing:
- **Food** is the only realized life-support demand — the population starves without it (and only became a binding constraint after the 2026-07-07 food-ratchet change).
- **Water is a dead stake.** `water_reclaimer` buildings produce `water_per_sec` and cost `threat_per_sec`, water is tracked, iconified, and factored into building "value" — but **nothing ever consumes it.** Enabling Water Filtration today adds beacon volume for *zero* payoff and there is *no penalty* for skipping it. It is a trap with no reason to spring it and no cost to ignore.

### Strengths
- The structure is present and cheap to complete: sections, repair→power, per-building threat cost, a water resource and its production. Realizing the concept is mostly *connecting existing pieces*, not building new systems.
- Theme-per-mechanic is the highest in the game: "you scream louder to keep your *own* people breathing" is tighter than any combat buff, and it reinforces the beacon fantasy instead of competing with it.

### Weaknesses
- **The one life-support subsystem that exists as pure survival-cost (Water Filtration) does nothing**, so the intended dilemma ("louder, but my people live") is not playable today.
- Because sectors/buildings are otherwise framed as combat/economy upside, "enable = strictly good"; there is no category whose only reason to exist is survival-at-the-cost-of-volume.
- **Population is an idle number that only goes up.** There is no reason to run a *lean* holdout, so the real decision — *how many people can I afford to keep alive and hidden?* — never gets asked.

### Improvement Ideas (the concept, made mechanical)
- **Make water a real demand.** The population consumes water like food; no water → dehydration → morale/health loss → deaths. Now Water Filtration is *required* to sustain (let alone grow) the holdout, and every reclaimer you light up is more beacon volume. This is the smallest change that turns the dead stake into the exact tension the owner described.
- **Make population a lever, not a counter.** More survivors = more workforce (scrap/research) + more scavenger teams (more evacuation) **but** more mouths → more life-support buildings → louder beacon → bigger waves. This *is* the "don't bite off more than you can handle" pitch from the owner's own `feedback.md`, finally mechanical.
- **Interlock with the meta-loop already built.** The survivor-evacuated ledger (reward for a big, loud holdout) and life-support demand (cost of a big, loud holdout) become the two ends of one population dial — grow it to save more people elsewhere, but only if you can feed, water, and hide them.

**Impact: Game-changing (this is plausibly the missing spine of the whole design). Cost: Small (water demand) → Medium (full life-support/military split + population-as-lever).**

---

## UI / Onboarding / Coach

### Purpose
The difference between "someone who knows the systems" and "anyone else" being able to play — the owner's stated #1 milestone.

### Current Implementation
4-step coach that advances on real actions; "NEXT STEP" suggestion strip driven by `build_ui_advice`; alert banners; power-grid snapshot; wave-preview card; map legend; skippable intro card.

### Strengths
- The coach + suggestion strip + legend together resolve the "wouldn't know what to do" problem convincingly (confirmed by the current screenshot vs. the May one).
- The advice engine (`ui_advice.rs`) is a strong asset — it already computes "the single best next action," which is exactly the scaffolding a new player needs.

### Weaknesses
- The advice layer is code-heavy and several UI files exceed the repo's own 800-line hard limit (`ui/gameplay.rs` 1273, `ui_advice.rs` 1065, `ui/mod.rs` 908, `render_panels.rs` 861) — tech debt that will slow every future UI change. (Also: two dead files, `src/state/gameplay.rs` and `src/state/gameplay/ui.rs`, still shadow the live `src/ui/gameplay.rs`.)
- No audio at all — a major absence for a game whose design doc leans hard on "the factory voice evolves from static to personality."

### Improvement Ideas
- Split the oversized UI files before adding more (the standards require it, and it's cheap now, expensive later).
- Even placeholder audio (beacon hum that intensifies by phase, breach alarm) would add more perceived polish per hour than any visual change.

**Impact: Medium (files) / High (audio). Cost: Small–Medium.**

---

# 4. Similar Games & Lessons

## Frostpunk *(the closest analogue — especially for the survival axis)*
**Similar:** a survival game built around **one central system that everything depends on**. Frostpunk's Generator is Last Assembly's beacon: you crank it harder to keep people alive against an escalating external threat, and every notch up carries a mounting cost. The Generator's heat range ≈ the beacon's volume; "the city must not die" ≈ "the holdout must not die"; the incoming storm ≈ the beacon phases climbing to Terminal Howl. The whole game is the tension Last Assembly is *reaching* for: **growth and survival are the same lever, and pulling it is what endangers you.**

**Does better (and these are exactly Last Assembly's current gaps):**
- **Survival infrastructure *is* the game.** Every building is life-support — heat, food, medical, shelter — each a real trade-off, none a dead stake. This is precisely the axis Last Assembly is missing (its Water Filtration is currently inert). Frostpunk is the proof that life-support-as-tension can carry an entire game.
- **Population is a two-sided, breakable resource,** not a number that only rises. Hope and Discontent can tip into revolt or exile if you push people too hard — so "how large a holdout can I sustain, and at what social cost?" is a live decision every hour. This is the exact "population as a lever" fix this review recommends.
- **Escalation is telegraphed and dramatic.** The storm is coming on a known curve; the temperature drops on schedule. That inevitability creates dread and forces hard pre-commitment — more than Last Assembly's smoothly-rising (and until recently invisible) awareness does.
- **Morality is emergent from mechanics,** not a meter — the law book (child labor, forced overtime, the "inhuman solutions") makes you *feel* the compromises. This is exactly the "soft, emergent morality" the design doc wants and hasn't built (its corruption/AI-Vault path is the natural home for it).

**Adapt:**
- Make life-support the **spine**, per §Survival Infrastructure: the water/food demand, and a clean life-support-vs-military split, so enabling a subsystem is a survival-vs-volume decision.
- Give the already-tracked **morale/health teeth** (Frostpunk's Hope/Discontent) so a mistreated or over-large holdout can *break*, not just slowly starve.
- **Telegraph the escalation** — a visible "the storm/terminal howl is coming" curve turns the beacon phases into dread instead of a stat.
- Route the design doc's "increasingly inhuman solutions" through the AI Vault as **mechanics, not a morality slider**.

**Don't copy:**
- **Its UI density and management overload.** Last Assembly *just* won its legibility fight; do not rebuild it into a Frostpunk-scale spreadsheet of laws, factions, and per-citizen sims. Take the *tensions*, keep the abstraction light.
- **The society-sim breadth** (individual named citizens, faction politics, the full law tree) — wrong scale for a TD-paced game.
- **Grimness for its own sake / no-win despair** if it fights the moment-to-moment satisfaction of holding a line. Last Assembly has a **combat layer Frostpunk lacks** — that's its own differentiator, so borrow Frostpunk's survival *spine* and bolt it onto the tower defense, rather than becoming a colder Frostpunk.

## They Are Billions
**Similar:** hold a base against escalating waves with a fragile economy behind the wall; one breach can cascade.
**Does better:** the *dread* of the final wave, and an economy where every building placement is a real trade-off (population vs. defense vs. income).
**Adapt:** its "you know the horde is coming and it never stops" pressure is exactly what Last Assembly's beacon wants to be. Steal the *inevitability*, not the RTS scope.
**Don't copy:** permadeath brutality and colony-sim breadth — wrong scale for this.

## Mindustry
**Similar:** factory reclamation + tower defense + power/resource routing on a 2D grid.
**Does better:** production chains that make the "factory" feel alive and make expansion a logistics puzzle; enemy waves that scale with *your* production.
**Adapt:** the idea that building bigger literally makes the enemy stronger — Mindustry ties threat to output far more legibly than Last Assembly's hidden awareness does.
**Don't copy:** deep conveyor logistics — that's a different game; Last Assembly's abstraction is the right call.

## FTL: Faster Than Light
**Similar:** a run is a series of risk/reward gambles where greed (one more jump / one more beacon phase) is punished; loss is expected and framed as story.
**Does better:** every decision has *persistent* stakes because resources carry the whole run and death is final. This is precisely what Last Assembly's repeatable cycle lacks.
**Adapt:** FTL's "push your luck for one more reward, knowing it might end you" is the emotional model the beacon should copy. Make the sacrifice *stick*.
**Don't copy:** roguelike permadeath if it fights the "persistent factory across missions" campaign vision — but borrow the *irreversibility of a bad gamble*.

## Kingdom: Two Crowns
**Similar:** minimalist survival where you send subjects out and pull back before nightfall; a beautiful, legible economy of risk under a day/night timer.
**Adapt:** its recall-before-the-timer loop is nearly identical to the scavenger mechanic — and Kingdom makes that single decision carry an entire game through *clarity and mood*, not complexity. Proof that Last Assembly's core loop can stand on its own if the stakes are felt.

---

# 5. Feature Improvement List

## Critical Improvements
| Priority | Feature | Description | Player Benefit | Cost |
|---|---|---|---|---|
| Critical | Persistent sacrifice ledger | Scavenger returns + beacon time feed a colony/survivors meter that carries across cycles and is the real win condition | Makes the core fantasy *mechanical*, not flavor; gives every risky second meaning | Medium |
| Critical | Kill the free-farm loop | Awareness (or sector damage) escalates permanently each cycle so you can't safely repeat a Warm beacon forever | Restores the "collapse is expected" tension the whole design promises | Small–Medium |
| Critical | Food as the greed ratchet | Holdout out-eats low-phase yields, forcing players up the risk curve to feed survivors | Turns an idle system into the engine of the core decision | Small |

## High Value Improvements
| Priority | Feature | Description | Player Benefit | Cost |
|---|---|---|---|---|
| High | Surface machine awareness | Show the top noise contributors in the existing THREAT panel | Difficulty feels earned and steerable, not arbitrary | Small |
| High | Qualitative enemy adaptation | Awareness tiers change enemy *composition/tactics*, not just numbers | Delivers the "reactive AI" promise; keeps combat fresh | Medium |
| High | Placeholder audio pass | Phase-driven beacon hum, breach alarm, tower fire | Enormous perceived-polish gain per hour | Medium |
| High | 2–3 counter-build enemies | Units that punish a specific tower type | Makes tower choice situational instead of solved | Medium |

## Nice To Have
| Priority | Feature | Description | Player Benefit | Cost |
|---|---|---|---|---|
| Med | Branching max-level tower upgrades | One identity fork per tower instead of flat +% | Build variety, replay depth | Medium |
| Med | Second map / route layout | A new spatial puzzle | Replayability | Medium |
| Med | Split oversized UI files | Bring `ui/gameplay.rs` et al. under the 800-line limit; delete dead `state/gameplay.rs` | Faster, safer iteration | Small |

## Avoid / Do Not Add
| Avoid | Why |
|---|---|
| Multiplayer / co-op | The fantasy is a solitary sacrifice; multiplayer dilutes it and 10×s scope for zero thematic gain |
| Deep production chains (Factorio-style) | The factory abstraction is *correct*; conveyors would bury the beacon decision under logistics busywork |
| A morality meter / dialogue trees | The doc already wisely chose "soft, emergent morality" — an explicit meter would cheapen it |
| More resource types | Scrap/power/food already sit mostly idle; adding water/data-cores as tracked-but-inert stats (as now) adds UI noise, not decisions |
| More tower *types* before enemy depth | You have 5 towers and effectively 2 enemy behaviors; add reasons to choose, not more choices |

---

# 6. Missing Gameplay Elements

## Persistent meta-progression
- **Expected?** The design doc explicitly promises a persistent factory and campaign map across missions; players of this genre expect a reason to return.
- **Needed?** *Yes — this is the missing keystone.* Without it the sacrifice has no consequence and the loop is a sandbox.
- **Implementation:** A between-cycle colony screen showing survivors saved and one persistent upgrade/unlock purchased with banked returns.
- **Priority:** Critical.

## Audio
- **Expected?** Yes — near-universal for the genre and central to this game's stated mood.
- **Needed?** Yes.
- **Implementation:** Toolkit-level SFX/music hooks; phase-indexed ambient bed.
- **Priority:** High.

## A real win/goal state
- **Needed?** Yes. "Hold as long as possible" is a score, not a goal. Players need something to *achieve* (evacuate N survivors, complete the exodus vessel from the doc's endings).
- **Priority:** High.

## Enemy variety / adaptation
- Covered in §3/§5. **Priority:** High.

## Things the game does *not* need (resist)
- Named/individual survivors, base cosmetics, crafting trees, a skill tree for the player-character. All would add surface without touching the core decision.

---

# 7. Content & Replayability Analysis

**Current replay drivers:** score-chasing (waves survived), the built-in randomness of scavenger loss, and the mild optimization of build order. That's it — and none of them compound, because nothing persists between cycles.

**The problem:** the game is a *sandbox that resets*. A second session is mechanically identical to the first: same map, same five enemies, same upgrade tree, same comfortable equilibrium. There is no unlock arc, no rising challenge you carry forward, no long-term goal.

**Highest-leverage fixes (all already listed above):**
1. Persistent sacrifice ledger + between-cycle unlocks → gives runs a compounding arc.
2. Permanent escalation per cycle → each run is harder than the last, self-generating difficulty.
3. Food ratchet → forces different (riskier) play each cycle instead of the safe equilibrium.

Emergent gameplay potential is real (power budget × threat × beacon phase × scavenger timing is a rich interaction space) but it's currently *dominated* by the safe strategy. Remove the safe strategy and the emergence surfaces on its own — you likely don't need more systems, just fewer escape hatches from the ones you have.

---

# 8. Player Experience Review

## First 10 Minutes
**Understands (now):** thanks to the coach + legend + suggestion strip, a new player learns repair → power → build → beacon cleanly. This is a solved problem as of the recent commits — a genuine achievement given where it was in May.
**Could improve:** the *why* is still thin. The player learns the verbs but not the stakes ("why am I raising a beacon that attracts enemies?"). One line of context at the beacon button ("Raising the beacon sends your teams out — and paints a target on you") would seat the fantasy at the exact moment of the key decision.

## First Hour
**Hook strength: moderate, at risk.** The first beacon cycle — watching your line hold, sweating a breach, deciding when to recall — is a good hour. But the *second* cycle reveals the flatness: same map, no new goal, loot that doesn't accumulate. The hook lands, then loosens. Whether an hour-one player continues to hour-two depends entirely on fixing the meta-loop.

## Long-Term
**Currently: nothing holds them.** No persistence, no unlock chase, no escalating goal. This is the make-or-break gap. With the §5 Critical fixes, the long-term driver becomes "how many survivors can I evacuate before the machines learn too much" — a compelling, ownable long-term goal that flows directly from the existing systems.

---

# 9. Development Roadmap

## Phase 1: Make It Matter (the fun is in the stakes)
**Goal:** turn the beacon from a farmable sandbox into a tense gamble. **This is the whole ballgame — do it before anything else.**
- Persistent sacrifice ledger (survivors evacuated carries across cycles; it's the goal). *(done)*
- Permanent per-cycle escalation (awareness or sector damage sticks). *(done)*
- Food ratchet forcing the risk curve. *(done)*
- **Make water a real demand** — kill the dead stake. The population consumes water; no water → dehydration → deaths. This alone makes Water Filtration the owner's exact example (louder to survive) and costs almost nothing to wire.
- One line of beacon-context framing.
**Why first:** every other improvement is wasted effort if the core loop is still a safe farm. This phase is mostly re-pointing reward flows through systems that already exist — cheap, and it validates whether the game is fun *before* you invest in content.

## Phase 2: Add Depth
**Goal:** make the moment-to-moment decisions as interesting as the meta-decision.
- Surface machine awareness; make it steerable. *(done)*
- Qualitative enemy adaptation tiers + 2–3 counter-build enemies. *(adaptation done; counter-build enemies pending)*
- **Split the factory into life-support vs military/economy, and make population a managed lever** — grow the holdout for workforce + evacuation, but every mouth needs food/water buildings that raise beacon volume. This is the survival-vs-volume axis; it turns "which buff next" into "how loud can I afford to be just to keep people alive."
- Branching max-level tower upgrades.
- Tie sector unlocks to threat trade-offs.
**Why second:** depth only pays off once players have a reason to keep playing (Phase 1). Adding enemies to a loop no one repeats is premature.

## Phase 3: Add Content
**Goal:** extend the validated loop.
- Second and third maps / route layouts.
- Mission structure with the doc's escalating framings (start-under-assault, multi-beacon).
- One or two of the doc's endgame paths as concrete goals.
**Why third:** content multiplies a fun loop and multiplies a boring one equally — only worth it after Phases 1–2 prove the loop.

## Phase 4: Polish
**Goal:** match presentation to the strong theme.
- Audio pass (phase-driven ambience, alarms, fire).
- Awakening visuals (rust→glow, the doc's UI-distortion-at-high-corruption).
- Split oversized UI files; delete dead modules; formalize balance fixtures.
**Why last:** polish amplifies an experience; it can't create one. (Caveat: a *cheap* audio pass could slip into Phase 2 for morale/perceived-progress — it's the highest polish-per-hour item.)

---

# 10. Final Assessment

## Strongest Idea
**The beacon-as-bait sacrifice loop** — "you are building a scream so others don't have to," expressed through the recall-before-shutdown decision. It's original, it's thematically tight, and both the owner and this review independently identified it as the thing worth protecting. Everything else in the game is in service of this one dial.

## Biggest Risk
**That the sacrifice never becomes mechanical.** Right now the game's unique fantasy is entirely flavor text wrapped around a competent but conventional TD, and the one novel decision is neutered by a repeatable, consequence-free cycle. If development pours into more towers/enemies/maps (content) while the meta-loop stays a safe sandbox, the project becomes "a decent TD with an unusual theme" — forgettable — rather than "the game where growth is the trap." The risk is polishing the wrong layer.

## Missing Ingredient
**A felt cost to growth — on two sides.** (1) *Reward side — persistent consequence:* a survivors-evacuated meter that only the beacon feeds and that a bad recall permanently dents. *(Since implemented.)* (2) *Cost side — survival infrastructure:* keeping your own holdout alive must require lighting up subsystems that make you louder and give no combat value — starting with the trivial fix of **making water actually consumed** so Water Filtration stops being a dead stake. Together these make growth a genuine dilemma: you expand to save more people, and expanding is exactly what gets you found. The reward side is now in; the cost side is the cheapest remaining high-impact move.

## Unique Selling Point
*"A tower defense where the smart move is to make yourself a bigger target — because you're bait, and every second you scream keeps someone else alive — and you have to scream just to keep your own people breathing."* No mainstream TD sells that. It's a real hook **if the game makes you feel the trade**, on both the reward (evacuation) and the cost (life-support-as-volume) side.

## Recommendation
**Continue development, but redesign the meta-loop first (Phase 1) before adding any content.**

The engineering is genuinely strong — clean architecture, data-driven content, the legibility crisis already solved, a working sim end to end. That's a large, real asset and it would be a mistake to archive it. But the project is one design decision away from either mattering or not: the sacrifice must cost something persistent. Do not build a second map, a sixth tower, or an audio suite until raising the beacon is a gamble a player can lose for keeps. Prove the core loop is *tense*, then invest in content and polish. The bones are here; the game needs stakes, not features.
