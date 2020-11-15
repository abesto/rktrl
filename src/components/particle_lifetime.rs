#[derive(Clone)]
pub struct ParticleLifetime {
    lifetime_ms: f32,
}

impl ParticleLifetime {
    #[must_use]
    pub fn new(lifetime_ms: f32) -> Self {
        ParticleLifetime { lifetime_ms }
    }

    pub fn age(&mut self, by: f32) {
        self.lifetime_ms -= by;
    }

    pub fn is_dead(&self) -> bool {
        self.lifetime_ms < 0.0
    }
}
