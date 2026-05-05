use ab_glyph::{FontRef, PxScale};
use anyhow::{Result, anyhow, bail};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};
use itertools::Itertools;
use webpx::{AnimationDecoder, AnimationEncoder, Encoder, Unstoppable, decode_rgba};

const MAX_SIZE: u32 = 16383;

#[derive(Clone)]
pub struct Frame {
    pub image: DynamicImage,
    pub delay: i32,
    pub action: u32,
}

impl Frame {
    pub fn new(image: DynamicImage, delay: i32, action: u32) -> Self {
        Self {
            image,
            delay,
            action,
        }
    }

    pub fn from_webp(bytes: &[u8], delay: i32, action: u32) -> Result<Frame> {
        let decoded = decode_rgba(bytes)?;
        let image = RgbaImage::from_raw(decoded.1, decoded.2, decoded.0)
            .ok_or(anyhow!("Failed to decode webp image."))?
            .into();

        Ok(Self::new(image, delay, action))
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        let resized = self
            .image
            .resize(width, height, image::imageops::FilterType::Triangle);
        let mut canvas =
            DynamicImage::ImageRgba8(ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0])));

        let (w, h) = resized.dimensions();

        canvas.copy_from(&resized, (width - w) / 2, (height - h) / 2)?;

        self.image = canvas;
        Ok(())
    }
}

pub fn get_error_image(error: String) -> Result<Vec<u8>> {
    let font = FontRef::try_from_slice(include_bytes!("/usr/share/fonts/TTF/Roboto-Regular.ttf"))?;
    let size = PxScale::from(24.0);
    let (width, height) = text_size(size, &font, &error);
    let mut canvas = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        width + 10,
        height,
        Rgba([0, 0, 0, 0]),
    ));
    draw_text_mut(
        &mut canvas,
        Rgba([255, 0, 0, 255]),
        5,
        0,
        size,
        &font,
        &error,
    );
    let encoded = Encoder::new_rgba(
        &canvas.to_rgba8().into_raw(),
        canvas.width(),
        canvas.height(),
    )
    .method(0)
    .quality(75.0)
    .encode(Unstoppable)?;
    Ok(encoded)
}

pub trait Frames: Sized {
    fn clone_action(&self, action: i32, current_action: u32) -> Vec<Frame>;
    fn get_mut_action(&mut self, action: i32) -> Vec<&mut Frame>;
    fn extract_action(&mut self, action: i32) -> Vec<Frame>;
    fn get_action_from_relative(&self, action: i32) -> u32;
    fn get_at_timestamp(&mut self, timestamp: u32) -> Option<&mut Frame>;
    fn duration(&self) -> u32;
    fn min_delay(&self) -> u32;

    fn dimensions(&self) -> (u32, u32);
    fn dimensions_column(&self) -> (u32, u32);
    fn column(&mut self, action: u32) -> Result<()>;
    fn dimensions_row(&self) -> (u32, u32);
    fn row(&mut self, action: u32) -> Result<()>;

    fn resize_all(&mut self, width: u32, height: u32) -> Result<()>;
    fn resize_all_to_max(&mut self) -> Result<(u32, u32)>;
    fn encode(&mut self) -> Result<Vec<u8>>;
    fn from_webp_animation(data: &[u8], action: u32) -> Result<Self>;
}

impl Frames for Vec<Frame> {
    fn clone_action(&self, action: i32, current_action: u32) -> Vec<Frame> {
        let action = self.get_action_from_relative(action);

        self.iter()
            .filter(|frame| frame.action == action)
            .map(|f| Frame {
                action: current_action,
                ..f.clone()
            })
            .collect()
    }

    fn get_mut_action(&mut self, action: i32) -> Vec<&mut Frame> {
        let action = self.get_action_from_relative(action);

        self.iter_mut()
            .filter(|frame| frame.action == action)
            .collect()
    }

    fn extract_action(&mut self, action: i32) -> Vec<Frame> {
        let action = self.get_action_from_relative(action);

        self.extract_if(.., |f| f.action == action).collect()
    }

    fn get_action_from_relative(&self, action: i32) -> u32 {
        if action < 0 {
            self.iter()
                .map(|f| f.action)
                .rev()
                .dedup()
                .nth(action.unsigned_abs() as usize - 1)
                .unwrap_or(0)
        } else {
            self.iter()
                .map(|f| f.action)
                .dedup()
                .nth(action as usize)
                .unwrap_or(0)
        }
    }

    fn get_at_timestamp(&mut self, timestamp: u32) -> Option<&mut Frame> {
        let mut ts = 0;
        for frame in self {
            ts += frame.delay as u32;
            if ts > timestamp {
                return Some(frame);
            }
        }
        None
    }

    fn duration(&self) -> u32 {
        self.iter().map(|f| f.delay as u32).sum()
    }

    fn min_delay(&self) -> u32 {
        self.iter().map(|f| f.delay).min().unwrap_or(0) as u32
    }

    fn dimensions(&self) -> (u32, u32) {
        self.iter()
            .map(|frame| frame.image.dimensions())
            .fold((0, 0), |acc, e| (acc.0.max(e.0), acc.1.max(e.1)))
    }

    fn dimensions_column(&self) -> (u32, u32) {
        self.iter()
            .map(|frame| frame.image.dimensions())
            .fold((0, 0), |acc, e| (acc.0.max(e.0), acc.1 + e.1))
    }
    fn dimensions_row(&self) -> (u32, u32) {
        self.iter()
            .map(|frame| frame.image.dimensions())
            .fold((0, 0), |acc, e| (acc.0 + e.0, acc.1.max(e.1)))
    }

    fn column(&mut self, action: u32) -> Result<()> {
        let (width, height) = self.dimensions_column();
        let mut canvas =
            DynamicImage::ImageRgba8(ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0])));
        let mut y = 0;

        for frame in self.iter() {
            let (w, h) = frame.image.dimensions();
            canvas.copy_from(&frame.image, (width - w) / 2, y)?;
            y += h;
        }

        self.clear();
        self.push(Frame::new(canvas, 1000, action));
        Ok(())
    }

    fn row(&mut self, action: u32) -> Result<()> {
        let (width, height) = self.dimensions_row();
        let mut canvas =
            DynamicImage::ImageRgba8(ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0])));
        let mut x = 0;

        for frame in self.iter() {
            let (w, h) = frame.image.dimensions();
            canvas.copy_from(&frame.image, x, (height - h) / 2)?;
            x += w;
        }

        self.clear();
        self.push(Frame::new(canvas, 1000, action));
        Ok(())
    }

    fn resize_all(&mut self, width: u32, height: u32) -> Result<()> {
        if width > MAX_SIZE {
            bail!("TOO FAT! {width}/{MAX_SIZE}");
        } else if height > MAX_SIZE {
            bail!("HEIGHT FAT! {height}/{MAX_SIZE}");
        }

        for frame in self.iter_mut() {
            frame.resize(width, height)?;
        }
        Ok(())
    }

    fn resize_all_to_max(&mut self) -> Result<(u32, u32)> {
        let (width, height) = self.dimensions();

        self.resize_all(width, height)?;
        Ok((width, height))
    }

    fn encode(&mut self) -> Result<Vec<u8>> {
        if self.is_empty() {
            bail!("No images to encode.".to_string());
        }
        let (width, height) = self.resize_all_to_max()?;

        let mut encoder = AnimationEncoder::new(width, height)?;
        encoder.set_method(0);
        encoder.set_quality(75.0);
        encoder.set_low_memory(true);

        let mut timestamp: i32 = 0;
        for frame in self {
            encoder.add_frame_rgba(&frame.image.to_rgba8().into_raw(), timestamp)?;
            timestamp += frame.delay;
        }

        Ok(encoder.finish(timestamp)?)
    }

    fn from_webp_animation(data: &[u8], action: u32) -> Result<Self> {
        let mut decoder = AnimationDecoder::new(data)?;
        let info = decoder.info();
        let (width, height) = (info.width, info.height);
        let decoded = decoder.decode_all()?;

        let is_single = decoded.len() <= 1;

        decoded
            .into_iter()
            .map(|frame| {
                let image: DynamicImage = RgbaImage::from_raw(width, height, frame.data)
                    .ok_or(anyhow!("Failed creating from raw RGBA"))?
                    .into();

                let delay = if is_single {
                    1000
                } else {
                    frame.duration_ms as i32
                };
                Ok(Frame::new(image, delay, action))
            })
            .collect()
    }
}
