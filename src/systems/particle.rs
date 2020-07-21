use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: FontCharType,
    lifetime: f32,
}

pub struct ParticleRequests {
    requests: Vec<ParticleRequest>,
}

impl ParticleRequests {
    #[must_use]
    pub fn new() -> ParticleRequests {
        ParticleRequests {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: FontCharType,
        lifetime: f32,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
        });
    }
}

impl Default for ParticleRequests {
    fn default() -> Self {
        ParticleRequests::new()
    }
}

systemdata!(ParticleSystemData(
    entities,
    write(ParticleRequests),
    read_expect(FrameData),
    write_storage(ParticleLifetime, Position, Renderable)
));

pub struct ParticleSystem;

impl<'a> System<'a> for ParticleSystem {
    type SystemData = ParticleSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        self.process_requests(&mut data);
        self.cull_dead_particles(&mut data);
    }
}

impl ParticleSystem {
    fn process_requests(&mut self, data: &mut ParticleSystemData) {
        for new_particle in data.particle_requests.requests.iter() {
            data.entities
                .build_entity()
                .with(
                    Position::new(new_particle.x, new_particle.y),
                    &mut data.positions,
                )
                .with(
                    Renderable {
                        color: ColorPair::new(new_particle.fg, new_particle.bg),
                        glyph: new_particle.glyph,
                        render_order: RenderOrder::Particle,
                    },
                    &mut data.renderables,
                )
                .with(
                    ParticleLifetime::new(new_particle.lifetime),
                    &mut data.particle_lifetimes,
                )
                .build();
        }

        data.particle_requests.requests.clear();
    }

    fn cull_dead_particles(&mut self, data: &mut ParticleSystemData) {
        let frame_time_ms = data.frame_data.frame_time_ms;

        for (lifetime,) in (&mut data.particle_lifetimes,).join() {
            lifetime.age(frame_time_ms);
        }

        for (entity, lifetime) in (&data.entities, &data.particle_lifetimes).join() {
            if lifetime.is_dead() {
                data.entities.delete(entity).expect("Particle will not die");
            }
        }
    }
}
