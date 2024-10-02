use crate::core::{Variables, WorkerMessage};
use ngrok::config::TunnelBuilder;
use ngrok::prelude::{TunnelExt, UrlTunnel};
use reqwest::blocking::Client;
use roboplc::controller::{Context, WResult, Worker};
use roboplc::event_matches;
use roboplc_derive::WorkerOpts;
use std::fmt::Debug;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::macros::BotCommands;
use teloxide::prelude::{ChatId, Message, Request, Requester, ResponseResult, Update};
use teloxide::types::InputFile;
use teloxide::utils::command::BotCommands as UtilsBotCommands;
use teloxide::{dptree, Bot};
use tokio::net::ToSocketAddrs;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};

#[derive(WorkerOpts)]
#[worker_opts(cpu = 2, priority = 80, scheduling = "fifo", blocking = true)]
pub struct BotWorker {}

impl Worker<WorkerMessage, Variables> for BotWorker {
    fn run(&mut self, context: &Context<WorkerMessage, Variables>) -> WResult {
        //@todo find a other way to check internet connection
        while !check_internet_connection() {
            info!("No internet connection. Retrying in 5 seconds...");
            sleep(Duration::from_secs(5));
        }

        info!("Internet connection established. Starting bot...");
        let result = Runtime::new().unwrap().block_on(bot(context));
        Ok(result)
    }
}

fn check_internet_connection() -> bool {
    info!("Checking internet connection...");
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|e| {
            error!("Failed to build client: {:?}", e);
            panic!("Could not build HTTP client");
        });

    match client.get("https://www.google.com").send() {
        Ok(response) => response.status().is_success(),
        Err(e) => {
            warn!("Failed to connect: {:?}", e);
            false
        }
    }
}


pub fn run_ngrok(
    auth_token: String,
    domain: String,
    forward_to: impl ToSocketAddrs + Send + Debug + 'static,
) -> oneshot::Sender<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        listen_ngrok(auth_token, domain, forward_to, shutdown_rx).await;
    });

    shutdown_tx
}

async fn listen_ngrok(
    auth_token: impl Into<String>,
    domain: impl Into<String>,
    forward_to: impl ToSocketAddrs + Send + Debug,
    shutdown_rx: oneshot::Receiver<()>,
) {
    let session = ngrok::Session::builder().authtoken(auth_token).connect().await.unwrap();
    let mut tunnel = session.http_endpoint().compression().domain(domain).listen().await.unwrap();

    info!("Ngrok trying to forward tcp");
    let url = tunnel.url().to_string();
    info!("Ngrok forwarding to: {:?}", forward_to);
    let tunnel = tunnel.forward_tcp(forward_to);
    info!("Ngrok tunnel established at: {}", url);

    tokio::select! {
        _ = tunnel => {
            info!("Ngrok tunnel closed");
        }
        _ = shutdown_rx => {
            info!("Received shutdown signal, closing ngrok tunnel");
        }
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "List commands.")]
    Help,
    #[command(description = "Get a photo from the camera.")]
    Photo,
    #[command(description = "Get a URL with video stream.")]
    GetVideo,
    #[command(description = "Stop video stream.")]
    StopVideo,
}

/// Bot initialization and command handling
async fn bot(context: &Context<WorkerMessage, Variables>) {
    let telegram_config = &context.variables().telegram_config;
    let bot = Bot::new(&telegram_config.token);

    for command in Command::bot_commands() {
        info!("Registered command: {:?}", command);
    }

    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("Failed to set bot commands");

    let handler = Update::filter_message()
        .branch(dptree::entry().filter_command::<Command>().endpoint(command_handler))
        .branch(
            dptree::filter(|msg: Message| !msg.text().map_or(false, |text| text.starts_with('/')))
                .endpoint(invalid_command_handler),
        );

    bot.send_message(ChatId(telegram_config.admin_user_id), "Bot started.")
        .send()
        .await
        .expect("Failed to send message to admin user");

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(context.clone())])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    context: Arc<Context<WorkerMessage, Variables>>,
) -> ResponseResult<()> {
    let hc = context
        .hub()
        .register("bot: command handler", event_matches!(WorkerMessage::Frame(_)))
        .unwrap();
    let allowed_user_ids = &context.variables().telegram_config.allowed_user_ids;
    match cmd {
        Command::Help => {
            let text = Command::descriptions().to_string();
            bot.send_message(msg.chat.id, text).await?;
        }
        Command::Photo => {
            info!("Received photo command from chat id: {:?}.", msg.chat.id);

            let user = msg.from.as_ref();
            if let Some(user) = user {
                info!("User: {:?}", user);
            }

            if should_notify_admin() && user.is_some() {
                let user = user.clone().unwrap();
                bot.send_message(
                    ChatId(context.variables().telegram_config.admin_user_id),
                    format!(
                        "Received photo command from chat id: {:?}. Username: {:?}",
                        user.id, user.username
                    ),
                )
                .await?;
            }

            let id = user.clone().unwrap().id.0 as i64;
            if allowed_user_ids.contains(&id) {
                info!(
                    "Received photo command from user: {:?}. Capturing photo...",
                    user.unwrap().username
                );

                // Wait for a frame from the hub
                if let Ok(WorkerMessage::Frame(frame_data)) = hc.try_recv() {
                    bot.send_photo(msg.chat.id, InputFile::memory(frame_data)).await?;
                } else {
                    bot.send_message(msg.chat.id, "Failed to capture photo. Please try again later.")
                        .await?;
                }
            } else {
                warn!("User not allowed to use this command. User id: {:?}", msg.chat.id);
                bot.send_message(msg.chat.id, "You are not allowed to use this command.")
                    .await?;
            }
        }
        Command::GetVideo => {
            info!("Received get_video command from chat id: {:?}.", msg.chat.id);

            let user = msg.from.as_ref();
            if should_notify_admin() {
                if let Some(user) = user {
                    bot.send_message(
                        ChatId(context.variables().telegram_config.admin_user_id),
                        format!(
                            "Received get_video command from chat id: {:?}. Username: {:?}",
                            user.id, user.username
                        ),
                    )
                    .await?;
                }
            }

            debug!("Sending video stream URL to chat id: {:?}", msg.chat.id);
            debug!("Ngrok is started: {:?}", &context.variables().is_ngrok_started);

            bot.send_message(
                msg.chat.id,
                format!("Video stream URL: https://{}/", &context.variables().ngrok_domain),
            )
            .await
            .expect("Could not send message");

            if *context.variables().is_ngrok_started.read() {
                debug!("Ngrok is already started.");

                bot.send_message(msg.chat.id, "Ngrok is already started.")
                    .await
                    .expect("Could not send message");
                return Ok(());
            }

            *context.variables().is_ngrok_started.write() = true;

            let ngrok_auth_token = context.variables().ngrok_auth_token.clone();
            let ngrok_domain = context.variables().ngrok_domain.clone();
            let server_address = context.variables().server_address.clone();
            let ngrok_tx: oneshot::Sender<()> = run_ngrok(ngrok_auth_token, ngrok_domain, server_address);
            {
                let mut ng_tx = context.variables().ngrok_shutdown_tx.lock().await;
                *ng_tx = Some(ngrok_tx);
            }
        }
        Command::StopVideo => {
            info!("Received stop_video command from chat id: {:?}.", msg.chat.id);

            let user = msg.from.as_ref();
            if should_notify_admin() {
                if let Some(user) = user {
                    bot.send_message(
                        ChatId(context.variables().telegram_config.admin_user_id),
                        format!(
                            "Received stop_video command from chat id: {:?}. Username: {:?}",
                            user.id, user.username
                        ),
                    )
                    .await?;
                }
            }
            debug!("Stopping video stream...");

            {
                let mut ngrok_shutdown = context.variables().ngrok_shutdown_tx.lock().await;

                if let Some(sender) = ngrok_shutdown.take() {
                    sender.send(()).expect("Failed to send shutdown signal to ngrok.");
                } else {
                    bot.send_message(msg.chat.id, "Video stream is not running.").await?;

                    return Ok(());
                }
            }

            *context.variables().is_ngrok_started.write() = false;
            bot.send_message(msg.chat.id, "Video stream stopped.").await?;
        }
    }
    Ok(())
}

async fn invalid_command_handler(bot: Bot, msg: Message, context: Arc<Context<WorkerMessage, Variables>>) -> ResponseResult<()> {
    info!("Received invalid command from chat id: {:?}.", msg.chat.id);
    let response = "You entered an invalid command. Please use /photo to request a photo or /getvideo to get a video stream URL.";
    bot.send_message(msg.chat.id, response).await?;

    notify_admin_about_invalid_command(&bot, &msg, context.variables().telegram_config.admin_user_id).await?;

    Ok(())
}

async fn notify_admin_about_invalid_command(bot: &Bot, msg: &Message, admin_id: i64) -> ResponseResult<()> {
    if should_notify_admin() {
        let admin_message = format!(
            "Invalid command received. User ID: {}, Username: {}, Command: {}",
            msg.chat.id,
            msg.chat.username().unwrap_or_default(),
            msg.text().unwrap_or("No text")
        );
        bot.send_message(ChatId(admin_id), admin_message).await?;
    }
    Ok(())
}

fn should_notify_admin() -> bool {
    // @todo implement logic here to determine if the admin should be notified about bot activities
    // For now, we'll always notify
    true
}
