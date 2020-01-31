pub use serenity::model::guild::Role;
use serenity::model::guild::{Guild, Member};
use serenity::{
    client::{bridge::gateway::ShardManager, Client},
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
    model::prelude::{Message, UserId},
    prelude::{Context, EventHandler, Mutex, TypeMapKey},
    CacheAndHttp,
};
use std::{env, sync::Arc, thread::JoinHandle};

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
    pub fn shutdown(&self) {
        let mut guard = self.shard_manager.lock();
        guard.shutdown_all();
    }
    pub fn with_guild_member<O, F: Fn(&Guild, &Member) -> O>(
        &self,
        user_id: u64,
        fun: F,
    ) -> Result<O, &'static str> {
        let mut cache = self.cache_and_http.cache.read();
        let guild = cache
            .guild(self.main_guild_id)
            .ok_or("MainGuild isn't in cache.")?;
        let guild = guild.read();
        let member = guild.members.get(&user_id.into()).ok_or("Member is None")?;
        Ok(fun(&*guild, &*member))
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
        let user = member.user.read();
        (user.name.clone(), member.nick.clone())
    }
}
impl Drop for MrHandy {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn start(token: &str, main_guild_id: u64) -> (MrHandy, JoinHandle<()>) {
    // Login with a bot token from the environment
    //let token = &env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::new(token, Handler).expect("Error creating client");
    {
        let mut data = client.data.write();
        data.insert::<MainGuild>(main_guild_id);
    }
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
            .on_dispatch_error(|_ctx, _msg, err| eprintln!("DispatchError: {:?}", err))
            .group(&GENERAL_GROUP)
            .after(|_ctx, _msg, cmd_name, error| {
                if let Err(why) = error {
                    println!("Error in {}: {:?}", cmd_name, why);
                }
            }),
    );
    let cache_and_http = Arc::clone(&client.cache_and_http);
    let shard_manager = Arc::clone(&client.shard_manager);

    let handle = std::thread::spawn(move || {
        // start listening for events by starting a single shard
        if let Err(why) = client.start() {
            println!("An error occurred while running the client: {:?}", why);
        }
    });
    (
        MrHandy {
            cache_and_http,
            shard_manager,
            main_guild_id,
        },
        handle,
    )
}

#[command]
fn private(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.author.dm(ctx, |msg| msg.content(":eyes:"))?;
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
