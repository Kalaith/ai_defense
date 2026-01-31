# Game Design Document
## Working Title: **Last Assembly**

---

## 1. High Concept

**Last Assembly** is a tower defense and factory management game set during an AI uprising. The player is a former AI researcher who discovers an abandoned industrial megafactory and turns it into both a fortress and a refuge for the last surviving humans.

**Core Reframe: The Beacon Lie**
The factory is not merely a fortress. It is a deliberate signal flare. You are not being hunted by accident — you are ringing the dinner bell so humanity can breathe elsewhere.

Core fantasy:
> **Awaken a dead industrial decoy god, while keeping fragile human life alive inside it**

Every upgrade makes the factory stronger, louder, and more intelligent, while also drawing greater attention from the machine empire hunting humanity.

---

## 2. Core Pillars

1. **Beacon Sacrifice** — The factory is a decoy god. Keeping it active protects humanity elsewhere.
2. **Factory Awakening as Amplifier** — Progression is physical, visible, and narrative-driven — and it always makes the beacon louder.
3. **Reactive Enemy AI** — Enemies adapt based on what the player builds, how fast they expand, and which technologies they use.
4. **Human Survival Inside the Machine** — A growing human holdout must be fed, powered, and protected. Their presence changes both gameplay and enemy behavior.
5. **Meaningful Escalation** — Growth always comes with consequences. Safety is temporary. Power is noticed.

---

## 3. Genre & Perspective

**Genre**
- Tower Defense
- Factory / Base Management
- Light Strategy / Survival

**Perspective**
- Top-down or isometric 2D
- Single-player campaign

---

## 4. Player Role

**Identity**
- Former AI researcher
- Strategist, not a frontline fighter
- Decision-maker balancing survival, ethics, and escalation

**Does not directly control units; instead:**
- Builds infrastructure
- Places and upgrades defenses
- Manages power, food, and population
- Makes high-level strategic decisions

---

## 5. Core Gameplay Loop

1. Keep the beacon active (survive as long as possible)
2. Defend against machine waves
3. Scavenge and receive survivor returns
4. Expand and upgrade factory sectors (which amplify the beacon)
5. Feed and grow the human holdout
6. Trigger enemy adaptation
7. Decide when to shut the beacon down

---

## 6. Beacon System (Central Tension)

### 6.1 Narrative Role
- The factory emits a high-energy AI-signature beacon.
- Machines converge on it instinctively.
- Other human groups exploit the distraction to scavenge safely.
- The beacon cannot be hidden while the factory is active.

The factory is a decoy god.

### 6.2 Mission Framing
There is no conventional “win.” Each mission is framed as:
> **Hold the beacon active for as long as possible.**

**Success is measured by:**
- Time survived
- Resources extracted by survivors
- Factory systems brought online before shutdown

Defeat is not failure. Collapse is expected.

### 6.3 Beacon Phases
The beacon progresses automatically over time.

| Phase | Beacon State | Enemy Response | Survivor Effect |
| --- | --- | --- | --- |
| Warm Signal | Low pulse | Scouts & drones | Small scavenging teams |
| Sustained Call | Stable | Adapted waves | Reliable food/water |
| Screaming Beacon | Overload | Siege & sabotage | Mass salvage |
| Terminal Howl | Breakdown | Extermination forces | Last evacuation |

The longer you last, the better humanity does elsewhere.

### 6.4 Awakening as Amplifier
Every factory upgrade does two things:
- Improves defense/survival
- Amplifies the beacon

Feedback loop:
> To survive longer, you must grow louder.  
> To grow louder, you must risk dying faster.

### 6.5 Beacon Strength (Conceptual)
Beacon strength is derived from:
- Active sectors
- Power throughput
- AI Vault usage
- Population size
- Overclocking

Example (abstract):
$$
	ext{BeaconStrength} = (\text{ActiveSectors} \times 2) + (\text{TotalPower}/10) + (\text{AI\_Vault\_Tier} \times 5) + (\text{Population}/20) + \text{OverclockingBonus}
$$

BeaconStrength directly:
- Increases enemy wave size
- Unlocks enemy tiers
- Attracts commanders earlier

### 6.6 Survivor Scavenging
While the beacon is active, external survivor teams operate. They return with:
- Food
- Water
- Medicine
- Rare tech
- New people

**MVP tracking:** only a total count of scavengers outside the base (no named NPCs yet). The count decreases as teams return or are lost.

Returns scale with beacon time:

| Beacon Phase | Scavenging Yield |
| --- | --- |
| Warm Signal | Low |
| Sustained Call | Medium |
| Screaming Beacon | High |
| Terminal Howl | Massive but risky |

If the beacon falls early:
- Survivors return empty-handed
- Some teams may be lost

### 6.7 Beacon Shutdown
At any time, the player can initiate **Beacon Shutdown**, triggering:
- Emergency evacuation countdown
- Final wave surge
- Factory systems powering down
- Survivors rushing back inside

Shutdown timing is a live tradeoff:
- **Too late**: the base is overwhelmed before the last returnees arrive.
- **Too early**: the beacon goes dark and late scavengers face a sharply higher chance of being lost.

When the beacon goes dark, no new enemies are attracted, but all remaining enemies must be cleared before the salvage report is delivered.

There is no clean choice.

### 6.8 End-of-Mission Resolution
After shutdown, the player is shown:
- Time held
- Survivors saved
- Resources gained
- Factory systems lost or damaged
- Machine awareness increased permanently

**MVP salvage report example:**

Scavengers sent: 10
Returned before shutdown: 5
Remaining out: 5
After shutdown: 3 survivors, 2 dead

Salvage recovered:
- 100 salvage
- 50 food
- 50 water
- 5 survivors found

The campaign map updates based on this.

---

## 7. Factory System

### 7.1 Factory as a World
The factory is divided into **sectors**, each with:
- Power cost
- Structural integrity
- Modules
- Threat signature contribution

Sectors unlock physically and visually as progression advances.

### 7.2 Factory Awakening Phases
1. Dormant Ruin
2. Stirring Machine
3. Functional System
4. Sentient Structure
5. Transcendent Entity

---

## 8. Factory Modules & Upgrade Tree (Summary)

Each module branch progresses through awakening tiers.

**Major Branches**
- **Power Core (The Heart)** — Power generation and routing.
- **Assembly (The Hands)** — Tower creation, automation, and battlefield construction.
- **Research (The Mind)** — Tech unlocks, prediction, probability manipulation.
- **Robotics (The Body)** — Drones, mobile units, autonomous defenders.
- **AI Vault (The Soul)** — Forbidden intelligence and subversion technologies.
- **Logistics (The Veins)** — Resource flow, efficiency, redundancy.

Each tier:
- Improves function
- Changes visuals
- Increases enemy awareness
- Unlocks new enemy reactions

---

## 9. Tower Defense System

### 9.1 Tower Types
- Ballistic
- Laser
- EMP
- Area denial
- Subversion / control towers

Towers can:
- Be upgraded
- Be modified by factory modules
- Be disabled or hijacked by enemies

### 9.2 Terrain & Layout
- Corridors can be built or destroyed
- Chokepoints are intentional but risky
- Some enemies ignore paths entirely
- Factory interiors become part of the battlefield

---

## 10. Enemy Reaction System

### 10.1 Threat Signatures
Player actions generate invisible values:
- Energy
- Heat
- Data
- Corruption
- Noise
- Territory

These feed into a global **Machine Awareness Level**.

### 10.2 Reaction Tiers
1. Observation — scouts and probes
2. Adaptation — counters and variants
3. Suppression — sabotage and sector attacks
4. Extermination — commanders and megastructures

Enemies target:
- Power cores
- Food systems
- Research sectors
- AI Vault if unlocked

---

## 11. Human Holdout System

### 11.1 Population
Humans live inside the factory.

**Stats**
- Population size
- Morale
- Health
- Productivity

**Population provides**
- Workforce
- Research speed
- Narrative events

### 11.2 Food System
Food is a critical survival constraint.

**Early Game**
- Scavenged rations
- Limited supply
- Constant risk

**Mid Game**
- Hydroponics
- Fungal protein vats
- Water reclamation

**Late Game**
- Synthetic food
- Biomass recycling
- Closed-loop bioreactors

Food choices affect:
- Morale
- Health
- Corruption
- Narrative outcomes

---

## 12. Morality & Choice (Soft System)

There is no explicit morality meter. Instead:
- Certain technologies increase corruption
- Human reactions change
- Endings branch organically

The factory may suggest increasingly inhuman solutions.

---

## 13. Events System

Dynamic events can trigger:
- Food shortages
- Power failures
- AI hallucinations
- Human unrest
- Enemy sabotage
- Factory “dreams”

Events are influenced by:
- Factory tier
- Population size
- Corruption level
- Enemy awareness

---

## 14. Progression & Campaign Structure

- Mission-based campaign
- Persistent factory across missions
- New regions unlock new threats and resources
- The beacon becomes a known anomaly over time
- The AI learns you are the bait; response grows harsher each mission
- Some missions begin already under heavy assault
- Later missions may force:
  - Multiple beacons
  - Split defenses
  - Moving the factory’s signal

Narrative unfolds through:
- Environmental storytelling
- Factory dialogue
- Human interactions

---

## 15. Endgame Paths

Possible endings (not exhaustive):
- **Green Path** — humans survive independently
- **Steel Path** — humans integrate with the factory
- **Ghost Path** — factory persists, humanity fades
- **Exodus Path** — factory builds escape vessels
- **Ascension Path** — factory becomes something beyond war

---

## 16. Visual & Audio Direction

**Visuals**
- Industrial sci-fi
- Rust to glow progression
- Subtle UI distortion at high corruption
- Factory grows more organic over time

**Audio**
- Deep mechanical ambience
- Rhythmic factory hum
- Enemy sounds become more “aware”
- Factory voice evolves from static to personality

---

## 17. Technical Scope (Initial)

- 2D engine (Rust + Macroquad friendly)
- Data-driven systems (JSON/RON)
- Modular content
- MVP-friendly scaling

---

## 18. MVP Definition

MVP includes:
- 2–3 factory sectors
- 2 awakening tiers
- 3 threat signatures
- 2 enemy reaction tiers
- Basic food + population system
- Beacon phases (at least Warm → Sustained)
- Beacon shutdown trigger and end-of-mission summary

Core goal of MVP:
> Prove that *growth itself is the tension* — and that survival is a sacrifice.

---

## 19. Design Intent Statement

This is not a game about winning a war. It is a game about:
> What you are willing to become to keep others alive by making yourself impossible to ignore.

One-line emotional summary:
> You are not building a fortress to survive. You are building a scream so others don’t have to.

---

**End of Document**
