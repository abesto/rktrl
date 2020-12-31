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

macro_rules! cae_system_state {
    ($name:ident { $($field:ident($link:ident) $filter_body:block)+ }) => {
        pub struct $name {
            $($field: CAESubscription),+
        }

        paste! {
            impl $name {
                $(fn [<$field _filter>]($link: &Link) -> bool $filter_body)+

                pub fn new(resources: &Resources) -> $name {
                    let cae = &mut *resources.get_mut::<CauseAndEffect>().unwrap();
                    $name {
                        $(
                        $field: cae.subscribe($name::[<$field _filter>]),
                        ),+
                    }
                }
            }
        }
    };
}
