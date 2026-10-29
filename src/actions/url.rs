use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};
use reqwest::Client;

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::{Frame, Frames}, parser::ParsableAction,
};

#[derive(Clone,Parsable,ParsableAction)]
pub struct URLAction(String);

#[distributed_slice(PARSERS)]
static PARSER: Parser = URLAction::parse_action;

impl Action for URLAction {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let client = Client::builder().user_agent("PFP/1.0").build()?;
            let bytes = client
                .get(&self.0)
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .map(|r| r.bytes())?
                .await?;

            let frames = if bytes.starts_with(b"RIFF") {
                Vec::from_webp_animation(&bytes, action)
            } else {
                Vec::from_unknown_data(&bytes, action)
            }?;

            images.extend(frames);
            Ok(())
        })
    }
}

