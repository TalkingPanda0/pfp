use anyhow::Result;
use linkme::distributed_slice;

use crate::{AppState, frames::Frame};
use std::{any::Any, pin::Pin, sync::Arc};

pub type ActionResult<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
pub type Parser = fn(&str, &mut Vec<Box<dyn Action>>, &Arc<AppState>) -> bool;

#[distributed_slice]
pub static PARSERS: [Parser];

pub trait Action: Send + Sync + ActionClone {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a>;
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
    async fn apply_actions(&mut self, images: &mut Vec<Frame>) -> Result<()>;
}

impl ActionList for Vec<Box<dyn Action>> {
    async fn apply_actions(&mut self, images: &mut Vec<Frame>) -> Result<()> {
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
