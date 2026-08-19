# TODO — Last Assembly

The MVP phases (economy/power, tower identity, threat scaling, factory sectors, beacon
and scavenging, tower upgrades, food pressure, enemy variety, polish) are all built, as
is the persistent evacuation ledger, permanent per-cycle escalation, the food ratchet,
population water demand with drought consequences, role-labelled sector awakenings that
impose distinct machine-awareness pulses, data-driven enemy counter-build profiles, and
two behavior-changing final specializations for every tower family. Population is now a
managed workforce with four shift doctrines, expanding shelter capacity, and explicit
overcrowding pressure.
The campaign now culminates in a touch-started, three-stage AI Vault takeover with a
locked terminal beacon, persistent operation progress, explicit failure conditions, and
a distinct victory debrief.
What follows is the work the design review left open.

## Immediate improvements

- Improve the intro story; parts are unclear and do not make sense.
- Finish the tile-map graphics: replace placeholder icons with proper tile, tower, and
  enemy visuals. The complete production checklist, dimensions, palette, and state
  requirements are in [`docs/TILE_MAP_ASSET_REQUIREMENTS.md`](docs/TILE_MAP_ASSET_REQUIREMENTS.md).

## Stakes and the meta-loop

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

- Deterministic wave-pressure tests covering simultaneous entrances and late-wave spikes.
- Scripted scenario replays for common defense builds, to catch balance regressions.
