use crate::systems::prelude::*;

cae_system_state!(MeleeCombatSystemState {
    subscribe(MeleeIntent)
});

#[system]
#[read_component(Name)]
#[read_component(CombatStats)]
#[read_component(HungerClock)]
#[read_component(Equipped)]
#[read_component(MeleePowerBonus)]
#[read_component(DefenseBonus)]
#[read_component(Position)]
pub fn melee_combat(
    #[state] state: &MeleeCombatSystemState,
    #[resource] game_log: &mut GameLog,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] map: &Map,
    world: &SubWorld,
) {
    for ref melee_intent in cae.get_queue(state.melee_intent) {
        // Where are we attacking?
        extract_label!(melee_intent @ MeleeIntent => target_position);
        // What's there (if anything)?
        let maybe_target = map.get_tile_contents(target_position).and_then(|targets| {
            targets
                .iter()
                .find(|&&target| world.has_component::<CombatStats>(target))
        });
        let target = match maybe_target {
            Some(&target) => target,
            _ => {
                cae.add_effect(melee_intent, Label::AttackedEmptySpace);
                continue;
            }
        };

        // Who's attacking?
        extract_nearest_ancestor!(cae, melee_intent @ Turn => actor);

        // Details about the attacker
        let (attacker_name, attacker_stats, maybe_attacker_hunger_clock) =
            <(&Name, &CombatStats, Option<&HungerClock>)>::query()
                .get(world, actor)
                .unwrap();
        if attacker_stats.hp <= 0 {
            cae.add_effect(melee_intent, Label::AttackerIsAlreadyDead);
            continue;
        }

        // Details about the target
        let (target_stats, target_name) =
            <(&CombatStats, &Name)>::query().get(world, target).unwrap();
        if target_stats.hp <= 0 {
            cae.add_effect(melee_intent, Label::TargetIsAlreadyDead);
            continue;
        }

        let melee_action = cae.add_effect(melee_intent, Label::MeleeAction { target });
        // We don't currently have to-hit / accuracy, so an attack is always a hit
        let hit = cae.add_effect(&melee_action, Label::Hit);

        // Calculate attack power
        let hunger_attack_power_bonus = maybe_attacker_hunger_clock
            .map(|clock| match clock.state {
                HungerState::WellFed => 1,
                _ => 0,
            })
            .unwrap_or(0);

        let equipment_attack_power_bonus = <(&Equipped, &MeleePowerBonus)>::query()
            .iter(world)
            .filter(|(equipped, _)| equipped.owner == actor)
            .map(|(_, bonus)| bonus.power)
            .sum::<i32>();

        let power: i32 =
            equipment_attack_power_bonus + attacker_stats.power + hunger_attack_power_bonus;

        // Calculate defense power
        let defense: i32 = {
            <(&Equipped, &DefenseBonus)>::query()
                .iter(world)
                .filter(|(equipped, _)| equipped.owner == target)
                .map(|(_, bonus)| bonus.defense)
                .sum::<i32>()
                + target_stats.defense
        };

        // Calculate and deal damage
        let damage = i32::max(0, power - defense);
        cae.add_effect(
            &hit,
            Label::Damage {
                to: target,
                amount: damage,
                bleeding: true,
            },
        );

        // TODO this whole if-else can go away once GameLog and damage_system are migrated to CAE
        if damage == 0 {
            game_log.entries.push(format!(
                "{} is unable to hurt {}",
                attacker_name, target_name
            ));
        } else {
            game_log.entries.push(format!(
                "{} hits {}, for {} hp.",
                attacker_name, target_name, damage
            ));
        }

        cae.add_effect(
            &hit,
            Label::ParticleRequest {
                x: target_position.x,
                y: target_position.y,
                fg: RGB::named(ORANGE),
                bg: RGB::named(BLACK),
                glyph: to_cp437('‼'),
                lifetime: 200.0,
            },
        );
    }
}
