use image::{DynamicImage, GenericImage, GenericImageView};
use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::{Frame, Frames}, parser::ParsableAction,
};

#[derive(Clone, Copy,Parsable,ParsableAction)]
pub struct Pad(Option<i32>, Option<i32>, Option<bool>);

#[distributed_slice(PARSERS)]
static PARSER: Parser = Pad::parse_action;

impl Action for Pad {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            let xmult = self.0.unwrap_or_default() as f32 / 100.0;
            let ymult = self.1.unwrap_or_default() as f32 / 100.0;
            let keep = self.2.unwrap_or_default();
            for frame in frames {
                let image = &frame.image;

                let (w, h) = image.dimensions();
                let (x, y) = ((w as f32 * xmult) as i32, (h as f32 * ymult) as i32);
                let (ow, oh) = if keep {
                    (w, h)
                } else {
                    (w + x.unsigned_abs(), h + y.unsigned_abs())
                };

                Frame::can_fit(ow, oh)?;
                let mut out = DynamicImage::new_rgba8(ow, oh);

                let src_x = x.max(0) as u32;
                let src_y = y.max(0) as u32;

                let dst_x = (-x).max(0) as u32;
                let dst_y = (-y).max(0) as u32;

                let copy_w = ow.saturating_sub(src_x + dst_x);
                let copy_h = oh.saturating_sub(src_y + dst_y);

                if keep {
                    if copy_w > 0 && copy_h > 0 {
                        let sub = image.view(src_x, src_y, copy_w, copy_h).to_image();
                        out.copy_from(&sub, dst_x, dst_y)?;
                    }
                } else {
                    out.copy_from(image, dst_x, dst_y)?;
                }

                frame.image = out;
                frame.action = action;
            }
            Ok(())
        })
    }
}


impl Pad {
    pub fn new(x: Option<i32>, y: Option<i32>, keep_size: Option<bool>) -> Self {
        Self(x, y, keep_size)
    }
}
