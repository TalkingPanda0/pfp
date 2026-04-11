use std::sync::Arc;
use crate::AppState;
use crate::action::ActionResult;
use crate::cache::Cache;
use crate::{action::Action, discord::DiscordPFP, frames::Frame};

#[derive(Clone)]
pub struct DiscordPFPAction(pub u64, pub String);

impl Action for DiscordPFPAction {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, state: &Arc<AppState>) -> bool {

        let Some(id) = input.strip_prefix("<@").and_then(|s| s.strip_suffix(">")).and_then(|s| s.parse().ok()) else {
            return false;
        };

        let Some(avatar) = actions.iter().find_map(|action| {
            let action = action.as_any().downcast_ref::<DiscordPFPAction>()?;
            if action.0 == id {
                Some(action.1.clone())
            }  else {
                None
            }
        }).or_else(|| state.discord.get_avatar(id).ok()) else {
            return false;
        };

        actions.push(Box::new(DiscordPFPAction(id, avatar)));
        true
    }

    fn apply<'a>(
        &'a self,
        images: &'a mut Vec<Frame>,
        action: u32
    ) -> ActionResult<'a> {
        Box::pin(async move {
            let pfp_bytes = match Cache::get(&self.1).await {
                Some(pfp) => pfp,
                None => {
                    let pfp = DiscordPFP::fetch_pfp(self.0, &self.1)
                        .await
                        .map_err(|err| err.to_string())?;
                    let _ = Cache::save(&pfp, &self.1)
                        .await
                        .inspect_err(|err| eprintln!("Failed saving cache: {:?}", err));
                    pfp
                }
            };

            images.push(Frame::from_webp(&pfp_bytes, 1000,action)?);
            Ok(())
        })
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

