macro_rules! cae_system_state {
    ($name:ident { subscribe($($variant:ident),+) $($rest:tt)* }) => {
        paste! {
            pub struct $name {
                $([<$variant:snake>]: CAESubscription),+
            }

            impl $name {
                $(
                    fn [<$variant:snake _filter>](link: &Link) -> bool {
                         matches!(link.label, Label::$variant { .. })
                    }
                )+

                pub fn new(resources: &Resources) -> $name {
                    let cae = &mut *resources.get_mut::<CauseAndEffect>().unwrap();
                    $name {
                        $(
                        [<$variant:snake>]: cae.subscribe($name::[<$variant:snake _filter>])
                        ),+
                    }
                }
            }
        }
    };
}

macro_rules! extract_label {
    ($link:ident @ $variant:ident => $($field:ident),+) => {
        let ($($field),+) = match $link.label {
            Label::$variant {$($field),+, ..} => ($($field),+),
            _ => unreachable!()
        };
    };
}

macro_rules! find_nearest_ancestor {
    ($cae:ident, $effect:ident @ $variant:ident) => {
        $cae.find_nearest_ancestor(&$effect, |link| matches!(link.label, Label::$variant { .. }))
            .unwrap();
    }
}

macro_rules! extract_nearest_ancestor {
    ($cae:ident, $effect:ident @ $variant:ident => $($field:ident),+) => {
        let ($($field),+) = {
            let ancestor = find_nearest_ancestor!($cae, $effect @ $variant);
            extract_label!(ancestor @ $variant => $($field),+);
            ($($field),+)
        };
    }
}

macro_rules! extract_cause {
    ($cae:ident, $effect:ident @ $variant:ident => $($field:ident),+) => {
        let ($($field),+) = {
            let cause = $cae.get_cause($effect).unwrap();
            extract_label!(cause @ $variant => $($field),+);
            ($($field),+)
        };
    }
}
