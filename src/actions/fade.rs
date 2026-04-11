use std::{any::Any, sync::Arc};

use image::DynamicImage;

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::Frame,
};

#[derive(Clone, Copy)]
pub struct Fade(bool);

fn apply_opacity(image: &mut DynamicImage, opacity: u8) -> Result<(), String> {
    image
        .as_mut_rgba8()
        .ok_or("Failed to get image as rgba8".to_string())?
        .iter_mut()
        .skip(3)
        .step_by(4)
        .for_each(|pixel| *pixel = (*pixel as f32 * (opacity as f32 / 100.0)).round() as u8 );
    Ok(())
}

impl Action for Fade {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _: &Arc<AppState>) -> bool {
        if input.eq_ignore_ascii_case("fadein") {
            actions.push(Box::new(Fade(true)));
            true
        } else if input.eq_ignore_ascii_case("fadeout") {
            actions.push(Box::new(Fade(false)));
            true
        } else {
            false
        }
    }
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>,action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let last = images.pop().ok_or("No image to fade.")?;

            if self.0 {
                for opacity in (0..=100).step_by(10) {
                    let mut image = last.image.clone();
                    apply_opacity(&mut image, opacity)?;
                    images.push(Frame::new(image, 10,action));
                }
            } else {
                for opacity in (0..=100).rev().step_by(10) {
                    let mut image = last.image.clone();
                    apply_opacity(&mut image, opacity)?;
                    images.push(Frame::new(image, 10,action));
                }
            }

            Ok(())
        })
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
