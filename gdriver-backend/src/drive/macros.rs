mod apply_change {
    #[macro_export]
    macro_rules! apply_change {
        ($meta:ident, $changed_meta:ident, $field:ident, $has_changed:expr) => {
            apply_change!($meta, $changed_meta, $field, $has_changed, where:true);
        };
        ($meta:ident, $changed_meta:ident, $field:ident, $has_changed:expr, where:$extra_condition:expr) => {
            apply_change!(
                $meta,
                $changed_meta,
                $field,
                $has_changed,
                $extra_condition,
                {
                    ::tracing::info!("{} changed from '{:?}' to '{:?}'", stringify!($field), $meta.$field, $changed_meta.$field);
                    $has_changed |= true;
                    $meta.$field = $changed_meta.$field.clone();
                }
            );
        };
        ($meta:ident, $changed_meta:ident, $field:ident, $has_changed:expr, $extra_condition:expr, $action_on_difference:expr) => {
            if $meta.$field != $changed_meta.$field && $extra_condition {
                $action_on_difference
            }
        };
    }
}
