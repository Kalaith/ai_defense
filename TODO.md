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
- Finish the remaining tile-map art polish: add the four depth-band motif sets and
  the final ability-state overlay pass. The production checklist, dimensions,
  palette, state requirements, and new depth/ability contract are in
  [`docs/TILE_MAP_ASSET_REQUIREMENTS.md`](docs/TILE_MAP_ASSET_REQUIREMENTS.md).

## Stakes and the meta-loop

## Content

- A second and third map / route layout.
- Mission structure with the escalating framings from the design doc (start-under-assault,
  multi-beacon).
- Additional depth directive art: replace the geometric cards with three small
  insignia while preserving the current touch-first text and button contract.

## Presentation

- Audio, from zero: phase-driven beacon hum, breach alarm, tower fire, sector power-up.
- Awakening visuals — rust giving way to glow as sections wake, UI distortion at high
  corruption, and stronger band-specific framing as the factory opens inward.
- One line of context on the beacon button naming the trade the player is about to make.

## Testing

- Deterministic wave-pressure tests covering simultaneous entrances and late-wave spikes.
- Scripted scenario replays for common defense builds, to catch balance regressions.
