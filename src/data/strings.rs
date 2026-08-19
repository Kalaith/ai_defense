//! Player-facing text, loaded from `assets/strings.json`.
//!
//! Every string the player can read lives in that file, not in Rust. Native
//! builds read it from disk at startup so writing passes need no rebuild; WASM
//! (and any missing or malformed file) falls back to the embedded copy.
//!
//! The table is immutable and loaded once, so it is reached through [`text()`]
//! rather than threaded through every draw signature. Because the `OnceLock`
//! outlives the process, `text()` hands back `&'static` data — which is what
//! lets label accessors keep returning `&'static str`.
//!
//! Templates carry `{name}` placeholders and are expanded with [`fill`]; see
//! each field's doc comment for the placeholders it accepts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

const STRINGS_JSON: &str = macroquad_toolkit::include_json_str!("../../assets/strings.json");

static STRINGS: OnceLock<Strings> = OnceLock::new();

/// The loaded string table. Cheap after the first call.
pub fn text() -> &'static Strings {
    STRINGS.get_or_init(load)
}

fn load() -> Strings {
    if let Some(disk) = load_from_disk() {
        return disk;
    }
    serde_json::from_str(STRINGS_JSON)
        .expect("embedded strings.json must be valid; the game cannot render text without it")
}

/// Read `assets/strings.json` from disk so prose can be edited without a
/// rebuild. Any failure falls through to the embedded copy.
#[cfg(not(target_arch = "wasm32"))]
fn load_from_disk() -> Option<Strings> {
    let raw = std::fs::read_to_string("assets/strings.json").ok()?;
    match serde_json::from_str(&raw) {
        Ok(parsed) => Some(parsed),
        Err(e) => {
            eprintln!("assets/strings.json is invalid ({e}); using the embedded copy");
            None
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_from_disk() -> Option<Strings> {
    None
}

/// Substitute `{name}` placeholders in a template loaded from JSON.
///
/// `format!` only accepts compile-time literals, so runtime templates need
/// this. Callers pre-format numbers to the precision they want:
/// `fill(&t.report.scrap, &[("n", &format!("{scrap:.0}"))])`.
pub fn fill(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in args {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Strings {
    pub menu: MenuStrings,
    pub intro: IntroStrings,
    pub coach: CoachStrings,
    pub pause: PauseStrings,
    pub hud: HudStrings,
    pub status: StatusStrings,
    pub beacon: BeaconStrings,
    pub vault: VaultStrings,
    pub factory_phases: FactoryPhaseStrings,
    pub threat: ThreatStrings,
    pub enemies: EnemyStrings,
    pub wave_preview: WavePreviewStrings,
    pub invalid_pad: InvalidPadStrings,
    pub entrances: HashMap<String, String>,
    pub report: ReportStrings,
    pub results: ResultsStrings,
    pub notifications: NotificationStrings,
    pub alerts: AlertStrings,
    pub advice: AdviceStrings,
    pub workforce: WorkforceStrings,
    pub boon: BoonStrings,
    pub panels: PanelStrings,
    pub build_panel: BuildPanelStrings,
    pub map: MapStrings,
    pub settings: SettingsStrings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuStrings {
    pub continue_label: String,
    pub continue_detail: String,
    pub continue_empty_label: String,
    pub continue_empty_detail: String,
    pub new_label: String,
    pub new_detail: String,
    pub settings_label: String,
    pub settings_detail: String,
    pub exit_label: String,
    pub exit_detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntroStrings {
    pub title: String,
    pub subtitle: String,
    /// Hand-broken lines; the card draws one per row without wrapping.
    pub body: Vec<String>,
    pub button: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoachStrings {
    /// `{step}`, `{total}`.
    pub header: String,
    pub skip: String,
    pub steps: Vec<CoachStep>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoachStep {
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PauseStrings {
    pub title: String,
    pub resume: String,
    pub settings: String,
    pub abandon: String,
    pub quit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HudStrings {
    pub survival: String,
    pub pop: String,
    /// `{count}`, `{max}`.
    pub pop_value: String,
    pub food: String,
    pub water: String,
    pub morale: String,
    pub health: String,
    pub power_grid: String,
    pub gen: String,
    pub used: String,
    pub net: String,
    pub battery: String,
    /// `{n}`.
    pub offline: String,
    pub threat: String,
    /// `{label}` — the loudest awareness signature.
    pub loudest: String,
    pub beacon_offline: String,
    /// `{wave}`, `{secs}`.
    pub wave_incoming: String,
    /// `{wave}`, `{alive}`.
    pub wave_active: String,
    /// `{note}`, `{composition}`.
    pub adaptation_composition: String,
    pub next_step: String,
    pub focus: String,
    /// `{cost}`, `{risk}`.
    pub cost_risk: String,
    pub beacon_control: String,
    /// `{n}`.
    pub evac: String,
    /// `{n}`, `{pending}`.
    pub evac_pending: String,
    /// `{draw}`.
    pub next_phase: String,
    /// `{tier}`, `{pct}`.
    pub threat_readout: String,
    /// `{n}`.
    pub teams_ready: String,
    pub start_beacon: String,
    pub start_beacon_tip: String,
    pub start_beacon_tip_detail: String,
    pub recall: String,
    pub recall_tip: String,
    pub recall_tip_detail: String,
    pub shutdown: String,
    pub shutdown_locked: String,
    pub shutdown_tip: String,
    pub shutdown_tip_detail: String,
    /// `{n}`.
    pub card_ready: String,
    pub card_standby: String,
    /// `{secs}`.
    pub card_return: String,
    pub card_awaiting: String,
    pub card_at_risk: String,
    pub card_out: String,
    pub card_home: String,
    pub speed_tag: String,
    pub incoming: String,
    /// `{n}`.
    pub wave_flash: String,
    pub beacon_shutdown_flash: String,
}

/// Shared machine status vocabulary, used by the SYSTEMS panel, the map nodes,
/// and the context panels so they can never drift apart.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusStrings {
    pub locked: String,
    pub damaged: String,
    pub repaired: String,
    pub online: String,
    pub offline: String,
    pub repair: String,
    pub power: String,
    /// `{n}`.
    pub need_scrap: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeaconStrings {
    pub phases: BeaconPhaseStrings,
    /// Pushed as a notification when the beacon escalates. Warm Signal is the
    /// opening state and has no line.
    pub flavor: BeaconFlavorStrings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultStrings {
    pub title: String,
    pub ready: String,
    pub ready_detail: String,
    pub begin: String,
    pub locked: String,
    pub clear_line: String,
    pub handshake: String,
    pub severance: String,
    pub override_stage: String,
    pub complete: String,
    /// `{stage}`, `{pct}`.
    pub progress: String,
    pub alert: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeaconPhaseStrings {
    pub warm_signal: String,
    pub sustained_call: String,
    pub screaming_beacon: String,
    pub terminal_howl: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeaconFlavorStrings {
    pub sustained_call: String,
    pub screaming_beacon: String,
    pub terminal_howl: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactoryPhaseStrings {
    pub dormant_ruin: String,
    pub stirring_machine: String,
    pub functional_system: String,
    pub sentient_structure: String,
    pub transcendent_entity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreatStrings {
    pub tiers: ThreatTierStrings,
    pub signatures: ThreatSignatureStrings,
    pub adapting: AdaptingStrings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreatTierStrings {
    pub observation: String,
    pub adaptation: String,
    pub suppression: String,
    pub extermination: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreatSignatureStrings {
    pub energy: String,
    pub heat: String,
    pub data: String,
    pub corruption: String,
    pub noise: String,
    pub territory: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptingStrings {
    pub scout: String,
    pub drone: String,
    pub heavy: String,
    pub saboteur: String,
    pub commander: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyStrings {
    pub scout: String,
    pub drone: String,
    pub heavy: String,
    pub saboteur: String,
    pub commander: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WavePreviewStrings {
    pub no_preview: String,
    /// `{count}`, `{name}`.
    pub entry: String,
    pub separator: String,
}

/// Why a pad cannot take the tower being placed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvalidPadStrings {
    pub needs_power: String,
    pub occupied: String,
    pub too_expensive: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportStrings {
    /// `{cycle}`.
    pub title: String,
    pub subtitle: String,
    /// `{n}`.
    pub headline: String,
    /// `{total}`.
    pub total: String,
    /// `{n}`.
    pub waves: String,
    /// `{phase}`.
    pub peak: String,
    /// `{out}`, `{home}`, `{lost}`.
    pub teams: String,
    /// `{n}`.
    pub scrap: String,
    /// `{n}`.
    pub food: String,
    /// `{n}`.
    pub population: String,
    /// `{pct}`.
    pub escalation: String,
    pub button: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultsStrings {
    pub title_victory: String,
    pub title_survived: String,
    pub title_lost: String,
    /// `{n}`.
    pub headline: String,
    /// `{n}`.
    pub lost_inside: String,
    /// `{n}`.
    pub waves: String,
    /// `{phase}`.
    pub peak: String,
    /// `{out}`, `{home}`, `{lost}`.
    pub teams: String,
    /// `{scrap}`, `{food}`, `{pop}`.
    pub carried: String,
    /// `{n}`.
    pub machines: String,
    /// `{n}`.
    pub surviving: String,
    pub outcome_survived: String,
    pub outcome_lost: String,
    pub outcome_victory: String,
    pub button: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationStrings {
    pub repairs_locked: String,
    /// `{path}`.
    pub path_opened: String,
    /// `{name}`.
    pub needs_power_from: String,
    /// `{name}`.
    pub repaired: String,
    /// `{name}`.
    pub powered: String,
    /// `{name}`, `{role}`, `{n}`, `{signature}`.
    pub sector_awakened: String,
    pub vault_takeover_started: String,
    pub vault_upload_complete: String,
    pub beacon_up: String,
    pub beacon_down: String,
    pub field_secured: String,
    /// `{n}`.
    pub evac_milestone: String,
    pub team_lost: String,
    /// `{scrap}`, `{food}`, `{pop}`.
    pub team_returned: String,
    pub scout_breach: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertStrings {
    pub counter_build: String,
    pub power_failure: String,
    /// `{n}`.
    pub power_failure_detail: String,
    pub power_deficit: String,
    /// `{secs}`.
    pub power_deficit_detail: String,
    pub food_shortage: String,
    pub food_low: String,
    /// `{secs}` — shared by both food alerts.
    pub food_detail: String,
    pub water_shortage: String,
    pub water_low: String,
    /// `{secs}` — shared by both water alerts.
    pub water_detail: String,
    pub infiltrator: String,
    pub infiltrator_detail: String,
    pub team_at_risk: String,
    /// `{n}`.
    pub team_at_risk_detail: String,
    pub beacon_escalating: String,
    /// `{draw}`.
    pub beacon_escalating_detail: String,
    pub machine_aware: String,
    pub machine_watching: String,
    /// `{pct}` — shared by both awareness alerts.
    pub awareness_detail: String,
    pub factory_damage: String,
    /// `{pct}`.
    pub factory_damage_detail: String,
    pub overcrowded: String,
    /// `{n}`, `{capacity}`.
    pub overcrowded_detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkforceStrings {
    pub title: String,
    /// `{count}`, `{capacity}`.
    pub subtitle: String,
    /// `{mode}`.
    pub hud_button: String,
    pub sustain: String,
    pub sustain_detail: String,
    pub balanced: String,
    pub balanced_detail: String,
    pub salvage: String,
    pub salvage_detail: String,
    pub defense: String,
    pub defense_detail: String,
    /// `{mode}`.
    pub active: String,
    pub close: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdviceStrings {
    pub vault_takeover: String,
    pub vault_takeover_detail: String,
    pub vault_takeover_cost: String,
    pub vault_takeover_risk: String,
    pub vault_hold: String,
    pub vault_hold_detail: String,
    pub restore_power: String,
    /// `{n}`.
    pub restore_power_detail: String,
    pub restore_power_cost: String,
    pub restore_power_risk: String,
    pub build_tower: String,
    pub build_tower_detail: String,
    /// `{scrap}`, `{power}`.
    pub build_tower_cost: String,
    pub build_tower_risk: String,
    /// `{id}`.
    pub power_pad: String,
    pub power_pad_detail: String,
    /// `{path}`.
    pub risk_opens: String,
    pub risk_none: String,
    pub start_beacon: String,
    pub start_beacon_detail: String,
    pub start_beacon_cost: String,
    /// `{phase}`.
    pub start_beacon_risk: String,
    pub recall: String,
    pub recall_detail: String,
    pub recall_cost: String,
    pub recall_risk: String,
    pub shutdown: String,
    pub shutdown_detail: String,
    pub shutdown_cost: String,
    pub shutdown_risk: String,
    pub console: String,
    pub console_detail: String,
    pub cost_none: String,
    /// `{name}`.
    pub repair: String,
    /// `{name}`.
    pub power: String,
    /// `{name}`.
    pub inspect: String,
    /// `{n}`.
    pub cost_scrap: String,
    pub cost_online: String,
    pub cost_locked: String,
    /// `{n}`.
    pub risk_noise: String,
    pub risk_offline: String,
    pub disabled_detail: String,
    pub purpose_grid: String,
    pub purpose_food: String,
    pub purpose_income: String,
    pub purpose_turn_online: String,
    pub purpose_next_damaged: String,
}

/// Machine benefit read-outs. `{n}` in each rate line.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoonStrings {
    pub scrap: String,
    pub food: String,
    pub water: String,
    pub power: String,
    pub separator: String,
    pub stability: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelStrings {
    pub systems: String,
    pub systems_subtitle: String,
    /// `{n}`.
    pub repair_cost: String,
    /// `{n}`.
    pub power_cost: String,
    pub online: String,
    pub locked: String,
    /// `{text}`.
    pub benefit: String,
    /// `{n}`.
    pub risk_noise: String,
    /// `{name}`, `{role}`.
    pub wakes_sector: String,
    /// `{n}`, `{signature}`.
    pub awakening_risk: String,
    pub factory_console: String,
    pub factory_console_subtitle: String,
    /// `{n}`.
    pub unlock: String,
    pub unlocked: String,
    /// `{name}`, `{n}`.
    pub upgrade_row: String,
    pub factory_console_fallback: String,
    /// `{path}`.
    pub unlocks_entrance: String,
    /// `{id}`.
    pub pad: String,
    pub pad_subtitle: String,
    /// `{path}`.
    pub clearing_opens: String,
    pub clears_position: String,
    pub status_debris: String,
    pub status_cleared: String,
    pub status_occupied: String,
    pub status_powered: String,
    pub clear: String,
    pub power_pad: String,
    pub select_tower: String,
    /// `{n}`.
    pub build: String,
    /// `{verb}`, `{n}`.
    pub cost_action: String,
    /// `{name}`.
    pub recommended: String,
    pub no_tower_unlocked: String,
    /// `{name}`.
    pub covers_no_route: String,
    /// `{name}`, `{paths}`.
    pub covers_routes: String,
    pub requires_research: String,
    pub max_level: String,
    pub choose_specialization: String,
    /// `{name}`, `{n}`.
    pub specialize: String,
    /// `{name}`.
    pub specialized: String,
    /// `{n}`.
    pub upgrade: String,
    /// `{name}`, `{level}`.
    pub tower_title: String,
    pub tower_subtitle: String,
    pub tower_fallback_name: String,
    /// `{damage}`, `{range}`, `{fire}`, `{power}`.
    pub tower_stats: String,
    /// `{shots}`, `{hits}`, `{kills}`.
    pub tower_wave_stats: String,
    /// `{mode}`.
    pub target_priority: String,
    pub target_closest: String,
    pub target_first: String,
    pub target_strongest: String,
    pub target_wounded: String,
    pub target_fastest: String,
    pub coverage_none: String,
    /// `{paths}`.
    pub coverage: String,
    pub building_subtitle: String,
    /// `{text}`.
    pub covers: String,
    /// `{text}`.
    pub expected: String,
    /// `{text}`.
    pub cost: String,
    /// `{n}`.
    pub need_scrap_line: String,
    /// `{scrap}`, `{power}`.
    pub cost_line: String,
    pub none: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildPanelStrings {
    pub title: String,
    pub subtitle: String,
    /// `{name}`, `{scrap}`, `{power}`.
    pub tower_label: String,
    pub affordable: String,
    /// `{n}`.
    pub need_scrap: String,
    /// `{n}`.
    pub need_power: String,
    /// `{name}`.
    pub needs: String,
    pub locked: String,
    pub map_key: String,
    pub key_powered_pad: String,
    pub key_unpowered_pad: String,
    pub key_debris: String,
    pub key_machine: String,
    pub key_entrance: String,
    pub key_route: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapStrings {
    /// `{path}`.
    pub can_open: String,
    pub locked_section: String,
    pub power_frontier: String,
    pub opens_this_route: String,
    pub selected_tower: String,
    pub factory: String,
    /// Short glyph text drawn on machine nodes, keyed by building type.
    pub node_labels: HashMap<String, String>,
    pub node_fallback: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsStrings {
    pub title: String,
    pub autosave: String,
    pub fast_start: String,
    pub master_volume: String,
    pub sfx_volume: String,
    pub tutorial_replay: String,
    pub tutorial_will_show: String,
    pub close: String,
}

#[cfg(test)]
mod tests;
