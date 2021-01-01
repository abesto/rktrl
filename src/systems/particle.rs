use crate::systems::prelude::*;

cae_system_state!(ParticleSystemState {
    requests(link) { matches!(link.label, Label::ParticleRequest { .. }) }
});

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
    for link in cae.get_queue(state.requests) {
        extract_label!(link @ ParticleRequest => x, y, fg, bg, glyph, lifetime);
        commands.push((
            Position::new(x, y),
            Renderable {
                glyph,
                color: ColorPair::new(fg, bg),
                render_order: RenderOrder::Particle,
            },
            ParticleLifetime::new(lifetime),
        ));
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
