# Last Assembly

Last Assembly is a tower defense and factory survival game about turning a dead industrial complex into a beacon, a fortress, and a temporary refuge for the last humans alive during an AI uprising.

You are not trying to hide. You are deliberately making noise so scavenger teams outside the base can move while the machines converge on you.

## Core Idea

Keep the beacon active for as long as you can.

The longer it runs, the more food, scrap, and survivors your scavenger teams can bring back. The longer it runs, the more enemies it attracts. You choose when to start the beacon, how far to expand the factory, and when to shut everything down before the base collapses.

## What You Do

- Place towers on powered pads to hold active attack paths.
- Clear debris and power new pads to extend your defenses.
- Repair and power factory systems to unlock resources, technologies, and new map areas.
- Balance population, food, morale, scrap, power, and machine awareness.
- Keep scavenger teams outside long enough to matter, then recall them before the situation turns terminal.
- Survive each wave, then decide whether to keep drawing enemies or shut the beacon down.

## The Main Tension

Expansion is never free.

Powering more of the factory gives you stronger tools: more production, better defenses, and access to later technologies. It also reveals new areas, opens new enemy approaches, and makes the beacon louder.

The central question is:

How much danger can you afford to invite before the resources stop being worth it?

## Map Progression

The factory is divided into connected systems. At the start, only the first sections are readable and usable. As you repair and power core machines, more of the map comes online.

New areas can provide:

- Tower pads and better chokepoints.
- Scrap, water, food, or power production.
- Technology upgrades.
- New attack routes into the base.

Dim silhouettes show future factory areas. Bright powered traces show the current active power network. Orange paths show enemy routes.

## Beacon And Scavengers

The beacon is the mission clock and the risk engine.

When the beacon is active:

- Waves begin coming in.
- Scavenger teams remain outside the base.
- Beacon strength rises with factory growth, power throughput, population, and advanced systems.
- Higher beacon phases increase reward potential and enemy pressure.

Beacon phases:

- Warm Signal: low early risk, low scavenger yield.
- Sustained Call: better returns, stronger enemy response.
- Screaming Beacon: high returns, dangerous pressure.
- Terminal Howl: huge potential rewards, high chance of losses.

Shutdown stops new waves from being attracted, but remaining enemies still have to be cleared and teams still need to make it home.

## Resources

Scrap is used for towers, repairs, powered pads, and upgrades.

Power keeps the defense grid and tower network online. If demand outruns generation and stored power, towers can go offline.

Food keeps the population alive. Food shortages damage morale and health.

Population improves productivity, but also increases pressure on food and can make the beacon stronger.

Morale and health are survival signals. If the holdout breaks, the factory does not matter.

## Towers

Current tower roles:

- Ballistic Turret: reliable early kinetic damage.
- Laser Emitter: high-energy damage, especially useful against heavier units.
- EMP Pylon: low damage, strong disruption and slowing.
- Flak Field: area denial against groups.
- Signal Jammer: subversion damage that can chain through nearby machines.

Towers must be placed on powered pads. Some upgrades improve tower damage, range, fire rate, power efficiency, or maximum level.

## Enemies

Current enemy types:

- Scout Drone: fast, light pressure.
- Assault Drone: standard combat unit.
- Siege Walker: slow, durable pressure.
- Infiltrator: fast sabotage threat.
- Command Node: heavy commander-class machine.

The enemy response escalates as waves advance and machine awareness rises.

## Basic Controls

- Mouse: interact with UI, select slots and buildings, place towers.
- Middle mouse drag: pan the map.
- Mouse wheel: zoom the map.
- Esc: pause or cancel placement.

## Reading The UI

- TOWERS: available defenses and their scrap/power costs.
- SURVIVAL: population, food, morale, scrap, power, and factory phase.
- WAVE: next wave timing or current wave pressure.
- BEACON DRAW: beacon phase, scavenger teams, threat, and start/shutdown/recall controls.
- SYSTEMS: revealed factory machines and what they provide or still need.
- Bottom context panel: selected pad, building, tower, or factory console actions.

## Current Build Goal

The playable goal is to make the first factory expansion loop feel clear:

1. Build enough defense to survive the first waves.
2. Use scrap to clear and power useful pads.
3. Bring factory systems online for resources and upgrades.
4. Keep the beacon running while scavengers gather more.
5. Shut down before the draw becomes fatal.

The game is still in active development, but the intended experience is already centered on that tradeoff: every step toward safety makes the base harder for the machines to ignore.

## Local Preview

After publishing, the WebGL build is available at:

http://127.0.0.1/games/ai_defense/
