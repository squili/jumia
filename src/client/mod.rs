use serenity::futures::future::BoxFuture;
use serenity::http::Http;
use serenity::model::id::ApplicationId;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

mod extension;
mod handler;

pub use extension::Extension;
pub use handler::EventHandler;

pub struct Client {
    pub serenity: serenity::Client,
}

impl Client {
    pub fn builder<'a>() -> ClientBuilder<'a> {
        ClientBuilder::new()
    }

    pub async fn start(mut self) -> serenity::Result<()> {
        self.serenity.start_autosharded().await
    }
}

#[derive(Error, Debug)]
pub enum ClientBuilderError {
    #[error("serenity error: `{0:?}`")]
    SerenityError(#[from] serenity::Error),
}

#[derive(Default)]
pub struct ClientBuilder<'a> {
    token: Option<String>,
    application_id: Option<ApplicationId>,
    event_handler: Option<EventHandler>,
    fut: Option<BoxFuture<'a, Result<Client, ClientBuilderError>>>,
}

impl<'a> ClientBuilder<'a> {
    pub fn new() -> Self {
        Self {
            event_handler: Some(Default::default()),
            ..Default::default()
        }
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn application_id(mut self, id: impl Into<ApplicationId>) -> Self {
        self.application_id = Some(id.into());
        self
    }

    pub fn event_handler<F>(mut self, callback: F) -> Self
    where
        F: Fn(EventHandler) -> EventHandler,
    {
        self.event_handler = self.event_handler.map(callback);
        self
    }

    pub fn extension<E: Extension>(mut self, extension: E) -> Self {
        self.event_handler = self
            .event_handler
            .map(|handler| extension.event_handler(handler));
        self
    }
}

impl<'a> Future for ClientBuilder<'a> {
    type Output = Result<Client, ClientBuilderError>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let token = self.token.take().expect("missing token");
            let http = Http::new_with_token(&token);
            let mut builder =
                serenity::Client::builder(&token).event_handler(self.event_handler.take().unwrap());
            let application_id = self.application_id;

            self.fut = Some(Box::pin(async move {
                let app_info = http.get_current_application_info().await?;

                builder =
                    builder.application_id(application_id.map(|id| id.0).unwrap_or(app_info.id.0));

                match builder.await {
                    Ok(client) => Ok(Client { serenity: client }),
                    Err(err) => Err(err.into()),
                }
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}
