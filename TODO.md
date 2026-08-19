# TODO — Last Assembly

The MVP phases (economy/power, tower identity, threat scaling, factory sectors, beacon
and scavenging, tower upgrades, food pressure, enemy variety, polish) are all built, as
is the persistent evacuation ledger, permanent per-cycle escalation, the food ratchet,
population water demand with drought consequences, role-labelled sector awakenings that
impose distinct machine-awareness pulses, data-driven enemy counter-build profiles, and
two behavior-changing final specializations for every tower family.
What follows is the work the design review left open.

## Immediate improvements

- Improve the intro story; parts are unclear and do not make sense.
- Finish the tile-map graphics: replace placeholder icons with proper tile, tower, and
  enemy visuals. The complete production checklist, dimensions, palette, and state
  requirements are in [`docs/TILE_MAP_ASSET_REQUIREMENTS.md`](docs/TILE_MAP_ASSET_REQUIREMENTS.md).
- Improve the opening balance and tutorial: one starting tower is insufficient, but the
  tutorial only explains that option. Start with or introduce multiple turrets.

## Stakes and the meta-loop

- Deepen the sector role split by making population a managed lever — grow the holdout
  for workforce and evacuation, but every mouth needs life-support machinery that raises
  beacon volume.
- A real win state: the vault takeover the fiction promises is unimplemented, so a run only
  ends when the player stops or the holdout dies.

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
