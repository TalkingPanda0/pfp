use std::{any::Any, sync::Arc};

use imageproc::compose::{overlay_mut};

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::{Frame, Frames},
};

fn combine(base: &mut Frame, overlay: &mut Frame, resize: bool) -> Result<(), String> {
    let (width, height) = (
        base.image.width().max(overlay.image.width()),
        base.image.height().max(overlay.image.height()),
    );

    if resize || base.image.width() < width || base.image.height() < height {
        base.resize(width, height)?;
        overlay.resize(width, height)?;
    }
    let base_buffer = base
        .image
        .as_mut_rgba8()
        .ok_or("Failed to convert image.".to_string())?;
    let overlay_buffer = overlay
        .image
        .as_rgba8()
        .ok_or("Failed to convert image.".to_string())?;

    overlay_mut(
        base_buffer,
        overlay_buffer,
        (base_buffer.width() - overlay_buffer.width()) / 2,
        (base_buffer.height() - overlay_buffer.height()) / 2,
    );
    Ok(())
}

#[derive(Clone, Copy)]
pub struct Combine(bool);

impl Combine {
    pub fn new(resize: bool) -> Self {
        Self(resize)
    }
}

impl Action for Combine {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        if input.starts_with("combine") {
            actions.push(Box::new(Combine(input.ends_with("r"))));
            true
        } else {
            false
        }
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move { 
            let mut overlay = images.get_from_action(-1, action);
            let mut base = images.get_from_action(-2, action);

            if overlay.is_empty() || base.is_empty() {
                return Err("No images to combine.".to_string());
            }

            let base_duration = base.duration();
            let overlay_duration = overlay.duration();

            let min_delay = base.min_delay().min(overlay.min_delay());
            let duration = base_duration.max(overlay_duration);

            let mut combined: Vec<Frame> = Vec::new();

            for ts in (0..=duration).step_by(min_delay as usize) {
                let mut base_frame = base.get_at_timestamp(ts % base_duration).ok_or("Failed to get frame for combine.".to_string())?.clone();
                let overlay_frame = overlay.get_at_timestamp(ts % overlay_duration).ok_or("Failed to get frame for combine.".to_string())?;
                combine(&mut base_frame, overlay_frame, self.0)?;

                combined.push(Frame{ delay: min_delay as i32,..base_frame } );
            }

            images.truncate(images.len() - (base.len() + overlay.len()));
            images.extend(combined);

            Ok(()) 
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
