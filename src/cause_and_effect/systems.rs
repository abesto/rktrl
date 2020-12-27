use legion::system;

use crate::cause_and_effect::CauseAndEffect;

#[system]
pub fn cae_debug(#[resource] cae: &CauseAndEffect) {
    println!("{:?}", &cae.dot());
}

#[system]
pub fn cae_clear(#[resource] cae: &mut CauseAndEffect) {
    cae.new_turn();
}
