 Plan to implement                                                                                                                                         │
│                                                                                                                                                           │
│ Circuit Board Map System — Implementation Plan                                                                                                            │
│                                                                                                                                                           │
│ Goal                                                                                                                                                      │
│                                                                                                                                                           │
│ Replace the free-grid tower placement with a circuit-board-style map where:                                                                               │
│ - Fixed tower slots (40+) defined in JSON, each with debris/power states                                                                                  │
│ - Multiple enemy paths unlocked by player choice (clearing debris in key areas)                                                                           │
│ - Two-step slot unlock: clear debris (cheap) → power slot (separate cost)                                                                                 │
│ - Circuit trace visuals: glowing connections between powered slots and the factory core                                                                   │
│                                                                                                                                                           │
│ ---                                                                                                                                                       │
│ Phase 1: Data Layer                                                                                                                                       │
│                                                                                                                                                           │
│ New file: assets/map.json                                                                                                                                 │
│                                                                                                                                                           │
│ {                                                                                                                                                         │
│   "factory_core": [840, 440],                                                                                                                             │
│   "slot_interact_radius": 25.0,                                                                                                                           │
│   "slots": [                                                                                                                                              │
│     {                                                                                                                                                     │
│       "id": "slot_01",                                                                                                                                    │
│       "x": 280.0, "y": 320.0,                                                                                                                             │
│       "initial_state": "powered",                                                                                                                         │
│       "clear_cost": 0.0,                                                                                                                                  │
│       "power_cost": 0.0,                                                                                                                                  │
│       "opens_entrance": null                                                                                                                              │
│     },                                                                                                                                                    │
│     {                                                                                                                                                     │
│       "id": "slot_12",                                                                                                                                    │
│       "x": 400.0, "y": 120.0,                                                                                                                             │
│       "initial_state": "debris",                                                                                                                          │
│       "clear_cost": 15.0,                                                                                                                                 │
│       "power_cost": 25.0,                                                                                                                                 │
│       "opens_entrance": "north_breach"                                                                                                                    │
│     }                                                                                                                                                     │
│   ],                                                                                                                                                      │
│   "paths": [                                                                                                                                              │
│     {                                                                                                                                                     │
│       "id": "main_west",                                                                                                                                  │
│       "entrance": [120, 360],                                                                                                                             │
│       "waypoints": [[280,360],[280,200],[480,200],[480,520],[680,520],[680,280],[840,280]],                                                               │
│       "exit": [840, 440],                                                                                                                                 │
│       "initially_active": true,                                                                                                                           │
│       "requires_entrance": null                                                                                                                           │
│     },                                                                                                                                                    │
│     {                                                                                                                                                     │
│       "id": "north_breach",                                                                                                                               │
│       "entrance": [480, 0],                                                                                                                               │
│       "waypoints": [[480,120],[600,120],[600,280],[760,280]],                                                                                             │
│       "exit": [840, 440],                                                                                                                                 │
│       "initially_active": false,                                                                                                                          │
│       "requires_entrance": "north_breach"                                                                                                                 │
│     }                                                                                                                                                     │
│   ],                                                                                                                                                      │
│   "traces": [                                                                                                                                             │
│     { "from": "slot_01", "to": "slot_02", "via": [[280,320],[360,320]] },                                                                                 │
│     { "from": "slot_05", "to": "factory_core", "via": [[700,400],[840,440]] }                                                                             │
│   ]                                                                                                                                                       │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ Full map will have 40+ slots across the playable area, 2-4 paths, and traces connecting nearby slots.                                                     │
│                                                                                                                                                           │
│ src/data/mod.rs — New structs                                                                                                                             │
│                                                                                                                                                           │
│ #[derive(Clone, Debug, Serialize, Deserialize)]                                                                                                           │
│ pub struct MapDef {                                                                                                                                       │
│     pub factory_core: [f32; 2],                                                                                                                           │
│     pub slot_interact_radius: f32,                                                                                                                        │
│     pub slots: Vec<SlotDef>,                                                                                                                              │
│     pub paths: Vec<PathDef>,                                                                                                                              │
│     pub traces: Vec<TraceDef>,                                                                                                                            │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ #[derive(Clone, Debug, Serialize, Deserialize)]                                                                                                           │
│ pub struct SlotDef {                                                                                                                                      │
│     pub id: String,                                                                                                                                       │
│     pub x: f32, pub y: f32,                                                                                                                               │
│     pub initial_state: String,     // "debris" | "cleared" | "powered"                                                                                    │
│     pub clear_cost: f32,                                                                                                                                  │
│     pub power_cost: f32,                                                                                                                                  │
│     pub opens_entrance: Option<String>,                                                                                                                   │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ #[derive(Clone, Debug, Serialize, Deserialize)]                                                                                                           │
│ pub struct PathDef {                                                                                                                                      │
│     pub id: String,                                                                                                                                       │
│     pub entrance: [f32; 2],                                                                                                                               │
│     pub waypoints: Vec<[f32; 2]>,                                                                                                                         │
│     pub exit: [f32; 2],                                                                                                                                   │
│     pub initially_active: bool,                                                                                                                           │
│     pub requires_entrance: Option<String>,                                                                                                                │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ #[derive(Clone, Debug, Serialize, Deserialize)]                                                                                                           │
│ pub struct TraceDef {                                                                                                                                     │
│     pub from: String,                                                                                                                                     │
│     pub to: String,                                                                                                                                       │
│     pub via: Vec<[f32; 2]>,                                                                                                                               │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ Add map_def: MapDef to GameData, add load_map_def() to loader.rs.                                                                                         │
│                                                                                                                                                           │
│ Files                                                                                                                                                     │
│                                                                                                                                                           │
│ - assets/map.json — NEW                                                                                                                                   │
│ - src/data/mod.rs — add structs + map_def field on GameData                                                                                               │
│ - src/data/loader.rs — add load_map_def()                                                                                                                 │
│                                                                                                                                                           │
│ ---                                                                                                                                                       │
│ Phase 2: Runtime Map Engine                                                                                                                               │
│                                                                                                                                                           │
│ New file: src/engine/map.rs                                                                                                                               │
│                                                                                                                                                           │
│ pub enum SlotState { Debris, Cleared, Powered }                                                                                                           │
│                                                                                                                                                           │
│ pub struct TowerSlot {                                                                                                                                    │
│     pub id: String,                                                                                                                                       │
│     pub position: Vec2,                                                                                                                                   │
│     pub state: SlotState,                                                                                                                                 │
│     pub clear_cost: f32,                                                                                                                                  │
│     pub power_cost: f32,                                                                                                                                  │
│     pub opens_entrance: Option<String>,                                                                                                                   │
│     pub tower_index: Option<usize>,                                                                                                                       │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ pub struct EnemyPath {                                                                                                                                    │
│     pub id: String,                                                                                                                                       │
│     pub full_path: Vec<Vec2>,   // entrance + waypoints + exit flattened                                                                                  │
│     pub active: bool,                                                                                                                                     │
│     pub requires_entrance: Option<String>,                                                                                                                │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ pub struct CircuitMap {                                                                                                                                   │
│     pub factory_core: Vec2,                                                                                                                               │
│     pub slot_interact_radius: f32,                                                                                                                        │
│     pub slots: Vec<TowerSlot>,                                                                                                                            │
│     pub paths: Vec<EnemyPath>,                                                                                                                            │
│     pub traces: Vec<TraceDef>,                                                                                                                            │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ Methods on CircuitMap:                                                                                                                                    │
│ - from_def(def: &MapDef) -> Self                                                                                                                          │
│ - slot_by_id(&self, id) -> Option<&TowerSlot>                                                                                                             │
│ - slot_by_id_mut(&mut self, id) -> Option<&mut TowerSlot>                                                                                                 │
│ - nearest_slot(&self, pos, max_dist) -> Option<usize>                                                                                                     │
│ - active_paths(&self) -> Vec<&EnemyPath>                                                                                                                  │
│ - check_entrance_unlocks(&mut self) — activates paths whose requires_entrance matches a cleared slot's opens_entrance                                     │
│ - is_node_powered(&self, id: &str) -> bool — for trace rendering                                                                                          │
│                                                                                                                                                           │
│ GameplayState changes                                                                                                                                     │
│                                                                                                                                                           │
│ - Replace map_path: Vec<Vec2> with circuit_map: CircuitMap                                                                                                │
│ - Add selected_slot: Option<usize>                                                                                                                        │
│ - Initialize in new() from data.map_def                                                                                                                   │
│                                                                                                                                                           │
│ Files                                                                                                                                                     │
│                                                                                                                                                           │
│ - src/engine/map.rs — NEW                                                                                                                                 │
│ - src/engine/mod.rs — add pub mod map;                                                                                                                    │
│ - src/state/gameplay/mod.rs — replace map_path, add selected_slot                                                                                         │
│                                                                                                                                                           │
│ ---                                                                                                                                                       │
│ Phase 3: Multi-Path Enemy Movement                                                                                                                        │
│                                                                                                                                                           │
│ Enemy changes (src/engine/enemy.rs)                                                                                                                       │
│                                                                                                                                                           │
│ - Add path_id: String field to Enemy                                                                                                                      │
│ - Enemy::new() takes path_id parameter                                                                                                                    │
│                                                                                                                                                           │
│ Wave changes (src/engine/wave_impl.rs)                                                                                                                    │
│                                                                                                                                                           │
│ - Add path_id: String to SpawnEntry                                                                                                                       │
│ - Change WaveManager::tick(dt, path) → tick(dt, circuit_map: &CircuitMap)                                                                                 │
│   - Each enemy looks up its own path via circuit_map.paths.find(|p| p.id == enemy.path_id)                                                                │
│ - generate_wave() and build_spawn_queue() take a path_id: String and spawn_point: Vec2                                                                    │
│                                                                                                                                                           │
│ Wave generation (src/state/gameplay/update.rs)                                                                                                            │
│                                                                                                                                                           │
│ In start_wave_if_ready():                                                                                                                                 │
│ - Get active paths from circuit_map.active_paths()                                                                                                        │
│ - Budget split: each path gets budget_multiplier * (0.5 + 0.5 / num_paths) — 1 path = full, 2 = 75% each, 3 = 67% each                                    │
│ - Call append_wave() once per active path with that path's entrance as spawn point                                                                        │
│                                                                                                                                                           │
│ tick_wave() change                                                                                                                                        │
│                                                                                                                                                           │
│ - Pass &self.circuit_map instead of &self.map_path                                                                                                        │
│                                                                                                                                                           │
│ Files                                                                                                                                                     │
│                                                                                                                                                           │
│ - src/engine/enemy.rs — add path_id                                                                                                                       │
│ - src/engine/wave_impl.rs — add path_id to SpawnEntry, change tick() signature                                                                            │
│ - src/state/gameplay/update.rs — multi-path wave gen, pass circuit_map to tick                                                                            │
│                                                                                                                                                           │
│ ---                                                                                                                                                       │
│ Phase 4: Slot-Based Tower Placement                                                                                                                       │
│                                                                                                                                                           │
│ Replace free-grid placement with slot targeting.                                                                                                          │
│                                                                                                                                                           │
│ src/ui/gameplay.rs changes:                                                                                                                               │
│                                                                                                                                                           │
│ - handle_placement_click() — find nearest powered+empty slot within slot_interact_radius, place tower at slot position, set slot.tower_index              │
│ - draw_placement_ghost() — highlight nearest valid slot instead of grid-snapped position; show range circle at slot position                              │
│ - handle_selection_click() — clicking a slot with a tower selects it; clicking an empty/debris/cleared slot selects the slot for interaction              │
│ - Remove snap_to_grid() and is_on_path()                                                                                                                  │
│ - Add place_tower_on_slot() — creates Tower, pushes to self.towers, sets slot.tower_index                                                                 │
│                                                                                                                                                           │
│ Slot interaction UI                                                                                                                                       │
│                                                                                                                                                           │
│ When selected_slot is set and no tower placement active, draw context at bottom of build panel:                                                           │
│ - Debris: "Clear Debris (X scrap)" button                                                                                                                 │
│ - Cleared: "Power Slot (X scrap)" button                                                                                                                  │
│ - Powered + empty: "Ready — select tower to build"                                                                                                        │
│ - Powered + occupied: show tower info (reuse existing selected tower panel)                                                                               │
│                                                                                                                                                           │
│ Slot action methods:                                                                                                                                      │
│                                                                                                                                                           │
│ - clear_slot_debris(idx) — deduct scrap, set Cleared, call check_entrance_unlocks(), notify if new path opened                                            │
│ - power_slot(idx) — deduct scrap, set Powered                                                                                                             │
│                                                                                                                                                           │
│ Files                                                                                                                                                     │
│                                                                                                                                                           │
│ - src/ui/gameplay.rs — rewrite placement, add slot panel                                                                                                  │
│ - src/state/gameplay/helpers.rs — remove point_to_segment_dist (no longer needed for placement)                                                           │
│                                                                                                                                                           │
│ ---                                                                                                                                                       │
│ Phase 5: Circuit Board Rendering                                                                                                                          │
│                                                                                                                                                           │
│ Replace draw_grid() + draw_path() with draw_circuit_board() in render.rs.                                                                                 │
│                                                                                                                                                           │
│ Draw order:                                                                                                                                               │
│                                                                                                                                                           │
│ 1. Traces — lines between connected slots. Powered = bright green glow + pulse. Unpowered = dim/dark.                                                     │
│ 2. Enemy paths — active paths drawn as brown/orange lines. Inactive paths faintly visible.                                                                │
│ 3. Tower slots — square pads:                                                                                                                             │
│   - Debris: brown/dark with "X" marker                                                                                                                    │
│   - Cleared: dim blue outline                                                                                                                             │
│   - Powered: green border, dark fill, slight glow                                                                                                         │
│ 4. Factory core — pulsing blue circle at center                                                                                                           │
│ 5. Entrance markers — red circles at active path entrances                                                                                                │
│                                                                                                                                                           │
│ Glow effect: draw wider transparent line behind bright traces.                                                                                            │
│                                                                                                                                                           │
│ Files                                                                                                                                                     │
│                                                                                                                                                           │
│ - src/state/gameplay/render.rs — replace draw_grid/draw_path with draw_circuit_board                                                                      │
│                                                                                                                                                           │
│ ---                                                                                                                                                       │
│ Phase 6: Save System                                                                                                                                      │
│                                                                                                                                                           │
│ src/save/mod.rs changes:                                                                                                                                  │
│                                                                                                                                                           │
│ Add SavedSlot:                                                                                                                                            │
│ pub struct SavedSlot {                                                                                                                                    │
│     pub id: String,                                                                                                                                       │
│     pub state: String,  // "debris" | "cleared" | "powered"                                                                                               │
│ }                                                                                                                                                         │
│                                                                                                                                                           │
│ Add to SaveData:                                                                                                                                          │
│ - #[serde(default)] pub slots: Vec<SavedSlot>                                                                                                             │
│ - Bump version to 2                                                                                                                                       │
│ - Keep towers field (tower positions now derived from slot positions but still stored independently)                                                      │
│                                                                                                                                                           │
│ build_save_data():                                                                                                                                        │
│                                                                                                                                                           │
│ - Serialize slot states alongside tower data                                                                                                              │
│                                                                                                                                                           │
│ apply_save():                                                                                                                                             │
│                                                                                                                                                           │
│ - Restore slot states from save                                                                                                                           │
│ - Rebuild tower→slot linkage                                                                                                                              │
│ - Call check_entrance_unlocks() to restore active paths                                                                                                   │
│ - v1 saves: slots missing → use defaults from map.json                                                                                                    │
│                                                                                                                                                           │
│ Files                                                                                                                                                     │
│                                                                                                                                                           │
│ - src/save/mod.rs — add SavedSlot, slots field                                                                                                            │
│ - src/state/gameplay/mod.rs — update build_save_data, apply_save                                                                                          │
│                                                                                                                                                           │
│ ---                                                                                                                                                       │
│ Map Design Notes                                                                                                                                          │
│                                                                                                                                                           │
│ The full map.json will need ~40-50 slots laid out around the existing path structure:                                                                     │
│ - ~8-10 slots start powered (around the main path)                                                                                                        │
│ - ~15-20 slots start as debris (expansion opportunities)                                                                                                  │
│ - ~15-20 slots start cleared but unpowered (cheaper to activate)                                                                                          │
│ - 2-3 additional paths that become active when specific debris slots are cleared                                                                          │
│ - Traces connect nearby slots forming the circuit board visual                                                                                            │
│                                                                                                                                                           │
│ I'll design the actual slot positions to make strategic sense with the existing map dimensions (playable area roughly 200-1000px wide, 35-600px tall).    │
│                                                                                                                                                           │
│ ---                                                                                                                                                       │
│ Verification                                                                                                                                              │
│                                                                                                                                                           │
│ 1. cargo build compiles clean                                                                                                                             │
│ 2. Start game — see circuit board with slots, traces, paths instead of plain grid                                                                         │
│ 3. Click debris slot → "Clear Debris" button → slot becomes cleared                                                                                       │
│ 4. Click cleared slot → "Power Slot" button → slot becomes powered (green)                                                                                │
│ 5. Select tower from build panel → click powered empty slot → tower placed                                                                                │
│ 6. Clear a slot with opens_entrance → new path activates, enemies spawn from new direction                                                                │
│ 7. Waves split enemies across active paths                                                                                                                │
│ 8. Save and load preserves slot states                                                                                                                    │
│ 9. Traces glow between powered slots, dim between unpowered                                                                                               │
╰───────────────────────────────────────────────────────────────────