use std::{any::Any, sync::Arc};

use ab_glyph::{FontRef, PxScale};
use hex_color::HexColor;
use image::{DynamicImage, GenericImageView, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::{Frame, Frames},
};

#[derive(Clone)]
enum Alignment {
    Top,
    Middle,
    Bottom,
}

impl Alignment {
    fn parse(input: &str) -> Option<Self> {
        match input {
            "top" => Some(Alignment::Top),
            "middle" => Some(Alignment::Middle),
            "bottom" => Some(Alignment::Bottom),
            _ => None,
        }
    }
}

fn find_best_size(
    width: u32,
    height: u32,
    min: f32,
    max: f32,
    font: &FontRef,
    text: &str,
) -> (u32, u32, PxScale) {
    let mut scale = PxScale::from(min);
    let mut result_width = 0;
    let mut result_height = 0;

    loop {
        let (text_width, text_height) = text_size(scale, font, text);

        if text_width > width || text_height > height {
            scale.x -= 1.0;
            scale.y -= 1.0;
            break;
        }
        (result_width, result_height) = (text_width, text_height);
        if scale.x >= max {
            break;
        }
        scale.x += 1.0;
        scale.y += 1.0;
    }

    (result_width, result_height, scale)
}

fn parse_color(input: &str) -> Option<Rgba<u8>> {
    match input {
        "white" => Some(Rgba([255, 255, 255, 255])),
        "black" => Some(Rgba([0, 0, 0, 255])),
        "red" => Some(Rgba([255, 0, 0, 255])),
        "green" => Some(Rgba([0, 255, 0, 255])),
        "blue" => Some(Rgba([0, 0, 255, 255])),
        _ => HexColor::parse(&format!("#{input}"))
            .map(|c| Rgba([c.r, c.g, c.b, c.a]))
            .ok(),
    }
}

// text(Text,Alignment,Color,Size,X,Y)
#[derive(Clone)]
pub struct TextAction(
    String,
    Alignment,
    Rgba<u8>,
    Option<f32>,
    Option<i32>,
    Option<i32>,
);

impl TextAction {
    fn draw(&self, image: &mut DynamicImage) -> Result<(), String> {
        let (width, height) = image.dimensions();
        let font =
            FontRef::try_from_slice(include_bytes!("/usr/share/fonts/TTF/Roboto-Regular.ttf"))
                .map_err(|err| err.to_string())?;
        let (text_width, text_height, text_scale) = if let Some(scale) = self.3 {
            let scale = PxScale::from(scale);
            let (width, height) = text_size(scale, &font, &self.0);

            (width, height, scale)
        } else {
            find_best_size(width, height / 2, 12.0, 128.0, &font, &self.0)
        };

        let x = self.4.unwrap_or(((width - text_width) / 2) as i32);
        let y = self.5.unwrap_or_else(|| {
            match self.1 {
                Alignment::Top => 0.0,
                Alignment::Middle => (height as f32 * 0.5) - (text_height as f32 * 0.5),
                Alignment::Bottom => height as f32 - text_height as f32,
            }
            .round() as i32
        });

        draw_text_mut(image, self.2, x, y, text_scale, &font, &self.0);
        Ok(())
    }
}

impl Action for TextAction {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _: &Arc<AppState>) -> bool {
        let Some(mut arguments) = input
            .strip_prefix("text(")
            .and_then(|s| s.strip_suffix(")"))
            .map(|s| s.split(","))
        else {
            return false;
        };
        let (text, alignment, color, scale, x, y) = (
            arguments.next().unwrap_or(""),
            arguments
                .next()
                .and_then(Alignment::parse)
                .unwrap_or(Alignment::Bottom),
            arguments
                .next()
                .and_then(parse_color)
                .unwrap_or(Rgba([255, 255, 255, 255])),
            arguments.next().and_then(|s| s.parse::<f32>().ok()),
            arguments.next().and_then(|s| s.parse::<i32>().ok()),
            arguments.next().and_then(|s| s.parse::<i32>().ok()),
        );

        actions.push(Box::new(TextAction(
            text.to_string(),
            alignment,
            color,
            scale,
            x,
            y,
        )));
        true
    }
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, _action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            for frame in images.get_mut_action(-1) {
                self.draw(&mut frame.image)?;
            }
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
