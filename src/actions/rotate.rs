use std::{f32::consts::PI};

use image::Rgba;
use imageproc::geometric_transformations::{Interpolation, rotate_about_center};
use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::{Frame, Frames}, parser::ParsableAction,
};

#[derive(Clone, Copy,Parsable,ParsableAction)]
pub struct Rotate(i32);

#[distributed_slice(PARSERS)]
static PARSER: Parser = Rotate::parse_action;

impl Action for Rotate {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            for frame in frames {
                let rotated = rotate_about_center(
                    &frame.image.to_rgba8(),
                    self.0 as f32 * (PI / 180.0),
                    Interpolation::Bilinear,
                    Rgba([0, 0, 0, 0]),
                );
                frame.image = rotated.into();
                frame.action = action;
            }
            Ok(())
        })
    }
}

impl Rotate {
    pub fn new(rotation: i32) -> Self {
        Self(rotation)
    }
}
