use anyhow::bail;
use image::{GenericImageView, imageops::FilterType};
use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::{Frame, Frames}, parser::ParsableAction,
};

#[derive(Clone, Copy,Parsable,ParsableAction)]
pub struct Scale(u32, u32);

#[distributed_slice(PARSERS)]
static PARSER: Parser = Scale::parse_action;

impl Action for Scale {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, _action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            if frames.is_empty() {
                bail!("No images to scale.");
            }
            for frame in frames {
                let (width, height) = frame.image.dimensions();
                let (width, height) = (
                    (width as f32 * (self.0 as f32 / 100.0)) as u32,
                    ((height as f32 * (self.1 as f32 / 100.0)) as u32),
                );
                Frame::can_fit(width, height)?;
                frame.image = frame
                    .image
                    .resize_exact(width, height, FilterType::Triangle);
            }

            Ok(())
        })
    }
}

impl Scale {
    pub fn new(width: u32, height: u32) -> Self {
        Self(width, height)
    }
}
