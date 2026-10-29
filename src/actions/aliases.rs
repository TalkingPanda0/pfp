use linkme::distributed_slice;

use crate::{
    action::{Action, PARSERS},
    actions::{
        animate::{Animate, AnimateMode, Property},
        combine::Combine,
        scale::Scale,
        tenor::Tenor,
    },
};

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>) -> bool {
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
                Some(100),
                Some(0),
                Some(10),
                Some(AnimateMode::End),
            )));
            true
        }
        "fadein" => {
            actions.push(Box::new(Animate::new(
                Property::Opacity,
                Some(0),
                Some(100),
                Some(10),
                Some(AnimateMode::End),
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
