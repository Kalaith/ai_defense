# Outstanding work — Last Assembly

Verified 23 August 2026 against the current game, data, assets, and recent changes.
The core campaign, four factory depths and directives, enemy abilities, AI Vault finale,
and the required tile-map asset files are in place. The items below remain open.

## Player experience

- Rewrite the intro so it clearly explains the factory, the AI Vault, and why waking
  sections increases the swarm's pressure.
- Add a short, contextual line beside the beacon control explaining the immediate
  trade-off before the player activates it.
- Add awakening feedback: rust yielding to power as sections wake, high-corruption UI
  distortion, and stronger depth-specific framing as the player moves inward.

## Content

- Create two additional map/route layouts.
- Add mission variants built around the design's start-under-assault and multi-beacon
  scenarios.
- Replace the depth-directive cards' geometric accents with three compact insignia,
  while keeping their text and touch controls unchanged.

## Audio

- Add phase-driven beacon hum, breach alarm, tower-fire, and sector-power-up sounds.

## Quality assurance

- Add deterministic wave-pressure tests for simultaneous entrances and late-wave
  spikes.
- Add scripted scenario replays for representative defense builds to catch balance
  regressions.
