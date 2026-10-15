use std::sync::Arc;

use linkme::distributed_slice;

use crate::{
    AppState,
    action::{Action, PARSERS},
};

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    let Some(count) = input
        .strip_prefix("times(")
        .and_then(|s| s.strip_suffix(")"))
        .and_then(|s| s.parse::<u32>().ok())
    else {
        return false;
    };
    let Some(last_action) = actions.last() else {
        return false;
    };

    let vec = vec![last_action.clone(); count as usize];
    actions.extend(vec);

    true
}
