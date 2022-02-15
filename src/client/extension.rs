use crate::client::EventHandler;

pub trait Extension {
    fn event_handler(&self, event_handler: EventHandler) -> EventHandler;
}
