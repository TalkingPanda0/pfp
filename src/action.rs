use crate::{
    AppState,
    actions::{
        aliases::Aliases, animate::Animate, column::Column, combine::Combine, copy::Copy,
        delay::Delay, discordpfp::DiscordPFPAction, grayscale::Grayscale,
        mirror::Mirror, reverse::Reverse, row::Row, squish::Squish, tenor::Tenor, text::TextAction,
        times::Times,
    },
    frames::Frame,
};
use std::{any::Any, pin::Pin, sync::Arc};

pub type ActionResult<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;
pub type Parser = fn(&str, &mut Vec<Box<dyn Action>>, &Arc<AppState>) -> bool;

static PARSERS: [Parser; 15] = [
    Grayscale::parse,
    DiscordPFPAction::parse,
    Squish::parse,
    Row::parse,
    Column::parse,
    Copy::parse,
    Times::parse,
    Tenor::parse,
    Combine::parse,
    TextAction::parse,
    Delay::parse,
    Aliases::parse,
    Reverse::parse,
    Animate::parse,
    Mirror::parse,
];

pub trait Action: Send + Sync + ActionClone {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a>;
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, discord: &Arc<AppState>) -> bool
    where
        Self: Sized;
    fn as_any(&self) -> &dyn Any;
}

pub trait ActionClone {
    fn clone_box(&self) -> Box<dyn Action>;
}

impl<T> ActionClone for T
where
    T: 'static + Action + Clone,
{
    fn clone_box(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Action> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait ActionList {
    async fn apply_actions(&mut self, images: &mut Vec<Frame>) -> Result<(), String>;
}

impl ActionList for Vec<Box<dyn Action>> {
    async fn apply_actions(&mut self, images: &mut Vec<Frame>) -> Result<(), String> {
        for (i, action) in self.iter().enumerate() {
            action.apply(images, i as u32).await?;
        }
        Ok(())
    }
}

pub fn parse_actions(inputs: &[&str], state: &Arc<AppState>) -> Vec<Box<dyn Action>> {
    let mut actions: Vec<Box<dyn Action>> = Vec::with_capacity(inputs.len());
    for input in inputs {
        for parser in PARSERS {
            if parser(input, &mut actions, state) {
                break;
            }
        }
    }
    actions
}
