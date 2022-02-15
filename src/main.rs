use jumia::client::{EventHandler, Extension};
use jumia::Client;
use serenity::client::Context;
use serenity::model::channel::Message;

struct TestExtension;

async fn bleh(ctx: &Context, message: &Message) {
    if message.content == "!extension" {
        message.reply(ctx, "Hey!").await.unwrap();
    }
}

impl Extension for TestExtension {
    fn event_handler(&self, handler: EventHandler) -> EventHandler {
        handler.message(|ctx, message| Box::pin(bleh(ctx, message)))
    }
}

#[tokio::main]
async fn main() {
    Client::builder()
        .token(std::env::var("DISCORD_TOKEN").unwrap())
        .event_handler(|handler| {
            handler.ready(|_, _| {
                Box::pin(async {
                    println!("Ready from closure");
                })
            })
        })
        .extension(TestExtension)
        .await
        .unwrap()
        .start()
        .await
        .unwrap();
}
