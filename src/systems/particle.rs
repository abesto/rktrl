use bracket_lib::prelude::*;
use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, IntoQuery, Resources};

use crate::cause_and_effect::{CAESubscription, CauseAndEffect, Label, Link};
use crate::{components::*, resources::*};

pub struct ParticleSystemState {
    subscription: CAESubscription,
}

impl ParticleSystemState {
    fn subscription_filter(link: &Link) -> bool {
        matches!(link.label, Label::ParticleRequest { .. })
    }

    pub fn new(resources: &Resources) -> ParticleSystemState {
        let cae = &mut *resources.get_mut::<CauseAndEffect>().unwrap();
        ParticleSystemState {
            subscription: cae.subscribe(ParticleSystemState::subscription_filter),
        }
    }
}

#[system]
#[read_component(Entity)]
#[write_component(ParticleLifetime)]
#[write_component(Position)]
#[write_component(Renderable)]
pub fn particle(
    world: &mut SubWorld,
    #[state] state: &ParticleSystemState,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] frame_data: &FrameData,
    commands: &mut CommandBuffer,
) {
    process_requests(commands, state, cae);
    cull_dead_particles(world, commands, frame_data);
}

fn process_requests(
    commands: &mut CommandBuffer,
    state: &ParticleSystemState,
    cae: &mut CauseAndEffect,
) {
    for link in cae.get_queue(state.subscription) {
        if let Label::ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
        } = link.label
        {
            commands.push((
                Position::new(x, y),
                Renderable {
                    glyph,
                    color: ColorPair::new(fg, bg),
                    render_order: RenderOrder::Particle,
                },
                ParticleLifetime::new(lifetime),
            ));
        } else {
            unreachable!();
        }
    }
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
