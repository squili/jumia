use serde_json::Value;
use serenity::async_trait;
use serenity::client::bridge::gateway::event::ShardStageUpdateEvent;
use serenity::client::{Context, EventHandler as SerenityEventHandler};
use serenity::model::channel::{
    Channel, ChannelCategory, GuildChannel, Message, PartialGuildChannel, Reaction, StageInstance,
};
use serenity::model::event::{
    ChannelPinsUpdateEvent, GuildMembersChunkEvent, InviteCreateEvent, InviteDeleteEvent,
    MessageUpdateEvent, PresenceUpdateEvent, ResumedEvent, ThreadListSyncEvent,
    ThreadMembersUpdateEvent, TypingStartEvent, VoiceServerUpdateEvent,
};
use serenity::model::gateway::Ready;
use serenity::model::guild::{
    Emoji, Guild, GuildUnavailable, Integration, Member, PartialGuild, Role, ThreadMember,
};
use serenity::model::id::{
    ApplicationId, ChannelId, EmojiId, GuildId, IntegrationId, MessageId, RoleId,
};
use serenity::model::interactions::Interaction;
use serenity::model::prelude::{CurrentUser, User, VoiceState};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

macro_rules! define_event_handler {
    ($($event_name: ident ($($arg_name: ident : $event_type: ty as $trait_type: ty),*))*) => {
        #[derive(Default)]
        pub struct EventHandler {
            $(
                $event_name: Vec<Box<dyn Fn(&Context, $($trait_type),*) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
            )*
            unknown: HashMap<String, Vec<Box<dyn Fn(&Context, &Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>>,
        }

        #[async_trait]
        impl SerenityEventHandler for EventHandler {
            $(
                async fn $event_name(&self, ctx: Context, $($arg_name: $event_type),*) {
                    for callback in &self.$event_name {
                        callback(&ctx, $(&$arg_name),*).await;
                    }
                }
            )*

            async fn unknown(&self, ctx: Context, name: String, raw: Value) {
                if let Some(callbacks) = self.unknown.get(&name) {
                    for callback in callbacks {
                        callback(&ctx, &raw).await;
                    }
                }
            }
        }

        impl EventHandler {
            $(
                #[allow(unused)]
                pub fn $event_name<F>(mut self, callback: F) -> Self
                where
                    F: 'static + Fn(&Context, $($trait_type),*) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync
                {
                    self.$event_name.push(Box::new(callback));
                    self
                }
            )*
        }
    };
}

define_event_handler! {
    channel_create(channel: &GuildChannel as &GuildChannel)
    category_create(channel: &ChannelCategory as &ChannelCategory)
    category_delete(channel: &ChannelCategory as &ChannelCategory)
    channel_delete(channel: &GuildChannel as &GuildChannel)
    channel_pins_update(channel: ChannelPinsUpdateEvent as &ChannelPinsUpdateEvent)
    channel_update(old: Option<Channel> as &Option<Channel>, new: Channel as &Channel)
    guild_ban_addition(guild_id: GuildId as &GuildId, banned_user: User as &User)
    guild_ban_removal(guild_id: GuildId as &GuildId, unbanned_user: User as &User)
    guild_create(guild: Guild as &Guild, is_new: bool as &bool)
    guild_delete(incomplete: GuildUnavailable as &GuildUnavailable, full: Option<Guild> as &Option<Guild>)
    guild_emojis_update(guild_id: GuildId as &GuildId, current_state: HashMap<EmojiId, Emoji> as &HashMap<EmojiId, Emoji>)
    guild_integrations_update(guild_id: GuildId as &GuildId)
    guild_member_addition(guild_id: GuildId as &GuildId, new_member: Member as &Member)
    guild_member_removal(guild_id: GuildId as &GuildId, user: User as &User, member: Option<Member> as &Option<Member>)
    guild_member_update(old: Option<Member> as &Option<Member>, new: Member as &Member)
    guild_members_chunk(chunk: GuildMembersChunkEvent as &GuildMembersChunkEvent)
    guild_role_create(guild_id: GuildId as &GuildId, new: Role as &Role)
    guild_role_delete(guild_id: GuildId as &GuildId, id: RoleId as &RoleId, role: Option<Role> as &Option<Role>)
    guild_role_update(guild_id: GuildId as &GuildId, old: Option<Role> as &Option<Role>, new: Role as &Role)
    guild_unavailable(guild_id: GuildId as &GuildId)
    guild_update(old: Option<Guild> as &Option<Guild>, new: PartialGuild as &PartialGuild)
    invite_create(data: InviteCreateEvent as &InviteCreateEvent)
    invite_delete(data: InviteDeleteEvent as &InviteDeleteEvent)
    message(message: Message as &Message)
    message_delete(channel_id: ChannelId as &ChannelId, id: MessageId as &MessageId, guild_id: Option<GuildId> as &Option<GuildId>)
    message_delete_bulk(channel_id: ChannelId as &ChannelId, ids: Vec<MessageId> as &Vec<MessageId>, guild_id: Option<GuildId> as &Option<GuildId>)
    message_update(old: Option<Message> as &Option<Message>, new: Option<Message> as &Option<Message>, event: MessageUpdateEvent as &MessageUpdateEvent)
    reaction_add(reaction: Reaction as &Reaction)
    reaction_remove(reaction: Reaction as &Reaction)
    reaction_remove_all(channel_id: ChannelId as &ChannelId, id: MessageId as &MessageId)
    presence_update(data: PresenceUpdateEvent as &PresenceUpdateEvent)
    ready(data: Ready as &Ready)
    resume(data: ResumedEvent as &ResumedEvent)
    shard_stage_update(data: ShardStageUpdateEvent as &ShardStageUpdateEvent)
    typing_start(data: TypingStartEvent as &TypingStartEvent)
    user_update(old: CurrentUser as &CurrentUser, new: CurrentUser as &CurrentUser)
    voice_server_update(data: VoiceServerUpdateEvent as &VoiceServerUpdateEvent)
    voice_state_update(guild_id: Option<GuildId> as &Option<GuildId>, old: Option<VoiceState> as &Option<VoiceState>, new: VoiceState as &VoiceState)
    webhook_update(guild_id: GuildId as &GuildId, channel_id: ChannelId as &ChannelId)
    interaction_create(interaction: Interaction as &Interaction)
    integration_create(integration: Integration as &Integration)
    integration_update(integration: Integration as &Integration)
    integration_delete(integration_id: IntegrationId as &IntegrationId, guild_id: GuildId as &GuildId, application_id: Option<ApplicationId> as &Option<ApplicationId>)
    stage_instance_create(stage_instance: StageInstance as &StageInstance)
    stage_instance_update(stage_instance: StageInstance as &StageInstance)
    stage_instance_delete(stage_instance: StageInstance as &StageInstance)
    thread_create(thread: GuildChannel as &GuildChannel)
    thread_update(thread: GuildChannel as &GuildChannel)
    thread_delete(thread: PartialGuildChannel as &PartialGuildChannel)
    thread_list_sync(data: ThreadListSyncEvent as &ThreadListSyncEvent)
    thread_member_update(thread_member: ThreadMember as &ThreadMember)
    thread_members_update(thread_members_update: ThreadMembersUpdateEvent as &ThreadMembersUpdateEvent)
}

impl EventHandler {
    pub fn unknown<
        T: Into<String>,
        F: 'static + Fn(&Context, &Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
    >(
        mut self,
        name: T,
        callback: F,
    ) -> Self {
        self.unknown
            .entry(name.into())
            .or_default()
            .push(Box::new(callback));
        self
    }
}
