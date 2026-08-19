# Last Assembly

Last Assembly is a tower defense and factory survival game about waking a dead industrial complex during an AI uprising.

What is left of a town shelters in the complex's depths, down near the core. You are working inward from the rim to wake the machine section by section, because the vault at the centre only answers a machine that is whole — and the AI asleep in that vault is the one that let the swarm off its leash. Take it back and the swarm stops.

There was never enough scrap for all of it, so teams go outside for the rest, and the beacon that covers them is also what tells the swarm where you are. Wake only what the grid can carry. Every machine left cold, the town pays for in people.

You do not tell anyone that you were the lead researcher who unlocked the last AI.

## Gameplay

- Place towers on powered pads to hold attack routes.
- Read incoming armor profiles and counter each machine with the right tower family.
- Take max-level towers into one of two irreversible final assemblies.
- Clear debris and bring factory sections back online.
- Balance food, scrap, morale, population, power, and enemy attention.
- Keep outside scavenger teams active long enough to matter.
- Decide when to expand, when to defend, and when to shut the beacon down.

## Goal

Reach the vault at the centre and take the AI back. Every section you wake carries you further in, and makes you louder. The campaign is currently open-ended — the takeover itself is not implemented yet, so a run ends when you choose to stop or when the machine goes cold with you inside it.

## Controls

- Mouse: place towers and use the interface.
- Esc: pause.

## Writing

All player-facing text lives in `assets/strings.json` — nothing is hardcoded in Rust. Native builds read `assets/strings.json` from the working directory at startup, so while iterating from the project folder (`cargo run`, the capture scripts) you can edit any line and just relaunch — no `cargo build` needed.

Everything else falls back to the copy embedded at compile time: WASM, a deployed build that ships without the `assets/` folder beside it, or a file that is missing or malformed. A parse failure is logged to stderr and the game still runs on the embedded text, so a bad edit degrades rather than breaks.

Templates use named placeholders like `{n}` or `{name}`; keep them intact when rewriting a line, since the code fills them in. Each field's accepted placeholders are documented in `src/data/strings.rs`. `cargo test` parses the embedded copy, so a broken `strings.json` fails the suite rather than shipping.

## Current Scope

Playable tower defense and factory-management loop with resource pressure, wave survival, and strategic expansion.

Remaining work is listed in `TODO.md`.
