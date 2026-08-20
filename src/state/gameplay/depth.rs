//! Depth progression rules and the persistent directive choice.

use crate::data::strings::{fill, text};
use crate::engine::depth::DepthDirective;

use super::GameplayState;

impl GameplayState {
    pub(crate) fn factory_depth(&self) -> u32 {
        self.map_state.deepest_visible_depth()
    }

    /// Deeper wings expose more of the machine's old routing network. The
    /// extra assault budget is small, visible, and paid back by the production
    /// multiplier below once the player keeps those wings online.
    pub(crate) fn depth_assault_bonus(&self) -> f32 {
        if self.survival_proof_active {
            return 0.0;
        }
        let base = self.factory_depth().saturating_sub(1) as f32 * 0.035;
        let directive_bonus = if self
            .depth_directive_for(self.factory_depth())
            .is_some_and(|directive| matches!(directive, DepthDirective::SalvageLifts))
        {
            0.04
        } else {
            0.0
        };
        base + directive_bonus
    }

    pub(crate) fn depth_production_multiplier(&self) -> f32 {
        if self.survival_proof_active {
            return 1.0;
        }
        let base = 1.0 + self.factory_depth().saturating_sub(1) as f32 * 0.025;
        if self
            .depth_directive_for(self.factory_depth())
            .is_some_and(|directive| matches!(directive, DepthDirective::SalvageLifts))
        {
            base + 0.08
        } else {
            base
        }
    }

    pub(crate) fn depth_route_speed_multiplier(&self) -> f32 {
        if self
            .depth_directive_for(self.factory_depth())
            .is_some_and(|directive| matches!(directive, DepthDirective::ReinforcedRoutes))
        {
            0.92
        } else {
            1.0
        }
    }

    pub(crate) fn depth_readout(&self) -> String {
        let depth = self.factory_depth();
        let name = text()
            .map
            .depth_names
            .get(depth.saturating_sub(1) as usize)
            .map(String::as_str)
            .unwrap_or("DEEP FACTORY");
        format!(
            "{} · {}",
            fill(
                &text().map.depth_label,
                &[("n", &format!("{depth:02}")), ("name", name)],
            ),
            self.factory.phase.label()
        )
    }

    pub(crate) fn depth_directive_for(&self, depth: u32) -> Option<DepthDirective> {
        self.depth_directives
            .get(depth.saturating_sub(1) as usize)
            .copied()
            .flatten()
    }

    pub(crate) fn choose_depth_directive(&mut self, directive: DepthDirective) {
        let Some(depth) = self.pending_depth_directive.take() else {
            return;
        };
        let index = depth.saturating_sub(1) as usize;
        if let Some(slot) = self.depth_directives.get_mut(index) {
            *slot = Some(directive);
        }
        let name = text()
            .depth_directive
            .chosen
            .replace("{name}", depth_directive_name(directive));
        self.push_notification(name);
        self.autosave();
    }
}

fn depth_directive_name(directive: DepthDirective) -> &'static str {
    let strings = &text().depth_directive;
    match directive {
        DepthDirective::ReinforcedRoutes => strings.reinforced_routes.as_str(),
        DepthDirective::SalvageLifts => strings.salvage_lifts.as_str(),
        DepthDirective::EvacuationRelay => strings.evacuation_relay.as_str(),
    }
}
