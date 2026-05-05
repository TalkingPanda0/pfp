use std::{any::Any, sync::Arc};

use anyhow::{anyhow, bail};

use crate::{
    AppState,
    action::{Action, ActionResult},
    actions::{opacity::Opacity, pad::Pad, rotate::Rotate,scale::Scale},
    frames::{Frame, Frames},
};

#[derive(Clone, Copy)]
pub enum Property {
    Opacity,
    X,
    Y,
    Rotation,
    ScaleX,
    ScaleY,
}

impl Property {
    fn parse(input: &str) -> Option<Self> {
        match input.to_ascii_lowercase().as_str() {
            "opacity" => Some(Self::Opacity),
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "rotation" | "rotate" => Some(Self::Rotation),
            "scalex" => Some(Self::ScaleX),
            "scaley" => Some(Self::ScaleY),
            _ => None,
        }
    }
    fn get_action(&self, value: i32) -> Box<dyn Action> {
        match self {
            Property::Opacity => Box::new(Opacity::new(value.unsigned_abs() as u8)),
            Property::Rotation => Box::new(Rotate::new(value)),
            Property::X => Box::new(Pad::new(value,0,true)),
            Property::Y => Box::new(Pad::new(0,value,true)),
            Property::ScaleX => Box::new(Scale::new(value.unsigned_abs(),100)), 
            Property::ScaleY => Box::new(Scale::new(100,value.unsigned_abs())), 
        }
    }
}

#[derive(Clone, Copy)]
pub enum AnimateMode {
    Yoyo,
    Loop,
    Continue,
    End,
}

impl AnimateMode {
    fn parse(input: &str) -> Option<Self> {
        match input.to_ascii_lowercase().as_str() {
            "yoyo" => Some(Self::Yoyo),
            "loop" => Some(Self::Loop),
            "continue" => Some(Self::Continue),
            "end" => Some(Self::End),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Animate(Property, i32, i32, i8, AnimateMode);

impl Action for Animate {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _discord: &Arc<AppState>) -> bool {
        let Some(mut arguments) = input
            .strip_prefix("animate(")
            .and_then(|s| s.strip_suffix(")"))
            .map(|s| s.split(","))
        else {
            return false;
        };
        let (property, start, end, speed, mode) = (
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
            arguments
                .next()
                .and_then(AnimateMode::parse)
                .unwrap_or(AnimateMode::End),
        );

        actions.push(Box::new(Animate(property, start, end, speed, mode)));
        true
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let mut last = images.extract_action(-1);
            if last.is_empty() {
                bail!("No image to animate!");
            }
            let image_duration = last.duration();
            let duration = image_duration.max((self.2.abs_diff(self.1) / self.3 as u32) * 16);

            let mut range: Box<dyn Iterator<Item = i32> + Send> = if self.1 > self.2 {
                let range = (self.2..=self.1).rev().step_by(self.3 as usize);
                match self.4 {
                    AnimateMode::Yoyo => {
                        let backward = (self.2..=self.1).step_by(self.3 as usize);
                        Box::new(range.chain(backward).cycle())
                    }
                    AnimateMode::End | AnimateMode::Continue => Box::new(range),
                    AnimateMode::Loop => Box::new(range.cycle()),
                }
            } else {
                let range = (self.1..=self.2).step_by(self.3 as usize);
                match self.4 {
                    AnimateMode::Yoyo => {
                        let backward = (self.1..=self.2).rev().step_by(self.3 as usize);
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
                    if matches!(self.4, AnimateMode::Continue)
                        && let Some(last_value) = last_value
                    {
                        last_value
                    } else {
                        break;
                    }
                };

                let mut vec = vec![image];

                self.0.get_action(value).apply(&mut vec, action).await?;

                images.push(vec.remove(0));
                last_value = Some(value);
            }

            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Animate {
    pub fn new(property: Property, start: i32, end: i32, speed: i8, mode: AnimateMode) -> Self {
        Self(property, start, end, speed, mode)
    }
}
