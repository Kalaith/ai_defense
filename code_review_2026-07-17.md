# Code Review — `ai_defense` (Last Assembly)

**Date:** 2026-07-17
**Reviewed against:** `CODE_STANDARDS.md` (verified byte-identical to canonical `RustGames/docs/CODE_STANDARDS.md`)
**Commit:** `58569af`
**Scope:** 53 `.rs` files, 12,650 lines; `assets/*.json`; `index.html`; `publish.ps1`

---

## Verdict

This is a **healthy, well-disciplined codebase**. The automated gates are green (`cargo clippy -D warnings` clean, `cargo fmt --check` clean, 13/13 tests passing), no file breaches the 800-line hard limit, deployment is fully compliant, and error handling is the strongest area in the project.

The findings cluster into **three coherent themes**, not scattered rot:

1. **Debris from one unfinished `mod.rs` → `_impl.rs` refactor** — ~681 dead lines (5.4% of the tree) that actively mislead readers and future audits.
2. **The `UiAction` pattern (§7.2) was never built** — the view layer mutates state inline, and a module doc claims the opposite.
3. **The JSON migration stopped before the beacon/threat subsystem** — leaving balance dials in Rust, one of them triplicated.

Everything below was verified against the actual source. Where a reviewer's claim did not survive verification, it is recorded in [Claims That Did Not Survive](#appendix-b--claims-that-did-not-survive-verification) rather than reported as a finding.

### Compliance at a glance

| Standard | Status |
|---|---|
| §2.2 File size (800 hard limit) | ✅ Pass — max 786 |
| §6 Error handling | ✅ Pass — 0 `unwrap`/`panic!` in non-test code |
| §8 Deployment / WebGL | ✅ Pass — all clauses |
| §10.1 rustfmt / §10.2 clippy | ✅ Pass (but every `#[allow]` undocumented) |
| §1.4 `_`-prefixed struct fields | ✅ Pass — zero |
| §7.4 Toolkit usage | ✅ Strong — 40+ call sites |
| **§1.4 No unused code** | ❌ **~681 dead lines** |
| **§7.1/7.2 UI is dumb / Action pattern** | ❌ **No `UiAction` exists; 19 mutating draws** |
| **§1.3/5.3 Data-driven design** | ⚠️ Beacon/threat dials hardcoded |
| **§4.1 Function size (100 max)** | ⚠️ 15 functions over |
| **§4.3 Argument count (≤3)** | ⚠️ Three 13–14 param functions |
| §9.2 Module docs | ⚠️ 6 of 53 missing |
| §11 Testing | ⚠️ `engine/` largely untested |

---

## ✅ DONE — Severity 1 — `src/state/gameplay.rs` is 323 lines of unreachable code

**§1.4, §2.3** · **Fix: delete the file**

Proven, not inferred: rustc's own dep-info (`.cargo-target/debug/deps/ai_defense.d`) lists 52 compiled files against 53 on disk. The one file the compiler never reads is `src/state/gameplay.rs`.

The mechanism is at `src/state/mod.rs:3`:

```rust
#[path = "gameplay/mod.rs"]
pub mod gameplay;
```

That `#[path]` points past `gameplay.rs` directly at `gameplay/mod.rs`, so `gameplay.rs` is never named by any `mod` declaration in the crate. It is tracked in git and looks live to every reader and every grep.

Its 323 lines are dead in two layers:
- `gameplay.rs:3-6` — a shim that would be redundant even if reachable (`state/mod.rs` already does exactly this).
- `gameplay.rs:8-323` — `#[cfg(FALSE)] mod legacy { … }`, a 315-line stale copy of `GameplayState`, `Resources`, `new()`, `from_save()`, `apply_save()`, `build_save_data()`. `cfg(FALSE)` never compiles even on a reachable path.

This also violates §2.3 ("Do not keep both `foo.rs` and `foo/mod.rs`; Rust treats that as an ambiguous module source"). Only the explicit `#[path]` disambiguates it — which is the workaround, not the fix.

## ✅ DONE — Severity 2 — `src/engine/wave.rs`: 358 of 363 lines are commented-out legacy

**§1.4** · **Fix: delete lines 6–363**

Same refactor debris, different mechanism. The file *is* compiled, but only lines 1–5 are live:

```rust
#[path = "wave_impl.rs"]
mod wave_impl;
#[allow(unused_imports)]
pub use wave_impl::*;
```

Lines 6–363 are a `/* … */` block wrapping a `#[cfg(FALSE)] mod legacy` — dead twice over. The commented text is also **corrupt**: `preview_wave` (`wave.rs:52-106`) has an entire `generate_wave` body spliced into it, and `impl WaveManager` (`:116`) opens with orphaned struct-field fragments. It has no value as reference material.

### Why Severity 1 and 2 matter more than "dead weight"

These two files hold the **pre-migration hardcoded constants**. Any grep-driven review — human or agent — surfaces phantom findings from them. During this very review, two independent reviewers had to spend effort ruling out magic numbers at `wave.rs:301`, `:297`, and `:313-315` that live inside a comment block. The live equivalents at `wave_impl.rs:325-328` correctly read from JSON tuning. **This debris has already cost real review time and will keep doing so until deleted.**

## Severity 3 — The `UiAction` pattern (§7.2) does not exist

**§7.1, §7.2** · **Fix: introduce `UiAction`, or correct the docs to match reality**

A crate-wide search for an action/intent enum returns exactly one hit — `SettingsAction` (`src/ui/settings.rs:10`, only `None`/`Close`). **There is no `UiAction` type.** The standard's §7.2 is unimplemented.

Instead, **19 `draw*` functions take `&mut self`** and call business logic inline:

| Location | Mutation inside a draw |
|---|---|
| `render_hud/beacon_panel.rs:143` | `self.start_beacon();` |
| `render_hud/beacon_panel.rs:195` | `self.trigger_shutdown();` |
| `render_hud/beacon_panel.rs:160-161` | `self.scavenger_recall_active = true;` |
| `render_panels/context_slot.rs:209` | `self.try_place_tower_on_slot(idx, &def_id, data)` |
| `render_panels/context_building.rs:101-102` | `self.repair_building(idx)` / `self.power_building(idx)` |
| `render_panels/context_factory.rs:44` | `self.purchase_upgrade(&id)` |
| `ui/panels.rs:40` | `self.placing_tower = Some(tower_id);` |
| `ui/overlays.rs:49` | `self.show_intro = false;` |

**The most important detail:** `src/state/gameplay/ui.rs:5-6` documents an invariant the code breaks —

> *"Panels never mutate state directly — they resolve a click to an intent and dispatch it to an `actions` method."*

`ui/actions.rs` is not a dispatcher; it is a bag of `pub(crate)` mutation methods panels call directly. The dispatch half of that sentence is true; the "never mutate directly" half is false (`panels.rs:40`, `overlays.rs:49`). **A documented invariant that the code violates is worse than no doc** — it tells the next contributor the guarantee holds when it doesn't.

The nearest thing to an intent enum is stringly-typed: `ui/panels.rs:94` declares `Option<(usize, &'static str)>`, matched at `:222-226` on `"repair"` / `"power"`.

Related §7.1 breaches — side effects inside the render pass:
- `ui/settings.rs:18` — `draw_settings_overlay(settings: &mut Settings)` mutates *and* writes to disk (`let _ = settings.save();` at `:87-89`).
- `ui/overlays.rs:129-136` — `finish_coach()` calls `Settings::load()` then `.save()`, invoked from `draw_coach` at `:190`.
- `render.rs:50` — `self.handle_map_click(data);` is the last line of `draw()`; it reads the mouse and can spend scrap and build a tower.

**Note:** `src/ui/build_panel.rs:23` already demonstrates the compliant model — a free function taking values and returning `Option<String>`. `ui/panels.rs:30` calls it correctly, then breaks the pattern at `:40` by assigning the result to state inside the draw. The good pattern exists; it just isn't held.

## Severity 4 — The beacon/threat subsystem was never migrated to JSON

**§1.3, §5.3** · **Fix: lift dials into the `threat`/`evacuation` sections that already exist**

`assets/constants.json` (15 sections) is real and well-plumbed — `state/gameplay/mod.rs:229-245` builds `WaveTuning` field-by-field from it. But **one subsystem was missed**, and the JSON's own shape proves it: `evacuation` defines all four per-phase rates (`warm_rate`, `sustained_rate`, `screaming_rate`, `terminal_rate`) but **not the thresholds that select the phase**; `threat` defines every accrual rate but **no decay rates and no tier boundaries**. The sections are ready; the dials are missing.

**4a. Threat tier thresholds triplicated — the one with correctness risk.** The `25.0` / `60.0` pair is hand-re-derived in three modules with no shared source:

```rust
// engine/threat.rs:104-108        — the canonical reaction_tier()
if level < 25.0 { ReactionTier::Observation } else if level < 60.0 { … }

// engine/wave_impl.rs:337-339     — re-derives it to gate enemy unlocks
if threat_awareness >= 60.0 { max_tier.max(3) } else if threat_awareness >= 25.0 { … }

// state/gameplay/helpers.rs:35-37 — re-derives it again to pick the HUD colour
if level < 25.0 { dark::TEXT_DIM } else if level < 60.0 { dark::WARNING }
```

`wave_impl` never calls `reaction_tier()`. Changing the curve in one place **silently desyncs enemy unlocks from the colour the player sees** — a real bug waiting, not just a standards nit. `constants.threat` already holds `adaptation_threshold` and `adaptation_unlock_threshold`, so this was the intended home.

**4b. Beacon phase thresholds hardcoded** — `engine/beacon.rs:43-51`: `61.0` / `36.0` / `16.0`. Half the dial (rates) is in JSON, half (boundaries) in Rust.

**4c. The entire beacon-strength formula is hardcoded** — `update/beacon_cycle.rs:132-135`, four uncommented weights:
```rust
self.beacon_strength = (unlocked * 2.0) + (power_throughput / 10.0)
    + (ai_vault_tier * 5.0) + (population / 20.0);
```
With 4b, tuning the beacon requires a recompile *and* reasoning across two files.

**4d. Threat decay rates hardcoded** — `engine/threat.rs:160-174`: `let decay_rate = 0.5 * dt;` and the `* 0.2` territory modifier. Accrual is data-driven, decay is not — the two sides of one equilibrium live in different files. The `* 0.2` is precisely the territory-decay dial balanced against the survival-proof test, and it is currently invisible to anyone reading the JSON.

**4e. Power economy** — `engine/factory.rs:90`: `40.0 + other_unlocked as f32 * 4.0`. Primary economy dials; `constants.economy` has no generation terms.

**4f. Minor** — `update/systems.rs:311`: `food_reward *= 1.5;` (logistics hub bonus, sitting two lines below correct `constants.waves.*` usage); `engine/tower.rs:334`: `fired_count * 0.01 * dt` (sole feed into the `heat` signature).

## Severity 5 — `draw_circuit_board` is 531 lines

**§4.1** · 15 functions exceed the **absolute max of 100**:

| Lines | Location |
|---|---|
| **531** | `render_map.rs:13` `draw_circuit_board` — 5.3× the max, 68% of its file |
| 182 | `render_hud/beacon_panel.rs:17` `draw_beacon_panel` |
| **169** | `engine/tower.rs:174` `tick_towers` — the only non-UI one; also untested |
| 164 | `ui/build_panel.rs:23` `draw_build_panel` |
| 161 | `ui/panels.rs:70` `draw_sector_panel` |
| 156 | `state/gameplay/mod.rs:193` `new` |
| 150 | `ui_advice/suggest.rs:15` `suggest_next_action` |
| 140 | `ui/icons.rs:38` `draw_icon` |
| 136 | `context_slot.rs:17`, `build_panel.rs:223`, `widgets.rs:62` |
| 119 | `wave_impl.rs:302`, `gameplay/mod.rs:356` |
| 112 | `ui_advice/alerts.rs:33` |
| 109 | `ui/placement.rs:12` |

`draw_circuit_board` (lines 13–543) draws paths, slots, buildings, traces, and hover state in one body — the clearest single refactor target in the repo. A further 34 functions sit in the 50–100 range (legal, over the 20–50 target); most are rendering, where long straight-line drawing is somewhat idiomatic.

Note `render_map.rs` at 786 lines is **under** the 800 hard limit but close enough to watch — extracting `draw_circuit_board` resolves both concerns at once.

## Severity 6 — 14-parameter functions, against a "prefer ≤ 3" rule

**§4.3** · `src/engine/wave_impl.rs` is the outlier:

- `wave_impl.rs:243` `preview_wave_entries` — **14 params**
- `wave_impl.rs:302` `build_spawn_queue` — **14 params**
- `wave_impl.rs:207` `preview_wave` — **13 params**
- `wave_impl.rs:92` `generate_wave` — 9 · `engine/tower.rs:124` `Tower::new` — 9 · `tower.rs:174` `tick_towers` — 8

**The fix already exists in the file.** `WaveTuning` is defined at `wave_impl.rs:32` and `WaveManager::new(tuning: WaveTuning)` takes it (`:72`). These three *free* functions instead explode the same tuning block into loose params — passing `wave_budget_base`, `wave_budget_per_wave`, `wave_commander_every`, `threat_budget_divisor`, `threat_health_mult_per_awareness` by hand at every call site. Threading `&WaveTuning` through is a mechanical change against a struct that is already there.

22 more functions take 5–7 params, mostly UI primitives where `(x, y, w, h, …)` geometry is defensible — treat those as idiom, not findings.

## Severity 7 — `#[allow]`s masking genuinely dead code

**§1.4, §10.2** · Verified by stripping the allows in a scratch copy and rebuilding. Warnings that fired:

```
warning: field `id` is never read              --> src\engine\map.rs:148:9
warning: function `preview_wave` is never used --> src\engine\wave_impl.rs:207:8
warning: variant `Info` is never constructed   --> src\state\gameplay\ui_advice.rs:58:5
```

- **`map.rs:146`** — `#[allow(dead_code)]` on `SectionRenderInfo` masks one dead field, `id`. §1.4 is explicit: *"If a field is unused, delete it — don't mark it as unused."* A struct-level allow is the same evasion as an `_` prefix, only broader — it will swallow every future dead field silently.
- **`ui_advice.rs:56`** — `AlertSeverity::Info` is **never constructed**. It appears only in match arms (`render_hud.rs:388`, `alerts.rs:186`), so it reads as live under grep but isn't. Every construction site uses `Warning`/`Critical`.
- **`wave_impl.rs:206`** — `preview_wave` is dead in the game binary; its only caller is a test (`:452`). The game uses `preview_wave_entries`. Correct fix is `#[cfg(test)]`, not `dead_code`.

**Stale allows suppressing nothing** (removing them produced zero warnings): `map.rs:158` on `MapSection`, `wave.rs:3` on `pub use wave_impl::*`.

**All 13 `#[allow]` sites are undocumented**, against §10.2's *"Document any `#[allow]` with a comment"*: `map.rs:146,158`; `tower.rs:123,173`; `wave.rs:3`; `wave_impl.rs:91,205,206,242,301`; `ui_advice.rs:56`; `state/mod.rs:14,40`. The five `clippy::too_many_arguments` sites are defensible on merit but point straight at Severity 6 — they suppress the exact lint that would have flagged it.

## Severity 8 — `engine/` calculations are almost entirely untested

**§11.1** · 13 tests exist and all pass. But §11.1 names "core game calculations" as the priority, and **every `engine/` module except `wave_impl.rs` has zero tests**:

| Module | Lines | Tests | Untested |
|---|---|---|---|
| `engine/map.rs` | 727 | **0** | `from_def`, `build_sections`, `nearest_slot`, pathing |
| `engine/tower.rs` | 342 | **0** | `tick_towers` — 169 lines of combat/damage/scrap math |
| `engine/threat.rs` | 174 | **0** | `dominant()`, `tick_decay` — the whole adaptation system |
| `engine/beacon.rs` | 53 | 0 | beacon phase state machine |
| `engine/factory.rs` | 165 | 0 | sector integrity/power |
| `engine/population.rs` | 70 | 0 | food/population loop |
| `engine/enemy.rs` | 121 | 0 | movement, path following |
| `save/mod.rs` | 168 | **0** | save/load round-trip — the only fallible I/O in the codebase |

What exists is good: `data/mod.rs:461` and `data/strings.rs:613` cover JSON loading, `wave_impl.rs` has 4 solid tests, and `update/survival_proof.rs:141 scripted_defense_can_survive_ten_waves` is a strong integration guard.

The sharpest gaps are **`tick_towers`** and **`threat.rs`** — they carry the game's balance math with no tests, and (per Severity 4a) `threat.rs`'s thresholds are simultaneously triplicated. A test pinning `reaction_tier()` boundaries would catch exactly the desync 4a describes. **`save/mod.rs` round-trip** is the other priority: it is the only genuinely fallible I/O here, and a save-corruption bug costs a player their campaign.

## Severity 9 — Duplication and smaller items

**9a. §7.4 — `draw_bounded_text`/`truncate_text` exists in three local copies plus the toolkit.**
- `src/ui/text.rs:7,13` — the canonical local pair
- `src/state/menu.rs:263,268` — **byte-for-byte identical** private copy
- `src/state/gameplay/render_hud.rs:519` — third copy, helper renamed `truncate_to_width`
- `macroquad-toolkit/src/ui/font.rs:306` — `truncate_text_to_width` / `truncate_text_to_width_ex` already ship this

One behavior, four implementations. Collapse onto the toolkit per §7.4.

**9b. §4.4 — Tuple returns.** `engine/map.rs:237`:
```rust
fn build_sections(sections: &[SectionDef])
    -> (Vec<MapSection>, HashMap<String, usize>, HashMap<String, usize>)
```
Two structurally identical `HashMap<String, usize>` returned unnamed and adjacent — **trivially swappable at the call site**. This is the one tuple return with real bug risk; it wants a named `MapSections` struct. Also `ui/panels.rs:233` `-> (String, &'static str, ConsoleButtonState)`, where the doc comment ("Label, intent, and button state") does the work field names should. The `Option<(usize, f32)>` trio (`map.rs:548`, `:564`, `helpers.rs:128`) is mild, well-worn idiom — not worth changing.

**9c. §2.1 — `data/` knows about `engine/`.** `data/mod.rs:6-7` imports `crate::engine::enemy::EnemyType` and `crate::engine::tower::TowerType`, against *"Data module has no knowledge of engine or UI."* Both are plain serde enums that belong in `data/`; the dependency points the wrong way. This is the **only** cross-domain import violation in the crate and is mechanically fixable.

**9d. §9.2 — 6 of 53 modules missing `//!` docs:** `main.rs`, `render_map.rs` (the largest file), `render_hud.rs`, `render.rs`, `helpers.rs`, `wave.rs` (moot once deleted). 47/53 comply — good discipline with a gap concentrated in the render layer. `helpers.rs` most needs one: "helpers" names nothing.

**9e. §10.3 — Shadowing.** `ui/build_panel.rs` re-declares `let cy` six times in the same scope inside `draw_map_key` (lines 260, 283, 291, 315, 335, 342) — §10.3 names this explicitly. Low real risk (each legend row is sequential and self-contained), and it's a symptom of the 136-line function: extracting a `draw_key_row` helper fixes both at once.

**9f. §7.4 — Button semantics.** `ui/widgets.rs:196` calls `is_mouse_button_pressed` directly rather than `toolkit::input::was_clicked`, so `draw_console_button` fires on **press** where §7.4's standard button fires on **release** (no cancel-by-dragging-off). Deliberate for some controls, but worth an explicit choice.

**9g. §6 — Loader policy is split and undocumented.** `data/loader.rs` treats its seven assets two ways: `load_tower_defs`/`load_enemy_defs`/`load_sector_data` degrade gracefully with `eprintln!` + empty default (matching §6.2 exactly), while `load_constants` (`:15`) and `load_map_def` (`:74`) `expect()`. That split is likely deliberate — no map means the game genuinely can't run, no towers is merely broken — but nothing says so. One comment settles it.

---

## Where the codebase is genuinely strong

Worth stating plainly, because these are the areas that usually fail:

- **Player-facing text is fully externalized.** Only 3 `draw_text` call sites exist crate-wide and **none passes a literal**. `data/strings.rs` implements a proper table — `include_str!` + `OnceLock`, disk-read on native for rebuild-free writing passes, embedded fallback for WASM, and a `fill()` template expander. 328 entries across 23 categories. Scale of problem: zero.
- **Error handling (§6) is the strongest area.** Zero `unwrap()`, zero `panic!`, zero `todo!`/`unreachable!` in non-test code. Only 3 `expect()` calls, all on `include_str!`-embedded JSON compiled into the binary — no runtime path exists where the data is missing, so a malformed asset is a build-time authoring error that fails deterministically on first launch. That is much closer to §6.1's "truly unrecoverable state" than to "missing entity". Lookups consistently use `.get()`/`Option`; slice indexing is guarded.
- **`_`-prefixed struct fields: zero.** The rule §1.4 calls out most sharply is being followed (the evasion here took the form of `dead_code` allows instead).
- **Deployment (§8): fully compliant.** `publish.ps1`, `index.html`, `catalog_thumbnail.png` all present. `index.html` satisfies every §8.4 clause — `mq_js_bundle.js` (`:91`), `load("ai_defense.wasm")` (`:134`), `canvas id="glcanvas"` (`:31`), `image-rendering: pixelated` (`:12`).
- **Toolkit adoption (§7.4) is strong and defensible.** 40+ call sites across 20 files: `colors::dark` (17×), `ui::draw_ui_text`/`measure_ui_text`, `math::pulse_range`, `camera`, `rng`, `persistence`, `timing::Cooldown`, `notifications`, `capture`. `draw_resource_bar` (`widgets.rs:19`) correctly delegates straight to `toolkit::ui::progress_bar`. `draw_console_button` and `icons.rs` are genuinely game-specific (CRT scanlines, corner brackets, boon glyphs) — fine hand-rolled.
- **`engine/` isolation is clean.** `engine/` imports nothing from `crate::state` or `crate::ui`. Every `&mut self` there is on a struct owning its own data; none takes `&mut GameplayState`. `map.rs:618 set_slot_state(&mut self, …) -> Vec<String>` returns results rather than reaching outward.
- **`render_map.rs` (786 lines) and `ui/placement.rs` are fully `&self`** — correct render layers, no mutation.
- **The standards doc is in sync** with canonical `docs/CODE_STANDARDS.md`.

---

## Recommended order of work

Sequenced so each step makes the next cheaper:

1. **Delete `src/state/gameplay.rs` and `src/engine/wave.rs:6-363`.** Zero risk (one is uncompiled, the other is inside a comment), removes 681 misleading lines, and stops phantom findings from polluting every future review. **Do this first** — it makes steps 2–4 easier to reason about.
2. **Sweep the masked dead code:** delete `SectionRenderInfo.id` and `AlertSeverity::Info`; re-gate `preview_wave` as `#[cfg(test)]`; drop the stale allows (`map.rs:158`, `wave.rs:3`); document the rest.
3. **Fix the threat-threshold triplication (4a)** — have `wave_impl` and `helpers` call `reaction_tier()`, and lift `25.0`/`60.0` into `constants.threat`. This is the only finding with a live correctness risk; pair it with a boundary test (Severity 8).
4. **Finish the JSON migration** for beacon/threat (4b–4f) into the `threat`/`evacuation` sections that already exist.
5. **Extract `draw_circuit_board`** (531 → several cohesive draws), which also pulls `render_map.rs` well clear of the 800 limit.
6. **Thread `&WaveTuning`** through the three free functions in `wave_impl.rs`, then delete the `too_many_arguments` allows it makes redundant.
7. **Decide on §7.2 explicitly.** Either introduce a real `UiAction` enum and a dispatcher, or — if the `&mut self` view layer is a deliberate, accepted divergence — **correct `ui.rs:5-6` so it stops asserting an invariant the code doesn't hold.** Either is defensible; the current state, where the doc contradicts the code, is not. This is the largest change here and is best planned separately.
8. **Add tests** for `tick_towers`, `threat.rs` boundaries, and `save/mod.rs` round-trip.
9. **Housekeeping:** collapse the three `truncate_text` copies onto the toolkit; name `map.rs:237`'s return struct; move `EnemyType`/`TowerType` into `data/`; add the 6 missing `//!` docs.

Items 1, 2, and 9 are mechanical and low-risk. Item 3 is the one that fixes a latent bug.

---

## Appendix A — How this review was verified

Findings were produced by four independent reviewers (data-driven design; architecture layering; functions/errors/docs/tests; dead code/toolkit) and then **re-verified line-by-line against the source before inclusion**. Reachability was proven with rustc dep-info; dead code was proven by stripping allows and rebuilding in a scratch copy; function lengths were measured with a brace-matching script, not eyeballed. Nothing in the working tree was modified.

## Appendix B — Claims that did not survive verification

Recorded so they are not re-litigated later:

- **"The local `truncate_text` copies are subtly worse — they hardcode `None` for the font, so they measure against a different font than they render with (latent mis-truncation bug)."** **False.** `macroquad-toolkit/src/ui/font.rs:102` does `measure_text(text, font.or(default_ui_font()), …)` — passing `None` falls back to the same default font `draw_ui_text` uses (`font.rs:117-119`). Measure and draw agree. The duplication (9a) is real; the bug is not.
- **"A `WaveTuning` struct is missing."** **False, and the truth is sharper.** `WaveTuning` exists at `wave_impl.rs:32` and is already used by `WaveManager::new`. The free functions simply don't take it — making Severity 6 a mechanical fix against an existing type rather than a design task.
- **Magic numbers at `wave.rs:297,301,313-315`** — inside the commented-out legacy block; the live code at `wave_impl.rs:325-328` correctly reads from JSON. Phantom findings caused by Severity 2.
- **"`engine/` services are stateful, violating §2.1."** **Not a violation.** Every `&mut self` is on a struct owning its own data; none mutates passed-in game state. The one fair nit: `engine/mod.rs:1` calls them "Stateless game logic services", which is inaccurate — they're stateful aggregates.
- **"§5.1 requires mutation via methods on `Game` in main.rs; mutation lives on `GameplayState`."** **Arguable divergence, not a defect.** `game.rs:99-110` owns the state machine and `transition()`, matching the per-state `update()`/`draw()` pattern the RustGames house guide prescribes.
- **`survival_proof.rs:53` (`let dt = 0.1;`) and `:42` (`current_wave >= 10`)** — harness parameters in a file documented as "Not reachable in normal play"; out of scope for §1.3.
