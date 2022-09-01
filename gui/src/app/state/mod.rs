mod settings;

use std::sync::Arc;

use iced::pure::{column, Element};
use iced::{widget::qr_code, Command, Subscription};

use super::{cache::Cache, error::Error, menu::Menu, message::Message, view};

pub use settings::SettingsState;

use crate::daemon::Daemon;

pub trait State {
    fn view<'a>(&'a self, cache: &'a Cache) -> Element<'a, view::Message>;
    fn update(
        &mut self,
        daemon: Arc<dyn Daemon + Sync + Send>,
        cache: &Cache,
        message: Message,
    ) -> Command<Message>;
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
    fn load(&self, _daemon: Arc<dyn Daemon + Sync + Send>) -> Command<Message> {
        Command::none()
    }
}

pub struct Home {}

impl State for Home {
    fn view<'a>(&self, _cache: &'a Cache) -> Element<'a, view::Message> {
        view::dashboard(&Menu::Home, None, column())
    }
    fn update(
        &mut self,
        _daemon: Arc<dyn Daemon + Sync + Send>,
        _cache: &Cache,
        _message: Message,
    ) -> Command<Message> {
        Command::none()
    }
}

impl From<Home> for Box<dyn State> {
    fn from(s: Home) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Default)]
pub struct ReceivePanel {
    address: Option<bitcoin::Address>,
    qr_code: Option<qr_code::State>,
    warning: Option<Error>,
}

impl State for ReceivePanel {
    fn view<'a>(&'a self, _cache: &'a Cache) -> Element<'a, view::Message> {
        if let Some(address) = &self.address {
            view::dashboard(
                &Menu::Receive,
                self.warning.as_ref(),
                view::receive::receive(address, self.qr_code.as_ref().unwrap()),
            )
        } else {
            view::dashboard(&Menu::Receive, self.warning.as_ref(), column())
        }
    }
    fn update(
        &mut self,
        _daemon: Arc<dyn Daemon + Sync + Send>,
        _cache: &Cache,
        message: Message,
    ) -> Command<Message> {
        if let Message::ReceiveAddress(res) = message {
            match res {
                Ok(address) => {
                    self.warning = None;
                    self.qr_code = Some(qr_code::State::new(&address.to_qr_uri()).unwrap());
                    self.address = Some(address);
                }
                Err(e) => self.warning = Some(e),
            }
        };
        Command::none()
    }

    fn load(&self, daemon: Arc<dyn Daemon + Sync + Send>) -> Command<Message> {
        let daemon = daemon.clone();
        Command::perform(
            async move {
                daemon
                    .get_new_address()
                    .map(|res| res.address)
                    .map_err(|e| e.into())
            },
            Message::ReceiveAddress,
        )
    }
}

impl From<ReceivePanel> for Box<dyn State> {
    fn from(s: ReceivePanel) -> Box<dyn State> {
        Box::new(s)
    }
}
