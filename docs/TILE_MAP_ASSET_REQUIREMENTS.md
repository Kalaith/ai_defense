# Tile-map asset requirements

This is the production checklist for replacing the geometric placeholders on the
Last Assembly gameplay map. It covers the entire current map: 80 tower pads, 13
machine nodes using 9 machine types, 10 factory sections, 5 breach routes, 5 tower
types, 5 enemy types, and the central factory core.

The setting is an old automated factory that has fallen into ruin. The player is
repairing it section by section, so every friendly structure must visibly progress
from corroded and broken, through mechanically repaired, to powered and alive.
Enemies are scavenged or purpose-built mixtures of cyborg tissue, exposed machinery,
industrial robots, and military hardware. They must not look like clean consumer
robots or simple colored shapes.

## Delivery rules

- Format: PNG, sRGB, 8-bit RGBA.
- Camera: top-down with restrained three-quarter height cues. Footprints must remain
  readable from directly above and must not hide adjacent paths or pads.
- Filtering: pixelated/nearest-neighbour. Use hard, deliberate clusters and avoid
  one-pixel noise that disappears when scaled.
- Lighting: dim neutral light from the upper left. Do not bake a global scene colour
  or long cast shadows into transparent sprites.
- Outline: one to two source pixels in dark blue-black (`#071014`) where separation
  from the floor is needed. Avoid solid black cartoon outlines.
- Transparent asset background: fully transparent RGBA `#00000000`.
- Opaque floor base: oxidised near-black green `#08100F`.
- No text, letters, numbers, health bars, selection rings, range rings, or status
  colours baked into art. The game draws those overlays.
- Leave at least 4 transparent pixels around movable sprites and 6 around structures.
- State silhouettes must stay aligned between frames so changing state does not make
  an object jump.

## Core palette

| Use | Colour |
| --- | --- |
| Deep floor/background | `#08100F` |
| Recesses and outline | `#071014` |
| Cold steel | `#263438` |
| Worn steel highlight | `#536267` |
| Oxidised copper | `#486A62` |
| Dark rust | `#5A2C20` |
| Fresh rust | `#9A4E2C` |
| Repair/scaffold amber | `#D49A2A` |
| Friendly power | `#38D77A` |
| Friendly control light | `#58B9D8` |
| Enemy warning red | `#DC3D32` |
| Enemy hot core | `#FF6A3D` |
| Organic cyborg tissue | `#765B58` |

Friendly repaired machinery should read as reclaimed factory equipment: mismatched
plates, fresh welds, amber clamps, and new cabling over old rust. Powered machinery
adds restrained green/cyan illumination; it does not become clean. Enemy red/orange
light is reserved for hostile units and open breaches.

## 1. Factory tile atlas

Deliver `assets/map/factory_tiles.png` at **512 x 512 px**, arranged as an **8 x 8
atlas of 64 x 64 px cells**. Cells 00-39 and 52-63 are opaque/full-bleed over
`#08100F`. Cells 40-51 are transparent overlays on `#00000000`.

Every tile must loop cleanly against compatible neighbours. Wall and route connection
letters use N/E/S/W screen directions.

| Cell | Asset | Required visual |
| ---: | --- | --- |
| 00 | `floor_plate_intact_a` | Riveted steel plates, quiet base variant |
| 01 | `floor_plate_intact_b` | Alternate seams and bolts |
| 02 | `floor_plate_intact_c` | Subtle inset maintenance hatch |
| 03 | `floor_plate_cracked_a` | Split plate, exposed dark underfloor |
| 04 | `floor_plate_cracked_b` | Buckled seam and missing fasteners |
| 05 | `floor_plate_rusted_a` | Broad dark-rust bloom |
| 06 | `floor_plate_rusted_b` | Flaked surface and fresh orange rust |
| 07 | `floor_plate_oil_stain` | Dull irregular oil spill, no shine ring |
| 08 | `floor_plate_shallow_water` | Dirty pooled water and faint reflection |
| 09 | `floor_grate` | Industrial grate over black machinery void |
| 10 | `floor_hazard_stripe` | Worn amber/black safety paint |
| 11 | `floor_collapsed_void` | Torn plate edge around impassable darkness |
| 12 | `wall_isolated` | Broken low factory wall, no connection |
| 13 | `wall_end_n` | Wall connected north only |
| 14 | `wall_end_e` | Wall connected east only |
| 15 | `wall_end_s` | Wall connected south only |
| 16 | `wall_end_w` | Wall connected west only |
| 17 | `wall_straight_ns` | North/south wall |
| 18 | `wall_straight_ew` | East/west wall |
| 19 | `wall_corner_ne` | North/east wall corner |
| 20 | `wall_corner_es` | East/south wall corner |
| 21 | `wall_corner_sw` | South/west wall corner |
| 22 | `wall_corner_wn` | West/north wall corner |
| 23 | `wall_t_nes` | T-junction open north/east/south |
| 24 | `wall_t_esw` | T-junction open east/south/west |
| 25 | `wall_t_swn` | T-junction open south/west/north |
| 26 | `wall_t_wne` | T-junction open west/north/east |
| 27 | `wall_cross` | Four-way wall junction |
| 28 | `route_straight_ns` | Reinforced enemy lane north/south |
| 29 | `route_straight_ew` | Reinforced enemy lane east/west |
| 30 | `route_corner_ne` | North/east lane corner |
| 31 | `route_corner_es` | East/south lane corner |
| 32 | `route_corner_sw` | South/west lane corner |
| 33 | `route_corner_wn` | West/north lane corner |
| 34 | `route_t_nes` | T-lane north/east/south |
| 35 | `route_t_esw` | T-lane east/south/west |
| 36 | `route_t_swn` | T-lane south/west/north |
| 37 | `route_t_wne` | T-lane west/north/east |
| 38 | `route_cross` | Four-way lane junction |
| 39 | `route_broken` | Ruined lane with temporary repair plates |
| 40 | `decal_rubble_light` | Small nuts, plate chips, and concrete fragments |
| 41 | `decal_rubble_medium` | Hand-sized scrap and broken conduit |
| 42 | `decal_rubble_heavy` | Collapsed machinery pieces; does not resemble a pad |
| 43 | `decal_broken_pipes` | Snapped pipe cluster with rusted ends |
| 44 | `decal_loose_cables` | Dead black cables, no friendly glow |
| 45 | `decal_scrap_pile` | Gears, sheet metal, and machine parts |
| 46 | `decal_water_leak` | Dirty leak and mineral residue |
| 47 | `decal_scorch_marks` | Directionless blast and weld burns |
| 48 | `decal_moss` | Sparse damp growth reclaiming the floor |
| 49 | `decal_collapsed_beam` | Short structural beam, contained to one cell |
| 50 | `decal_repair_scaffold` | Amber brace, welding leads, and fresh plate |
| 51 | `decal_power_cable` | New friendly cable with restrained green nodes |
| 52 | `section_scrap_processing` | Conveyor teeth and scrap-channel floor motif |
| 53 | `section_water_filtration` | Pipe manifold and drain motif |
| 54 | `section_power_spine` | Heavy bus bars and ceramic insulators |
| 55 | `section_assembly_hall` | Floor rails and robot work-cell markings |
| 56 | `section_logistics_hub` | Cargo track and loading-zone markings |
| 57 | `section_hydroponics` | Broken planter channels with surviving green traces |
| 58 | `section_robotics_bay` | Tool sockets and manipulator anchor points |
| 59 | `section_research_core` | Shielded cable floor and data-jack pattern |
| 60 | `section_ai_vault` | Black composite plates with hostile red corruption |
| 61 | `section_factory_heart` | Dense old conduits with reclaimed green power |
| 62 | `breach_sealed_floor` | Buckled but closed industrial blast hatch |
| 63 | `breach_open_floor` | Torn hatch opening with enemy-red depth light |

## 2. Tower pads and breach markers

Deliver `assets/map/tower_pads.png` at **256 x 64 px**, four horizontal **64 x 64
px frames**, transparent background `#00000000`.

| Frame | Asset | Required visual |
| ---: | --- | --- |
| 0 | `pad_debris` | Mounting ring buried under scrap and crossed beams |
| 1 | `pad_cleared` | Exposed dead socket, scraped clean but unpowered |
| 2 | `pad_powered_empty` | Repaired socket with subtle green contacts |
| 3 | `pad_powered_occupied` | Clamped powered base visible beneath a tower |

Deliver `assets/map/breach_markers.png` at **288 x 96 px**, three horizontal **96 x
96 px frames**, transparent background `#00000000`.

| Frame | Asset | Required visual |
| ---: | --- | --- |
| 0 | `breach_locked_preview` | Strained closed hatch, dim red warning seam |
| 1 | `breach_active` | Torn gate, smoke-dark interior, strong red core |
| 2 | `breach_disabled` | Collapsed/foamed gate with dead warning lamps |

## 3. Repairable factory machines

Each machine file contains three horizontal states: **broken**, **repaired**, and
**powered**. Each state is **128 x 96 px**, making every sheet **384 x 96 px**.
Background is transparent `#00000000`. The powered frame must retain all rust and
repair patches from the repaired frame, adding only active mechanisms and restrained
green/cyan light.

| File | Machine design |
| --- | --- |
| `assets/buildings/scrap_converter.png` | Crusher jaws, magnetic separator, scrap output chute |
| `assets/buildings/water_reclaimer.png` | Filter drums, corroded tanks, patched blue-grey pipes |
| `assets/buildings/nutrient_vats.png` | Sealed growth vats, pumps, damaged grow lamps |
| `assets/buildings/power_turbine.png` | Heavy turbine housing, exposed rotor, new bus cables |
| `assets/buildings/power_distribution_spine.png` | Transformer stacks, breakers, ceramic insulators |
| `assets/buildings/assembly_hall_core.png` | Central conveyor nexus and dormant assembly arms |
| `assets/buildings/logistics_hub_core.png` | Cargo carousel, lift mechanism, routing scanner |
| `assets/buildings/robotics_bay_core.png` | Repaired industrial robot arms around a tool cradle |
| `assets/buildings/research_core.png` | Shielded server cabinets and damaged analysis chamber |

Deliver `assets/buildings/factory_core.png` at **384 x 128 px**, three horizontal
**128 x 128 px frames**, transparent background `#00000000`:

| Frame | Asset | Required visual |
| ---: | --- | --- |
| 0 | `factory_core_dormant` | Ancient central control engine, cold and corroded |
| 1 | `factory_core_repaired` | Fresh plates, welds, amber braces, no power glow |
| 2 | `factory_core_powered` | Rotating/energised heart with friendly green core |

## 4. Defense towers

Deliver `assets/towers/towers.png` at **320 x 192 px**, a grid of five columns and
three rows. Every frame is **64 x 64 px** on transparent `#00000000`. Columns are the
tower types below; rows are levels 1, 2, and 3. Higher levels add recognisable hardware,
not just more glow. Keep the mounting point at pixel `(32, 42)` in every frame.

| Column | Tower | Level progression and silhouette |
| ---: | --- | --- |
| 0 | Ballistic Turret | Single salvaged autocannon; twin feed; armoured rotary cannon |
| 1 | Laser Emitter | Exposed focusing lens; cooled dual lens; shielded beam projector |
| 2 | EMP Pylon | Coil mast; double capacitor cage; large forked discharge crown |
| 3 | Flak Field | Scrap mortar; multi-tube launcher; armoured radial shrapnel array |
| 4 | Signal Jammer | Patched dish; dual antenna cage; rotating corrupted-signal crown |

Tower equipment belongs to the repaired factory: worn steel, visible welds, replacement
plates, amber clamps, and friendly green/cyan power. Preserve the existing gameplay accent
identity: ballistic amber, laser red-white, EMP blue, flak yellow, jammer violet. Laser red
must read as a focused weapon colour, not the diffuse hostile red used by enemies.

## 5. Enemy animation sheets

Each enemy is a four-frame horizontal movement loop: **contact**, **compression**,
**passing**, **recovery**. The game may rotate the sheet toward movement direction, so
draw every unit facing right. Use transparent `#00000000`; keep feet/wheels aligned to a
common ground point. Hit flash, dodge transparency, health bars, slow effects, and death
particles remain code-driven and must not be baked in.

| File | Sheet / frame size | Required enemy design |
| --- | --- | --- |
| `assets/enemies/scout_drone.png` | **192 x 48 px** / 48 x 48 | Fast spider-like reconnaissance robot built from a camera pod, four thin mechanical legs, one scavenged organic optic, and a small red sensor. Narrow, agile silhouette. |
| `assets/enemies/assault_drone.png` | **192 x 48 px** / 48 x 48 | Low tracked industrial loader converted into an assault robot, with welded armour, one cyborg torso/arm remnant, exposed ammo feed, and red targeting lamps. Medium silhouette. |
| `assets/enemies/siege_walker.png` | **256 x 64 px** / 64 x 64 | Slow four-legged foundry walker: thick hydraulic legs, furnace-like armoured body, grafted muscle bundles driving pistons, and a heavy breaching ram. Broadest non-boss silhouette. |
| `assets/enemies/infiltrator.png` | **192 x 48 px** / 48 x 48 | Lean humanoid cyborg made from maintenance-robot limbs and a wrapped organic spine, with cable tendrils, cutting tools, and a dim violet-red sensor cluster. Must read as evasive rather than heavily armoured. |
| `assets/enemies/command_node.png` | **256 x 64 px** / 64 x 64 | Floating or multi-legged command machine surrounding a preserved organic brain case, antenna crown, escort-control transmitters, layered armour, and a hot red central eye. Tall/commanding silhouette distinct from the siege walker. |

Do not give all enemies humanoid bodies. Across the set there must be one insectoid unit,
one tracked unit, one quadruped industrial unit, one humanoid cyborg, and one alien-looking
command machine. Shared exposed cables and enemy-red control lights unify the faction.

## 6. Implementation acceptance checklist

- [ ] All 64 tile-atlas cells are present and mapped by name.
- [ ] Random floor variants do not create obvious repeated seams at map scale.
- [ ] Every one of the 10 sections is identifiable through machinery and floor motifs,
  while still reading as one contiguous ruined factory.
- [ ] Broken, repaired, and powered machine states are distinguishable by silhouette at
  normal zoom and do not rely on colour alone.
- [ ] All 5 towers are distinguishable in grayscale and at level 1.
- [ ] Tower level changes are visible without selection rings.
- [ ] All 5 enemies are distinguishable in grayscale at their normal rendered size.
- [ ] Friendly green/cyan power and hostile red/orange light are never ambiguous.
- [ ] No sprite obscures route arrows, selection feedback, or health bars.
- [ ] Transparent edges have no white/coloured matte fringe.
- [ ] Assets render with nearest-neighbour filtering and no texture bleeding.
- [ ] The title-screen palette and gameplay-map palette feel like the same factory world.
- [ ] Verification captures replace the current gameplay image in `docs/verification/`
  once the art is integrated.

## Delivery total

- **1** factory tile atlas containing **64** named cells.
- **1** tower-pad sheet containing **4** states.
- **1** breach-marker sheet containing **3** states.
- **9** repairable-machine sheets containing **27** machine-state frames.
- **1** factory-core sheet containing **3** states.
- **1** tower atlas containing **15** tower/level frames.
- **5** enemy sheets containing **20** movement frames.
- **19 PNG files**, **136 required frames/cells** in total.
