macro_rules! cae_system_state {
    ($name:ident { $($field:ident : $($variant:ident)|+),+ }) => {
        pub struct $name {
            $($field: CAESubscription),+
        }

        paste! {
            impl $name {
                $(
                    fn [<$field _filter>](link: &Link) -> bool {
                         $(matches!(link.label, Label::$variant { .. }) )||+
                    }
                )+

                pub fn new(resources: &Resources) -> $name {
                    let cae = &mut *resources.get_mut::<CauseAndEffect>().unwrap();
                    $name {
                        $(
                        $field: cae.subscribe($name::[<$field _filter>])
                        ),+
                    }
                }
            }
        }
    };
}

macro_rules! extract_label {
    // Separate macro branch for a single field, to satisfy unused_parens lint
    ($link:ident@$variant:ident => $field:ident) => {
        let $field = match $link.label {
            Label::$variant {$field, ..} => $field,
            _ => unreachable!()
        };
    };

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
    ($cae:ident, $effect:ident @ $variant:ident => $field:tt) => {
        let $field = {
            let ancestor = find_nearest_ancestor!($cae, $effect @ $variant);
            extract_label!(ancestor @ $variant => $field);
            $field
        };
    }
}
