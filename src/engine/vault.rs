//! Deterministic final-operation state for taking the AI vault back.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultStage {
    Handshake,
    Severance,
    Override,
    Complete,
}

#[derive(Clone, Debug, Default)]
pub struct VaultTakeover {
    pub active: bool,
    pub progress: f32,
    pub upload_complete: bool,
}

impl VaultTakeover {
    pub fn start(&mut self) {
        if self.active || self.upload_complete {
            return;
        }
        self.active = true;
        self.progress = 0.0;
    }

    /// Returns true only on the tick that finishes the upload.
    pub fn tick(&mut self, dt: f32, total_seconds: f32) -> bool {
        if !self.active || self.upload_complete {
            return false;
        }
        self.progress = (self.progress + dt).min(total_seconds);
        if self.progress < total_seconds {
            return false;
        }
        self.active = false;
        self.upload_complete = true;
        true
    }

    pub fn fraction(&self, total_seconds: f32) -> f32 {
        if self.upload_complete {
            1.0
        } else {
            (self.progress / total_seconds.max(0.01)).clamp(0.0, 1.0)
        }
    }

    pub fn stage(&self, stage_seconds: f32) -> VaultStage {
        if self.upload_complete {
            return VaultStage::Complete;
        }
        if self.progress < stage_seconds {
            VaultStage::Handshake
        } else if self.progress < stage_seconds * 2.0 {
            VaultStage::Severance
        } else {
            VaultStage::Override
        }
    }
}

#[cfg(test)]
mod tests;
