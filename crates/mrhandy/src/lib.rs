use parking_lot::{RwLock, RwLockReadGuard};
pub use serenity::{
    self,
    model::guild::{Guild, Role},
};
use serenity::{client::bridge::gateway::GatewayIntents, model::guild::Member};
use serenity::{
    client::{bridge::gateway::ShardManager, Client},
    framework::standard::{
        macros::{command, group, hook},
        CommandError, CommandResult, DispatchError, StandardFramework,
    },
    model::prelude::{Message, UserId},
    prelude::{Context, EventHandler, Mutex, TypeMapKey},
    CacheAndHttp,
};
use std::{collections::HashMap, env, future::Future, sync::Arc, thread::JoinHandle};

#[group]
#[commands(private)]
struct General;

struct Handler;

impl EventHandler for Handler {}

struct MainGuild;
impl TypeMapKey for MainGuild {
    type Value = u64;
}

#[derive(Clone)]
pub struct MrHandy {
    pub cache_and_http: Arc<CacheAndHttp>,
    pub shard_manager: Arc<Mutex<ShardManager>>,
    pub main_guild_id: u64,
}

impl MrHandy {
    /*pub async fn shutdown(&self) {
        let mut guard = self.shard_manager.lock().await;
        guard.shutdown_all();
    }*/
    pub async fn with_guild_member<O, F: Fn(&Guild, &Member) -> O>(
        &self,
        user_id: u64,
        fun: F,
    ) -> Result<O, &'static str> {
        self.with_guild(|guild| match guild {
            Some(guild) => {
                let member = guild.members.get(&user_id.into()).ok_or("Member is None")?;
                Ok(fun(guild, member))
            }
            None => Err("MainGuild isn't in cache."),
        })
        .await
    }

    pub async fn with_guild<O, F: FnOnce(Option<&Guild>) -> O>(&self, fun: F) -> O {
        let cache = &self.cache_and_http.cache;
        let mut fun = Some(fun);
        cache
            .guild_field(self.main_guild_id, |guild| {
                (fun.take().unwrap())(Some(guild))
            })
            .await
            .unwrap_or_else(|| (fun.take().unwrap())(None))
    }

    pub async fn clone_members(&self) -> Option<Members> {
        self.with_guild(move |guild| {
            guild.map(|guild| Members {
                members: guild.members.clone(),
            })
        })
        .await
    }

    pub async fn send_message(&self, channel: String, text: String) -> Result<(), Error> {
        let channel_id = self
            .with_guild(move |guild| {
                let guild = guild.ok_or(Error::NoMainGuild)?;
                let channel = guild
                    .channels
                    .values()
                    .find(|ch| &ch.name == &channel)
                    .ok_or_else(|| Error::ChannelNotFound(channel))?;
                Ok(channel.id)
            })
            .await?;
        let _ = channel_id
            .say(&self.cache_and_http.http, text)
            .await
            .map_err(Error::Serenity)?;
        Ok(())
    }

    pub fn get_roles<O, F: Fn(&Role) -> O>(guild: &Guild, member: &Member, fun: F) -> Vec<O> {
        member
            .roles
            .iter()
            .filter_map(|role_id| guild.roles.get(role_id))
            .map(fun)
            .collect()
    }

    pub fn get_name_nick(member: &Member) -> (String, Option<String>) {
        let user = &member.user;
        (user.name.clone(), member.nick.clone())
    }
}

pub enum Error {
    NoMainGuild,
    ChannelNotFound(String),
    Serenity(serenity::Error),
}

/*impl Drop for MrHandy {
    fn drop(&mut self) {
        self.shutdown();
    }
}*/

pub struct Members {
    members: HashMap<UserId, Member>,
}
impl Members {
    pub fn get(&self, user_id: u64) -> Option<&Member> {
        self.members.get(&user_id.into())
    }
}

#[hook]
async fn dispatch_error_hook(_context: &Context, _msg: &Message, error: DispatchError) {
    eprintln!("DispatchError: {:?}", error)
}

#[hook]
async fn after_hook(_: &Context, _: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        println!("Error in {}: {:?}", cmd_name, why);
    }
}

pub async fn init(token: &str, main_guild_id: u64) -> (MrHandy, Client) {
    // Login with a bot token from the environment
    //let token = &env::var("DISCORD_TOKEN").expect("token");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .on_dispatch_error(dispatch_error_hook)
        .group(&GENERAL_GROUP)
        .after(after_hook);
    let client = Client::builder(token)
        .intents(GatewayIntents::all())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
    /*{
        let mut data = client.data.write().await;
        data.insert::<MainGuild>(main_guild_id);
    }*/
    let cache_and_http = Arc::clone(&client.cache_and_http);
    let shard_manager = Arc::clone(&client.shard_manager);

    (
        MrHandy {
            cache_and_http,
            shard_manager,
            main_guild_id,
        },
        client,
    )
}

#[command]
async fn private(ctx: &Context, msg: &Message) -> CommandResult {
    msg.author.dm(ctx, |msg| msg.content(":eyes:")).await?;
    Ok(())
}
/*
#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!")?;
    Ok(())
}

#[command]
fn roles(ctx: &mut Context, msg: &Message) -> CommandResult {
    let who: Result<u64, _> = msg.content["~roles".len()..].trim().parse();
    let (name, roles) = if let Ok(who) = who {
        println!("Getting roles from: {}", who);
        let data = ctx.data.read();
        let main_guild = data.get::<MainGuild>().ok_or("Can get MainGuild id")?;

        let cache = ctx.cache.read();
        let guild = cache
            .guild(*main_guild)
            .ok_or("MainGuild isn't in cache.")?;
        let guild = guild.read();
        guild
            .members
            .get(&UserId(who))
            .map(|member| (member.display_name().into_owned(), member.roles.clone()))
    } else {
        println!("Getting roles from self");
        msg.member(&ctx)
            .map(|member| (member.display_name().into_owned(), member.roles.clone()))
    }
    .ok_or("Member is None")?;

    //let roles = member.roles(&ctx).ok_or("Roles are None")?;
    //let roles: Vec<_> = roles.into_iter().map(|role| role).collect();
    let roles_string = format!("Name: {}, Roles: {:#?}", name, &roles);
    msg.reply(&ctx, roles_string)?;
    Ok(())
}
*/
