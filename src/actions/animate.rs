use anyhow::{anyhow, bail};
use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, actions::{opacity::Opacity, pad::Pad, rotate::Rotate, scale::Scale}, frames::{Frame, Frames}, parser::ParsableAction,
};

#[derive(Clone, Copy, Parsable)]
pub enum Property {
    Opacity,
    X,
    Y,
    Rotation,
    ScaleX,
    ScaleY,
}

impl Property {
    fn get_action(&self, value: i32) -> Box<dyn Action> {
        match self {
            Property::Opacity => Box::new(Opacity::new(value.unsigned_abs() as u8)),
            Property::Rotation => Box::new(Rotate::new(value)),
            Property::X => Box::new(Pad::new(Some(value), None, Some(true))),
            Property::Y => Box::new(Pad::new(None, Some(value), Some(true))),
            Property::ScaleX => Box::new(Scale::new(value.unsigned_abs(), 100)),
            Property::ScaleY => Box::new(Scale::new(100, value.unsigned_abs())),
        }
    }
}

#[derive(Clone, Copy, Parsable)]
pub enum AnimateMode {
    Yoyo,
    Loop,
    Continue,
    End,
}

#[derive(Clone, Copy,Parsable,ParsableAction)]
pub struct Animate {
    property: Property,
    start: Option<i32>,
    end: Option<i32>,
    speed: Option<i8>,
    mode: Option<AnimateMode>,
}

#[distributed_slice(PARSERS)]
static PARSER: Parser = Animate::parse_action;

impl Action for Animate {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let mut last = images.extract_action(-1);
            if last.is_empty() {
                bail!("No image to animate!");
            }

            let start = self.start.unwrap_or(0);
            let end = self.end.unwrap_or(100);
            let speed = self.speed.unwrap_or(5);
            let mode = self.mode.unwrap_or(AnimateMode::End);

            let image_duration = last.duration();
            let duration = image_duration.max((end.abs_diff(start) / speed as u32) * 16);

            let mut range: Box<dyn Iterator<Item = i32> + Send> = if start > end {
                let range = (end..=start).rev().step_by(speed as usize);
                match mode {
                    AnimateMode::Yoyo => {
                        let backward = (end..=start).step_by(speed as usize);
                        Box::new(range.chain(backward).cycle())
                    }
                    AnimateMode::End | AnimateMode::Continue => Box::new(range),
                    AnimateMode::Loop => Box::new(range.cycle()),
                }
            } else {
                let range = (start..=end).step_by(speed as usize);
                match mode {
                    AnimateMode::Yoyo => {
                        let backward = (start..=end).rev().step_by(speed as usize);
                        Box::new(range.chain(backward).cycle())
                    }
                    AnimateMode::End | AnimateMode::Continue => Box::new(range),
                    AnimateMode::Loop => Box::new(range.cycle()),
                }
            };
            let mut last_value: Option<i32> = None;

            for ts in (0..=duration).step_by(16) {
                let mut image = last
                    .get_at_timestamp(ts % image_duration)
                    .ok_or(anyhow!("Failed to get frame for combine."))?
                    .clone();
                image.delay = 16;

                let value = if let Some(value) = range.next() {
                    value
                } else {
                    if matches!(mode, AnimateMode::Continue)
                        && let Some(last_value) = last_value
                    {
                        last_value
                    } else {
                        break;
                    }
                };

                let mut vec = vec![image];

                self.property.get_action(value).apply(&mut vec, action).await?;

                images.push(vec.remove(0));
                last_value = Some(value);
            }

            Ok(())
        })
    }
}

impl Animate {
    pub fn new(property: Property, start: Option<i32>, end: Option<i32>, speed: Option<i8>, mode: Option<AnimateMode>) -> Self {
        Self{property, start, end, speed, mode}
    }
}
