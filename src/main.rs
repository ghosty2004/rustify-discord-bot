use dotenv_codegen::dotenv;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, prelude::*},
    prelude::*,
};
use songbird::{error::*, SerenityInit};

struct Handler;

// constants
const PREFIX: &str = "!";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut command_name = String::from(format!(
            "{PREFIX}{}",
            msg.content.split(" ").next().unwrap()
        ));
        if command_name.starts_with(PREFIX) {
            command_name = command_name.replace(PREFIX, "");

            let args = msg
                .content
                .split(" ")
                .enumerate()
                .filter(|(index, _)| *index != 0)
                .map(|(_, arg)| arg.to_string())
                .collect::<Vec<String>>();

            match command_name.as_str() {
                // ping
                "ping" => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                        println!("Error sending message: {:?}", why);
                    }
                }

                // channels
                "channels" => {
                    let channels = ctx
                        .http
                        .get_channels(msg.guild_id.unwrap().0)
                        .await
                        .unwrap();

                    if let Err(any) = msg
                        .channel_id
                        .send_message(&ctx.http, |m| {
                            m.embed(|e| {
                                e.color(0x00FF00).title("Guild channels").description(
                                    channels
                                        .iter()
                                        .map(|channel| format!("{}: {}", channel.id, channel.name))
                                        .collect::<Vec<String>>()
                                        .join("\n"),
                                )
                            })
                        })
                        .await
                    {
                        println!("Error sending message: {:?}", any);
                    }
                }

                // leave voice channel
                "leavevc" => {
                    let manager = songbird::get(&ctx)
                        .await
                        .expect("Songbird Voice client placed in at initialisation.")
                        .clone();

                    match manager.leave(msg.guild_id.unwrap().0).await {
                        Ok(_) => {
                            if let Err(any) = msg
                                .channel_id
                                .say(&ctx.http, "I left the voice channel !")
                                .await
                            {
                                println!("Error sending message: {:?}", any);
                            }
                        }
                        Err(err) => match err {
                            JoinError::NoCall => {
                                if let Err(any) = msg
                                    .channel_id
                                    .say(&ctx.http, "I'm not in a voice channel !")
                                    .await
                                {
                                    println!("Error sending message: {:?}", any);
                                }
                            }
                            _ => {}
                        },
                    }
                }

                // join voice channel
                "joinvc" => match args.as_slice() {
                    [channel_name] => {
                        let guild = ctx.http.get_guild(msg.guild_id.unwrap().0).await.unwrap();

                        let channels = guild.channels(&ctx.http).await.unwrap();
                        let channels = channels
                            .iter()
                            .map(|(_, channel)| channel)
                            .collect::<Vec<&GuildChannel>>();
                        let channels = channels
                            .iter()
                            .filter(|channel| channel.kind == ChannelType::Voice)
                            .collect::<Vec<&&GuildChannel>>();

                        let channel = channels
                            .iter()
                            .find(|channel| channel.name == *channel_name);

                        if let Some(channel) = channel {
                            let manager = songbird::get(&ctx)
                                .await
                                .expect("Songbird Voice client placed in at initialisation.")
                                .clone();

                            match manager.join(guild.id.0, channel.id.0).await.1 {
                                Ok(_) => {
                                    println!("Joined voice channel: {}", channel.name);
                                }
                                Err(any) => {
                                    println!("Error joining voice channel: {:?}", any);
                                }
                            };
                        } else {
                            if let Err(any) = msg
                                .channel_id
                                .say(&ctx.http, "Channel with this name not found !")
                                .await
                            {
                                println!("Error sending message: {:?}", any);
                            }
                        }
                    }
                    _ => {
                        if let Err(any) = msg
                            .channel_id
                            .say(&ctx.http, "Please specify the channel name")
                            .await
                        {
                            println!("Error sending message: {:?}", any);
                        }
                    }
                },

                // set activity
                "setactivity" => {
                    let activity = args.join(" ");
                    if activity.len() < 1 {
                        msg.channel_id
                            .say(&ctx.http, "Please specify the activity")
                            .await
                            .unwrap();
                    } else {
                        ctx.set_activity(Activity::playing(&activity)).await;
                        msg.channel_id
                            .say(&ctx.http, "Activity set !")
                            .await
                            .unwrap();
                    }
                }

                // if nothing else...
                _ => {}
            }
        }
    }

    async fn ready(&self, _ctx: Context, bot: Ready) {
        println!("{} is ready !", bot.user.name);
    }
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_TYPING
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(dotenv!("TOKEN"), intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
