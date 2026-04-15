use std::{any::Any, f32::consts::PI, sync::Arc};

use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use imageproc::geometric_transformations::{Interpolation, rotate_about_center};

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::Frame,
};

#[derive(Clone, Copy)]
pub enum Property {
    Opacity,
    X,
    Y,
    Rotation,
}

impl Property {
    fn parse(input: &str) -> Option<Self> {
        match input {
            "opacity" => Some(Self::Opacity),
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "rotation" | "rotate" => Some(Self::Rotation),
            _ => None,
        }
    }

    fn apply(&self, image: &DynamicImage, value: i32) -> Result<DynamicImage, String> {
        match self {
            Self::Opacity => Self::apply_opacity(image, value.unsigned_abs()),
            Self::X => Self::apply_pos(
                image,
                (image.width() as f32 * (value as f32 / 100.0)) as i32,
                0,
            ),
            Self::Y => Self::apply_pos(
                image,
                0,
                (image.height() as f32 * (value as f32 / 100.0)) as i32,
            ),
            Self::Rotation => {
                Ok(rotate_about_center(&image.to_rgba8(), value as f32 * (PI / 180.0), Interpolation::Bilinear, Rgba([0,0,0,0])).into()) 
            }
        }
    }

    fn apply_pos(image: &DynamicImage, x: i32, y: i32) -> Result<DynamicImage, String> {

        let (w, h) = image.dimensions();
        let mut out = DynamicImage::new_rgba8(w, h);

        let src_x = x.max(0) as u32;
        let src_y = y.max(0) as u32;

        let dst_x = (-x).max(0) as u32;
        let dst_y = (-y).max(0) as u32;

        let copy_w = w.saturating_sub(src_x + dst_x);
        let copy_h = h.saturating_sub(src_y + dst_y);

        if copy_w > 0 && copy_h > 0 {
            let sub = image.view(src_x, src_y, copy_w, copy_h).to_image();
            out.copy_from(&sub, dst_x, dst_y).ok();
        }

        Ok(out)
    }

    fn apply_opacity(image: &DynamicImage, opacity: u32) -> Result<DynamicImage, String> {
        let mut out = image.clone();

        out.as_mut_rgba8()
            .ok_or("Failed to get image as rgba8".to_string())?
            .iter_mut()
            .skip(3)
            .step_by(4)
            .for_each(|pixel| *pixel = (*pixel as f32 * (opacity as f32 / 100.0)).round() as u8);
        Ok(out)
    }
}

#[derive(Clone)]
pub struct Animate(Property, i32, i32, i8);

impl Action for Animate {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _discord: &Arc<AppState>) -> bool {
        let Some(mut arguments) = input
            .strip_prefix("animate(")
            .and_then(|s| s.strip_suffix(")"))
            .map(|s| s.split(","))
        else {
            return false;
        };
        let (property, start, end, speed) = (
            arguments
                .next()
                .and_then(Property::parse)
                .unwrap_or(Property::Opacity),
            arguments
                .next()
                .and_then(|arg| arg.parse().ok())
                .unwrap_or(0),
            arguments
                .next()
                .and_then(|arg| arg.parse().ok())
                .unwrap_or(100),
            arguments
                .next()
                .and_then(|arg| arg.parse().ok())
                .unwrap_or(5),
        );

        actions.push(Box::new(Animate(property, start, end, speed)));
        true
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let last = images.pop().ok_or("No image to animate.")?;
            let reverse = self.2 < self.1;
            if reverse {
                for value in (self.2..=self.1).rev().step_by(self.3 as usize) {
                    let image = self.0.apply(&last.image, value)?;
                    images.push(Frame::new(image, 10, action));
                }
            } else {
                for value in (self.1..=self.2).step_by(self.3 as usize) {
                    let image = self.0.apply(&last.image, value)?;
                    images.push(Frame::new(image, 10, action));
                }
            }
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Animate {
    pub fn new(property: Property, start: i32, end: i32, speed: i8) -> Self {
        Self(property, start,end, speed)
    }
}
