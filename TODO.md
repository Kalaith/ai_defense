# TODO — Last Assembly

The MVP phases (economy/power, tower identity, threat scaling, factory sectors, beacon
and scavenging, tower upgrades, food pressure, enemy variety, polish) are all built, as
is the persistent evacuation ledger, permanent per-cycle escalation, and the food ratchet.
What follows is the work the design review left open.

## Immediate improvements

- Improve the intro story; parts are unclear and do not make sense.
- Finish the tile-map graphics: replace placeholder icons with proper tile, tower, and
  enemy visuals.
- Improve the opening balance and tutorial: one starting tower is insufficient, but the
  tutorial only explains that option. Start with or introduce multiple turrets.

## Stakes and the meta-loop

- Make water a real demand: the population consumes it, running dry kills people. Today
  water is produced and costs beacon volume but nothing ever draws on it — a dead stake,
  and the cheapest way to land the "grow louder just to keep your own people breathing" axis.
- Split the factory into life-support vs. military/economy sectors and make population a
  managed lever — grow the holdout for workforce and evacuation, but every mouth needs
  food/water buildings that raise beacon volume.
- Tie sector unlocks to explicit threat trade-offs rather than a flat scrap cost.
- A real win state: the vault takeover the fiction promises is unimplemented, so a run only
  ends when the player stops or the holdout dies.

## Combat depth

- 2–3 counter-build enemies that punish a specific tower type, so tower choice is situational.
- Branching upgrades at max tower level — one identity fork per tower instead of flat +%.

## Content

- A second and third map / route layout.
- Mission structure with the escalating framings from the design doc (start-under-assault,
  multi-beacon).

## Presentation

- Audio, from zero: phase-driven beacon hum, breach alarm, tower fire, sector power-up.
- Awakening visuals — rust giving way to glow as sections wake, and UI distortion at high
  corruption.
- One line of context on the beacon button naming the trade the player is about to make.

## Testing

- Deterministic wave and tower-targeting tests covering simultaneous enemies, priority ties,
  and late-wave pressure spikes.
- Scripted scenario replays for common defense builds, to catch balance regressions.
