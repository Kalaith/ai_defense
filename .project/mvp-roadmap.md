# Last Assembly — MVP Roadmap

This document breaks the path to MVP into concrete phases. Each phase builds on the previous one and results in a testable, playable state.

MVP goal from the design doc:

> Prove that *growth itself is the tension*.

MVP scope: 2-3 factory sectors, 2 awakening tiers, 3 threat signatures, 2 enemy reaction tiers, basic food + population system, beacon phases (at least Warm → Sustained), beacon shutdown trigger and end-of-mission summary.

---

## Current State (Baseline)

What works today:

- Menu → Gameplay → Results state transitions
- Wave spawning with budget/tier system, 5 enemy types
- Enemies walk a grid-aligned path with health bars
- 5 tower types placeable from build panel, all fire at nearest enemy
- Scrap awarded on kill, deducted on build
- Factory sectors visible, unlockable for scrap, trigger awakening phases
- Threat signature tracks 6 dimensions with passive decay, displayed in HUD
- Population loses 1 per enemy reaching the end
- Food consumed on wave complete (flat +10 food, then consume)
- Loss on population 0 or factory integrity 0
- Pause, placement ghost with invalid-position feedback

What's stubbed or disconnected:

- All 5 tower types behave identically (no type-specific mechanics)
- Tower upgrades (level field exists, never used)
- Power is a number on screen — never consumed, never generated
- Factory modules (empty vector, no effects)
- Sector integrity never changes (field exists, never damaged)
- Threat level is cosmetic — doesn't affect waves or enemy behavior
- Population productivity calculated but unused
- Enemy types all walk the same path with no special behavior
- Save/load defined but never called
- No visual/audio feedback for combat (no shot lines, no hit flash)

---

## Phase 1 — Economy & Power Loop ✓ COMPLETE

**Goal**: Resources matter. Power constrains growth. Scrap feels scarce.

### What was implemented

**Power rate system** (generation vs consumption per second):
- Power Core sector generates 20 power/s; each additional unlocked sector adds +2 power/s
- Each tower drains `cost_power` per second (stored as `power_drain` on Tower struct)
- Each unlocked sector drains `power_cost` per second
- `resources.power` is a buffer pool: starts at 50, capped at 200, fills/drains based on net rate
- **Brownout**: when pool hits 0 and net rate is negative, towers deactivate oldest-first until drain ≤ generation
- **Recovery**: when pool > 0, all inactive towers reactivate
- Tower placement costs only scrap (no power deduction). Blocked if adding the tower's drain would push net rate negative AND pool < 10
- HUD shows `"Power: {pool} (+{rate}/s)"` — green when positive, red when negative

**Scrap economy**:
- Starting scrap: 75 (from 100)
- Wave-end bonus: `15 + wave * 3` (from `20 + wave * 5`)
- Kill scrap unchanged

**Population productivity**:
- Each frame adds `productivity * 0.02 * dt` scrap
- Population loss directly hurts scrap income

### Key implementation details for future phases
- `Tower` struct has `power_drain: f32` and `is_active: bool` fields
- `Factory::power_generation()` and `Factory::power_consumption()` methods exist
- Power Core's +20/s generation and per-sector +2/s bonus are already live (Phase 4 should not re-add these)
- Brownout deactivates towers by iteration order (oldest first); future phases should preserve this ordering

---

## Phase 2 — Tower Identity ✓ COMPLETE

**Goal**: Tower choice matters. Each type plays a distinct role.

### 2.1 Type-specific combat behavior
- **Ballistic**: No change (single target, reliable damage) — the baseline
- **Laser**: Double damage to HeavyUnit, half damage to Scout (armor piercing)
- **EMP**: On hit, set a `slowed_timer` on the enemy (reduce speed by 50% for 2s). Low damage compensated by control
- **Area Denial**: Damage all enemies within range (not just nearest). Reduce per-target damage to compensate
- **Subversion**: On kill, nearby enemies take 30% of killed enemy's max_health as damage (chain reaction)

### 2.2 Enemy `slowed_timer` field
- Add `slowed_timer: f32` to Enemy
- `move_along_path` checks slowed_timer: if > 0, speed *= 0.5, decrement timer by dt

### 2.3 Visual feedback
- Draw a line from tower to target when firing (flash for 0.1s)
- Color the line by tower type
- Area denial draws a brief pulse circle instead
- Brownout-deactivated towers render dimmed (50% alpha) — `is_active` already exists from Phase 1

### 2.4 Power drain considerations
- Different tower types have different `power_drain` values (set via `cost_power` in tower defs)
- EMP and Subversion should have higher power drain to create meaningful tower-mix decisions
- Area Denial's continuous AoE should have moderate drain to balance its effectiveness

### Verification
- EMP visibly slows enemies
- Laser melts heavy units faster than ballistic
- Flak Field damages clumps
- Subversion creates chain kills in tight groups
- Shot lines visible during combat
- Deactivated towers are visually distinct from active ones

---

## Phase 3 — Threat Drives Difficulty ✓ COMPLETE (see note)

**Goal**: The threat system has teeth. Expanding the factory makes enemies harder.

### 3.1 Threat influences wave generation
- Pass `threat.awareness_level()` into `generate_wave`
- Threat adds bonus budget: `awareness_level / 10` extra budget points
- Threat adds bonus health scaling: `1.0 + awareness_level * 0.005` multiplier on top of wave scaling

### 3.2 Reaction tier unlocks enemy tiers earlier
- Adaptation (25+): Unlock tier 2 enemies regardless of wave number
- Suppression (60+): Unlock tier 3 enemies regardless of wave number
- This means aggressive expansion can face Commanders on wave 3

### 3.3 Threat from tower placement
- Each tower placed adds `noise += 0.5`
- Firing towers add `heat += 0.01 * dt` while actively shooting

### 3.4 Threat display improvement
- Show numeric awareness level in HUD alongside tier name
- Flash the threat indicator when tier increases

### Verification
- Rush-unlocking all sectors → threat spikes → enemies get noticeably harder
- Slow cautious play keeps threat low, enemies stay manageable
- Player feels the tradeoff: expand for power vs stay quiet for safety

---

## Phase 4 — Factory Sectors Do Things ✓ COMPLETE

**Goal**: Sectors are meaningful strategic choices, not just unlock-for-phase-progression.

### 4.1 Sector effects on unlock
Each sector grants a passive bonus. Note: Power Core's base generation (+20/s) and per-sector generation bonus (+2/s each) are already implemented in Phase 1 via `Factory::power_generation()`. This phase adds the *other* sector bonuses and the integrity/disable system.

- **Power Core** (starts unlocked): Generation already active. New: if disabled (integrity 0), `power_generation()` returns 0 — triggers brownout cascade
- **Assembly Hall** (starts unlocked): +10% tower damage (applied as multiplier in `tick_towers`)
- **Research Lab**: Unlocks tower upgrades (Phase 6 prerequisite)
- **Robotics Bay**: +15% tower fire rate (applied as multiplier in `tick_towers`)
- **AI Vault**: Towers gain +20% range, but +5 territory threat on unlock
- **Logistics Hub**: +25% scrap from kills

### 4.2 Sector integrity matters
- Enemies that reach the end damage a random unlocked sector's integrity (-5 per breach)
- If a sector reaches 0 integrity, it becomes **disabled** (loses its bonus, stops contributing to `power_generation()`)
- Disabled sector still drains `power_cost` (damaged systems leak power) — this creates urgency to repair
- Player can spend scrap to repair sectors (cost: 20 scrap per 25 integrity)
- Disabled sector still counts as "unlocked" for awakening phase but provides no bonus

### 4.3 Sector disable interacts with brownout
- When Power Core is disabled: generation drops to 0 + other sectors' +2/s bonuses are lost → immediate brownout likely
- When any sector is disabled: its power drain continues but its generation bonus (+2/s) is lost
- This makes sector integrity a direct power economy concern, not just a bonus toggle

### 4.4 Sector repair UI
- Damaged sectors show a "Repair (20)" button in the sector panel
- Disabled sectors show in red with "OFFLINE" label
- Power Core shows current generation rate to help player diagnose brownouts

### Verification
- Losing Logistics Hub means scrap income drops
- Losing Power Core means power generation drops to near-zero → brownout deactivates towers oldest-first
- Repairing sectors is a meaningful scrap expenditure decision
- Unlocking AI Vault is a clear risk/reward choice (range bonus vs threat spike)
- Disabling a high-drain sector while keeping it "unlocked" creates a power deficit the player must address

---

## Phase 5 — Beacon & Scavenging ✓ COMPLETE (needs update: separate recall/shutdown flow)

**Goal**: The beacon is the reason the game exists. Growth makes you louder. Louder draws worse things. The player chooses when to stop.

This phase implements the core gameplay loop steps that are currently missing: beacon activation, survivor scavenging, beacon shutdown, and end-of-mission resolution (design doc sections 6.1–6.8). Without this, the game is a standard tower defense — with it, the game becomes the "decoy god sacrifice" described in the design doc.

### 5.1 Beacon strength calculation
Beacon strength is a derived value, computed each frame from live game state:
```
beacon_strength = (unlocked_sectors * 2) + (power_throughput / 10) + (ai_vault_tier * 5) + (population / 20)
```
- `power_throughput` = `factory.power_generation()` (from Phase 1)
- `ai_vault_tier` = 0 if AI Vault locked/disabled, tier level otherwise (from Phase 4)
- Beacon strength is displayed in the HUD

### 5.2 Beacon phases
Beacon phase advances based on beacon strength thresholds (not time alone):

| Phase | Strength Threshold | Enemy Effect | Scavenging Yield |
|-------|-------------------|--------------|-----------------|
| Warm Signal | 0–15 | Scouts & drones only | Low |
| Sustained Call | 16–35 | Adapted waves (tier 2 unlocked) | Medium |
| Screaming Beacon | 36–60 | Siege & sabotage (tier 3 unlocked) | High |
| Terminal Howl | 61+ | Extermination forces (commanders guaranteed) | Massive but risky |

- Beacon phase feeds into `generate_wave` alongside threat (Phase 3) — beacon phase sets a floor on enemy tier availability
- Phase transitions are one-way during a mission (beacon cannot quiet down once it escalates)
- Visual: HUD shows current beacon phase name with pulsing intensity

### 5.3 Survivor scavenging
While the beacon is active, external survivor teams operate autonomously:
- Track `scavengers_out: u32` (number of teams currently scavenging)
- At mission start: send out initial teams (e.g. 3 teams)
- Each wave completion: one team may return with resources based on beacon phase
  - Warm Signal: 5 food, 5 scrap
  - Sustained Call: 10 food, 10 scrap, chance of +1 population
  - Screaming Beacon: 20 food, 15 scrap, +1 population
  - Terminal Howl: 30 food, 25 scrap, +2 population, but 20% chance team is lost
- MVP: no named NPCs, just a count of teams out and a returns log
- HUD shows `"Scavengers: {out} out / {recalled} recalled"` — returns appear as brief notification text

### 5.4 Recall scavengers (separate from shutdown)
At any time the player can press a "Recall Scavengers" button:
- Begins pulling scavenger teams back — teams return over time
- **Beacon stays on** — waves continue at current intensity while scavengers return
- The player must keep defending while waiting for teams to come home
- This is the intended flow: recall first, hold out, then shutdown once teams are safe

### 5.5 Beacon shutdown (player-initiated end)
At any time after wave 1, the player can press a "Shutdown Beacon" button:
- **No new waves spawn** after shutdown
- If a wave is currently in progress, that wave **plays out to completion** — enemies already on the field finish their attack
- If shutdown occurs between waves, no further wave spawns
- Any scavenger teams still out when the beacon shuts down have a **flat 20% chance of being lost** per team
- Once the current wave (if any) is cleared, the mission ends and transitions to results

**Optimal play**: Recall scavengers → fight waves while waiting for all teams to return → shutdown beacon once everyone is home → survive the current wave → clean results.

**Desperate play**: Shutdown beacon immediately to stop waves — risk losing scavengers still out.

If the player never shuts down, the mission ends on loss (population 0 or factory integrity 0) — all scavengers still out are lost. This is a valid outcome but scores worse than a controlled shutdown.

### 5.6 End-of-mission resolution
Replace the current bare Results state with a proper salvage report:
- Time survived (waves completed)
- Beacon phase reached
- Scavengers sent / returned / lost
- Resources gained by external survivors (the "point" of the beacon)
- Factory systems online at shutdown
- Population surviving

The key emotional beat: even if the factory falls, the longer you held the beacon, the more humanity benefited elsewhere. Defeat is not failure.

### 5.7 Interaction with existing systems
- **Phase 1 (Power)**: `power_throughput` feeds beacon strength — more power = louder beacon
- **Phase 3 (Threat)**: Beacon phase and threat are separate escalation axes. Threat makes current enemies harder; beacon phase unlocks new enemy tiers. Both grow with expansion but via different mechanisms
- **Phase 4 (Sectors)**: Each sector unlocked raises beacon strength by +2. Disabling a sector does NOT reduce beacon strength (the signal was already sent — you can't un-ring the bell)

### Verification
- Unlocking sectors → beacon strength rises → beacon phase advances → harder enemy tiers appear
- Scavenger teams return with resources on wave complete — yields scale with beacon phase
- Recall button pulls scavengers back while waves continue — player must keep defending
- Shutdown button stops new waves; current in-progress wave plays out to completion
- Shutdown between waves → immediate mission end transition
- Scavengers still out at shutdown have 20% chance of being lost per team
- Factory falls → all scavengers still out are lost
- Results screen shows salvage report with all mission stats
- Losing without shutdown is possible but the results screen reflects it (worse survivor outcomes)

---

## Phase 6 — Tower Upgrades ✓ COMPLETE

**Goal**: Invested towers are valuable. Losing sectors that buff them hurts.

### 6.1 Upgrade system
- Requires Research Lab unlocked
- Click a placed tower to select it → show upgrade button in build panel
- Upgrade costs: `base_scrap_cost * level` scrap
- Max level: 3
- Each level: +30% damage, +10% range
- Visual: tower circle grows slightly per level, gains a ring

### 6.2 Upgrade UI
- Clicking a placed tower highlights it and shows stats + upgrade button in build panel
- If Research Lab is offline/locked, upgrade button shows "Requires Research Lab"

### Verification
- Can't upgrade until Research Lab unlocked
- Level 3 tower is noticeably stronger
- Losing Research Lab doesn't un-upgrade existing towers but prevents new upgrades

---

## Phase 7 — Population & Food Pressure ✓ COMPLETE

**Goal**: Population is a living system, not just a hit counter.

### 7.1 Continuous food consumption
- Food consumed per frame: `population.count * 0.02 * dt` (roughly 1 food per person per 50 seconds)
- Remove the flat +10 food on wave complete
- Instead: each wave complete awards `food_per_wave` from constants (10)
- Logistics Hub bonus (Phase 4): +50% food from wave completion
- Scavenger returns (Phase 5) provide additional food scaled by beacon phase — this is the primary food income at higher beacon phases

### 7.2 Starvation consequences
- When food hits 0: morale drops 2/s, health drops 1/s
- When health < 20: population starts dying (1 person per 5 seconds)
- When morale < 20: productivity drops to 25% of normal (less scrap generation)
- Since Phase 1 connects `productivity * 0.02 * dt` to scrap income, starvation creates a compounding economic spiral: low morale → low productivity → less scrap → can't repair sectors → brownouts → towers offline

### 7.3 Population recovery
- When food > 0 and morale < 50: morale slowly recovers (+1/s)
- When food > 0 and health < 80: health slowly recovers (+0.5/s)

### 7.4 Population display
- Add food supply bar to HUD
- Add morale indicator (text color: green > 50, yellow 20-50, red < 20)
- Scrap income rate from productivity visible on hover or in expanded HUD

### Verification
- Ignoring food leads to slow population death spiral
- Keeping food stocked keeps productivity high
- Player must balance tower spending vs ensuring wave completion for food
- Starvation spiral is recoverable if caught early but fatal if ignored (scrap income collapses)

---

## Phase 8 — Enemy Variety ✓ COMPLETE

**Goal**: Enemy types feel distinct. Composition matters for tower selection.

### 8.1 Scout behavior
- 20% chance to dodge an attack (take 0 damage). Visual: brief flicker
- When a Scout reaches the end, it doesn't damage integrity — instead it increases threat noise by +3

### 8.2 Saboteur behavior
- Partially ignores path: at each waypoint, 30% chance to skip the next waypoint (takes a shortcut)
- On reaching the end: damages a specific sector by 15 integrity (targets lowest-integrity unlocked sector)
- Saboteurs targeting Power Core can trigger brownout cascades — this is intentional and creates high-priority targeting decisions

### 8.3 Commander aura
- While alive, all other enemies within 150 units gain +25% speed
- On death: all enemies within 150 units take 10% of their max HP as damage (disruption burst)

### 8.4 HeavyUnit
- Takes 50% reduced damage from Ballistic towers (armor)
- Takes 150% damage from Laser towers
- Normal damage from everything else

### Verification
- Mixed waves require mixed tower types
- Saboteurs punish single-path-only defense
- Commanders make focus-fire priority calls meaningful
- Scouts are annoying but not deadly — they escalate threat

---

## Phase 9 — Polish & Game Feel ✓ COMPLETE

**Goal**: The game feels responsive and readable.

### 9.1 Combat juice
- Tower fires: brief colored line to target (fades over 0.1s)
- Enemy death: small particle burst (4-6 squares scatter outward)
- Enemy hit: brief white flash on the enemy circle
- Wave start: screen-edge flash + "WAVE X" text that fades

### 9.2 Sound (if time permits)
- Tower fire: short click/zap per type
- Enemy death: crunch
- Wave start: alarm tone
- Sector unlock: power-up hum

### 9.3 Quality of life
- Tooltip on tower hover: shows name, level, damage, range, fire rate
- Wave preview: before wave starts, show upcoming enemy composition as icons
- Speed control: press Space to toggle 2x speed

### 9.4 Save/load
- Auto-save on wave complete (serialize game state to SaveData)
- Load from menu screen ("Continue" button, only shown if save exists)
- Save stores: wave, resources, tower positions/types/levels, sector states, threat, population

### Verification
- Combat is visually clear — player can read what's happening
- Player can resume a run after closing the game
- 2x speed makes between-wave downtime tolerable

---

## Phase Summary

| Phase | Focus | Key Deliverable | Status |
|-------|-------|-----------------|--------|
| 1 | Economy & Power | Power pool/rate system, brownout, scrap tuning | ✓ Done |
| 2 | Tower Identity | 5 towers play differently, shot lines, dimmed inactive towers | ✓ Done |
| 3 | Threat Drives Difficulty | Expansion makes enemies harder | ✓ Done (missing: tower placement/firing threat per §3.3) |
| 4 | Factory Sectors | Sector bonuses, integrity/disable, brownout cascades | ✓ Done |
| 5 | **Beacon & Scavenging** | **Beacon strength/phases, survivor scavenging, recall/shutdown, results** | ✓ Done (needs update: separate recall/shutdown per §5.4–5.5) |
| 6 | Tower Upgrades | Invested towers are valuable | ✓ Done |
| 7 | Population & Food | Population is a living pressure system | ✓ Done |
| 8 | Enemy Variety | Enemy types have distinct behavior | ✓ Done |
| 9 | Polish & Game Feel | Combat juice, QoL, save/load | ✓ Done |

Phases 1-5 establish the core tension loop (the MVP thesis). Phase 5 is the keystone — without the beacon, the game is a standard tower defense. With it, the game becomes the "decoy god sacrifice" the design doc describes.

Phases 6-8 add depth and replayability.
Phase 9 makes it feel like a game instead of a prototype.

After Phase 5, all 7 steps of the core gameplay loop (design doc §5) are implemented:

| Loop Step | Phase |
|-----------|-------|
| 1. Keep the beacon active | Phase 5 (beacon strength, phases) |
| 2. Defend against machine waves | Baseline + Phases 2, 3, 8 |
| 3. Scavenge and receive survivor returns | Phase 5 (scavenging system) |
| 4. Expand and upgrade factory sectors | Phase 4 (sector bonuses) + Phase 6 (tower upgrades) |
| 5. Feed and grow the human holdout | Phase 7 (food pressure) + Phase 5 (scavenger food returns) |
| 6. Trigger enemy adaptation | Phase 3 (threat) + Phase 5 (beacon phase unlocks enemy tiers) |
| 7. Decide when to shut the beacon down | Phase 5 (shutdown trigger, evacuation, results) |

---

## Cross-Phase Interactions

Phase 1's power pool/rate/brownout system creates emergent interactions with later phases:

- **Phase 2 + 1**: Tower power drain varies by type → tower composition affects power budget, not just combat effectiveness
- **Phase 3 + 1**: High threat → harder waves → more towers needed → more power drain → closer to brownout threshold
- **Phase 4 + 1**: Sector disable removes generation bonus and Power Core disable triggers cascade brownout; disabled sectors still drain power
- **Phase 7 + 1**: Starvation → low productivity → less scrap income → can't repair sectors or build towers → brownout spiral
- **Phase 8 + 1**: Saboteurs targeting Power Core can instantly shift the power balance and deactivate defenses mid-wave

Phase 5's beacon system ties everything together:

- **Phase 5 + 1**: Power throughput feeds beacon strength — more power generation = louder beacon = harder enemies
- **Phase 5 + 3**: Beacon phase and threat are parallel escalation axes — threat makes current enemies harder, beacon phase unlocks new tiers
- **Phase 5 + 4**: Each unlocked sector raises beacon strength — sector expansion is beacon amplification (design doc §6.4)
- **Phase 5 + 7**: Scavenger returns are the primary food source at higher beacon phases — shutting down too early starves the holdout
- **Phase 5 + 8**: Beacon phase determines which enemy types can appear — Terminal Howl guarantees commanders regardless of wave number
