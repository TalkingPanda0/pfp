use ab_glyph::{FontRef, PxScale};
use anyhow::Result;
use hex_color::HexColor;
use image::{DynamicImage, GenericImageView, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::{Frame, Frames}, parser::{Parsable, ParsableAction},
};

#[derive(Clone, Parsable)]
enum Alignment {
    Top,
    Middle,
    Bottom,
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

impl Parsable for Rgba<u8> {
    fn parse<'a>(input: &mut impl Iterator<Item = &'a str>) -> Option<Self> {
        match input.next()? {
            "white" => Some(Rgba([255, 255, 255, 255])),
            "black" => Some(Rgba([0, 0, 0, 255])),
            "red" => Some(Rgba([255, 0, 0, 255])),
            "green" => Some(Rgba([0, 255, 0, 255])),
            "blue" => Some(Rgba([0, 0, 255, 255])),
            text => HexColor::parse(&format!("#{text}"))
                .map(|c| Rgba([c.r, c.g, c.b, c.a]))
                .ok(),
        }
    }
}

// text(Text,Alignment,Color,Size,X,Y)
#[derive(Clone,Parsable,ParsableAction)]
#[name("text")]
pub struct TextAction(
    String,
    Option<Alignment>,
    Option<Rgba<u8>>,
    Option<f32>,
    Option<i32>,
    Option<i32>,
);

impl TextAction {
    fn draw(&self, image: &mut DynamicImage) -> Result<()> {
        let (width, height) = image.dimensions();
        let font =
            FontRef::try_from_slice(include_bytes!("/usr/share/fonts/TTF/Roboto-Regular.ttf"))?;
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
                Some(Alignment::Top) => 0.0,
                Some(Alignment::Middle) => (height as f32 * 0.5) - (text_height as f32 * 0.5),
                _ => height as f32 - text_height as f32,
            }
            .round() as i32
        });

        draw_text_mut(image, self.2.unwrap_or(Rgba([255, 255, 255, 255])), x, y, text_scale, &font, &self.0);
        Ok(())
    }
}

#[distributed_slice(PARSERS)]
static PARSER: Parser = TextAction::parse_action;

impl Action for TextAction {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, _action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            for frame in images.get_mut_action(-1) {
                self.draw(&mut frame.image)?;
            }
            Ok(())
        })
    }
}
