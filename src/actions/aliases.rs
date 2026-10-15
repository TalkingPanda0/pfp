use std::sync::Arc;

use linkme::distributed_slice;

use crate::{
    AppState,
    action::{Action, PARSERS},
    actions::{
        animate::{Animate, AnimateMode, Property},
        combine::Combine,
        scale::Scale,
        tenor::Tenor,
    },
};

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    match input {
        "explode" => {
            actions.push(Box::new(Tenor::new(
                "explosion-deltarune-deltarune-explosion-boom-explode-gif-16548223447993760048"
                    .to_string(),
            )));
            actions.push(Box::new(Combine::new(true)));
            true
        }
        "fadeout" => {
            actions.push(Box::new(Animate::new(
                Property::Opacity,
                100,
                0,
                10,
                AnimateMode::End,
            )));
            true
        }
        "fadein" => {
            actions.push(Box::new(Animate::new(
                Property::Opacity,
                0,
                100,
                10,
                AnimateMode::End,
            )));
            true
        }
        "squish" => {
            actions.push(Box::new(Scale::new(200, 100)));
            true
        }
        _ => false,
    }
}
