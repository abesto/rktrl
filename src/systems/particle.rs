use bracket_lib::prelude::*;
use legion::{Entity, IntoQuery, system, systems::CommandBuffer, world::SubWorld};

use crate::{components::*, resources::*};

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

#[system]
#[read_component(Entity)]
#[write_component(ParticleLifetime)]
#[write_component(Position)]
#[write_component(Renderable)]
pub fn particle(
    world: &mut SubWorld,
    #[resource] particle_requests: &mut ParticleRequests,
    #[resource] frame_data: &FrameData,
    commands: &mut CommandBuffer,
) {
    process_requests(commands, particle_requests);
    cull_dead_particles(world, commands, frame_data);
}

fn process_requests(commands: &mut CommandBuffer, particle_requests: &mut ParticleRequests) {
    for new_particle in particle_requests.requests.iter() {
        commands.push((
            Position::new(new_particle.x, new_particle.y),
            Renderable {
                color: ColorPair::new(new_particle.fg, new_particle.bg),
                glyph: new_particle.glyph,
                render_order: RenderOrder::Particle,
            },
            ParticleLifetime::new(new_particle.lifetime),
        ));
    }

    particle_requests.requests.clear();
}

fn cull_dead_particles(world: &mut SubWorld, commands: &mut CommandBuffer, frame_data: &FrameData) {
    let frame_time_ms = frame_data.frame_time_ms;

    <(Entity, &mut ParticleLifetime)>::query().for_each_mut(world, |(entity, lifetime)| {
        lifetime.age(frame_time_ms);
        if lifetime.is_dead() {
            commands.remove(*entity);
        }
    });
}
