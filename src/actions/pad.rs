use std::{any::Any, sync::Arc};

use image::{DynamicImage, GenericImage, GenericImageView};

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::{Frame, Frames},
};

#[derive(Clone, Copy)]
pub struct Pad(i32, i32, bool);

impl Action for Pad {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        let Some(mut args) = input
            .strip_prefix("pad(")
            .and_then(|s| s.strip_suffix(")"))
            .map(|s| s.split(","))
        else {
            return false;
        };

        let (x, y, keep) = (
            args.next().and_then(|s| s.parse().ok()).unwrap_or(0),
            args.next().and_then(|s| s.parse().ok()).unwrap_or(0),
            args.next().and_then(|s| s.parse().ok()).unwrap_or(false),
        );

        actions.push(Box::new(Self(x, y, keep)));

        true
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            let xmult = self.0 as f32 / 100.0;
            let ymult = self.1 as f32 / 100.0;
            for frame in frames {
                let image = &frame.image;

                let (w, h) = image.dimensions();
                let (x, y) = ((w as f32 * xmult) as i32, (h as f32 * ymult) as i32);
                let (ow, oh) = if self.2 {
                    (w, h)
                } else {
                    ((w as i32 + x) as u32, (h as i32 + y) as u32)
                };

                Frame::can_fit(ow, oh)?;
                let mut out = DynamicImage::new_rgba8(ow, oh);

                let src_x = x.max(0) as u32;
                let src_y = y.max(0) as u32;

                let dst_x = (-x).max(0) as u32;
                let dst_y = (-y).max(0) as u32;

                let copy_w = ow.saturating_sub(src_x + dst_x);
                let copy_h = oh.saturating_sub(src_y + dst_y);

                if self.2 {
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Pad {
    pub fn new(x: i32, y: i32, keep_size: bool) -> Self {
        Self(x, y, keep_size)
    }
}
