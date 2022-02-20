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
    ($($tt:tt)*) => {
        define_event_handler_impl!(@duplicate = {
            events = [$($tt)*]
        });
    };
}

macro_rules! define_event_handler_impl {
    // =========================================================================
    // Duplicate the tokens so we have a reference copy to verify that
    // $ref == "@ref".
    // =========================================================================
    (@duplicate = {
        events = [$($name:ident($($(@$ref:ident)? $arg:ident: $type:ty),* $(,)?))*]
    }) => {
        define_event_handler_impl!(@verify = {
            events = [$($name($($(@$ref)? $arg: $type),*))*]
            events_duplicate = [$($name($($(@$ref)? $arg: $type),*))*]
        });
    };
    // =========================================================================
    // Verify that $@ref == "@ref".
    // =========================================================================
    (@verify = {
        events = [$(
            $name:ident($(
                $(@$ref:ident)?
                $arg:ident:
                $type:ty
            ),* $(,)?)
        )*]
        events_duplicate = [$(
            $duplicate_name:ident($(
                $(@ref)?
                $duplicate_arg:ident:
                $duplicate_type:ty
            ),* $(,)?)
        )*]
    }) => {
        define_event_handler_impl!(@collect_events = {
            events = [ ]
            rest = [$($name($($(@$ref)? $arg: $type),*))*]
        });
    };
    // =========================================================================
    // Collect the events into a structured format that can be emitted easily
    // =========================================================================
    (@collect_events = {
        events = [$({
            name = $en:ident // event name
            ref = $eref:literal // event (uses) ref
            args = [$(
                // event arg name
                // event arg original type
                // event arg referenced type
                ($ean:ident: $eaot:ty => $eart:ty)
            )*]
        })*]
        rest = [
            $en_:ident($( // event name
                $(@$earef_:ident)? // event arg ref
                $ean_:ident: // event arg name
                $eat_:ty // event arg type
            ),* $(,)?)
            $($rest:tt)*
        ]
    }) => {
        define_event_handler_impl!(@collect_args = {
            super = {
                events = [$({
                    name = $en
                    ref = $eref
                    args = [$(($ean: $eaot => $eart))*]
                })*]
                rest = [ $($rest)* ]
            }
            name = $en_
            ref = false
            args = [ ]
            rest = [ $(($(@$earef_)? $ean_: $eat_))* ]
        });
    };
    (@collect_args = {
        super = {
            events = [$({
                name = $sen:ident // super event name
                ref = $seref:literal // super event (uses) ref
                args = [
                    // super event arg name
                    // super event arg original type
                    // super event arg referenced type
                    $(($sean:ident: $seaot:ty => $seart:ty))*
                ]
            })*]
            rest = [ $($srest:tt)* ] // super rest
        }
        name = $en:ident // event name
        ref = $eref:literal // event (uses) ref
        args = [
            // event arg name
            // event arg original type
            // event arg referenced type
            $(($ean:ident: $eaot:ty => $eart:ty))*
        ]
        rest = [
            ($arg:ident: $type:ty)
            $($rest:tt)*
        ]
    }) => {
        define_event_handler_impl!(@collect_args = {
            super = {
                events = [$({
                    name = $sen
                    ref = $seref
                    args = [$(($sean: $seaot => $seart))*]
                })*]
                rest = [ $($srest)* ]
            }
            name = $en
            ref = $eref
            args = [
                $(($ean: $eaot => $eart))*
                ($arg: $type => &'__jumia__ &$type)
            ]
            rest = [ $($rest)* ]
        });
    };
    (@collect_args = {
        super = {
            events = [$({
                name = $sen:ident // super event name
                ref = $seref:literal // super event (uses) ref
                args = [
                    // super event arg name
                    // super event arg original type
                    // super event arg referenced type
                    $(($sean:ident: $seaot:ty => $seart:ty))*
                ]
            })*]
            rest = [ $($srest:tt)* ] // super rest
        }
        name = $en:ident // event name
        ref = $eref:literal // event (uses) ref
        args = [
            // event arg name
            // event arg original type
            // event arg referenced type
            $(($ean:ident: $eaot:ty => $eart:ty))*
        ]
        rest = [
            (@ref $arg:ident: $type:ty)
            $($rest:tt)*
        ]
    }) => {
        define_event_handler_impl!(@collect_args = {
            super = {
                events = [$({
                    name = $sen
                    ref = $seref
                    args = [$(($sean: $seaot => $seart))*]
                })*]
                rest = [ $($srest)* ]
            }
            name = $en
            ref = true
            args = [
                $(($ean: $eaot => $eart))*
                ($arg: &$type => &'__jumia__ &$type)
            ]
            rest = [ $($rest)* ]
        });
    };
    (@collect_args = {
        super = {
            events = [$({
                name = $sen:ident // super event name
                ref = $seref:literal // super event (uses) ref
                args = [
                    // super event arg name
                    // super event arg original type
                    // super event arg referenced type
                    $(($sean:ident: $seaot:ty => $seart:ty))*
                ]
            })*]
            rest = [ $($srest:tt)* ] // super rest
        }
        name = $en:ident // event name
        ref = $eref:literal // event (uses) ref
        args = [
            // event arg name
            // event arg original type
            // event arg referenced type
            $(($ean:ident: $eaot:ty => $eart:ty))*
        ]
        rest = [ ]
    }) => {
        define_event_handler_impl!(@collect_events = {
            events = [
                $({
                    name = $sen
                    ref = $seref
                    args = [$(($sean: $seaot => $seart))*]
                })*
                {
                    name = $en
                    ref = $eref
                    args = [$(($ean: $eaot => $eart))*]
                }
            ]
            rest = [ $($srest)* ]
        });
    };
    (@collect_events = {
        events = [$({
            name = $name:ident
            ref = $ref:literal
            args = [
                $(($arg_name:ident: $arg_orig_type:ty => $arg_ref_type:ty))*
            ]
        })*]
        rest = [ ]
    }) => {
        define_event_handler_impl!(@emit = [$({
            name = $name
            ref = $ref
            args = [
                $(($arg_name: $arg_orig_type => $arg_ref_type))*
            ]
        })*]);
    };
    // =========================================================================
    // Emit the result
    // =========================================================================
    (@emit = [$({
        name = $name:ident
        ref = $ref:literal
        args = [
            $(($arg_name:ident: $arg_orig_type:ty => $arg_ref_type:ty))*
        ]
    })*]) => {
        #[derive(Default)]
        pub struct EventHandler {
            $(
                $name: Vec<Box<
                    dyn for<'__jumia__> Fn(
                        &'__jumia__ Context,
                        $($arg_ref_type),*
                    ) -> Pin<Box<
                        dyn Future<Output = ()>
                        + Send
                        + '__jumia__
                    >>
                    + Send
                    + Sync
                >>,
            )*
            unknown: HashMap<String, Vec<Box<
                dyn for<'__jumia__> Fn(
                    &'__jumia__ Context,
                    &'__jumia__ Value,
                ) -> Pin<Box<
                    dyn Future<Output = ()>
                    + Send
                    + '__jumia__
                >>
                + Send
                + Sync
            >>>,
        }

        #[async_trait]
        impl SerenityEventHandler for EventHandler {
            $(
                async fn $name(&self, ctx: Context, $($arg_name: $arg_orig_type),*) {
                    for callback in self.$name.iter() {
                        callback(&ctx, $(&&$arg_name),*).await;
                    }
                }
            )*

            async fn unknown(&self, ctx: Context, name: String, raw: Value) {
                if let Some(callbacks) = self.unknown.get(&name) {
                    for callback in callbacks.iter() {
                        callback(&ctx, &raw).await;
                    }
                }
            }
        }

        impl EventHandler {
            $(
                #[allow(unused)]
                pub fn $name<F>(mut self, callback: F) -> Self
                where
                    F: for<'__jumia__> Fn(
                        &'__jumia__ Context, $($arg_ref_type),*)
                        -> Pin<Box<dyn Future<Output = ()> + Send + '__jumia__>>
                            + Send
                            + Sync
                            + 'static,
                {
                    self.$name.push(Box::new(callback));
                    self
                }
            )*
        }
    };
}

define_event_handler! {
    channel_create(@ref channel: GuildChannel)
    category_create(@ref category: ChannelCategory)
    category_delete(@ref category: ChannelCategory)
    channel_delete(@ref channel: GuildChannel)
    channel_pins_update(pins: ChannelPinsUpdateEvent)
    channel_update(old: Option<Channel>, new: Channel)
    guild_ban_addition(guild_id: GuildId, banned_user: User)
    guild_ban_removal(guild_id: GuildId, unbanned_user: User)
    guild_create(guild: Guild, is_new: bool)
    guild_delete(incomplete: GuildUnavailable, full: Option<Guild>)
    guild_emojis_update(guild_id: GuildId, current_state: HashMap<EmojiId, Emoji>)
    guild_integrations_update(guild_id: GuildId)
    guild_member_addition(guild_id: GuildId, new_member: Member)
    guild_member_removal(guild_id: GuildId, user: User, member: Option<Member>)
    guild_member_update(old: Option<Member>, new: Member)
    guild_members_chunk(chunk: GuildMembersChunkEvent)
    guild_role_create(guild_id: GuildId, new: Role)
    guild_role_delete(guild_id: GuildId, id: RoleId, role: Option<Role>)
    guild_role_update(guild_id: GuildId, old: Option<Role>, new: Role)
    guild_unavailable(guild_id: GuildId)
    guild_update(old: Option<Guild>, new: PartialGuild)
    invite_create(data: InviteCreateEvent)
    invite_delete(data: InviteDeleteEvent)
    message(message: Message)
    message_delete(channel_id: ChannelId, id: MessageId, guild_id: Option<GuildId>)
    message_delete_bulk(channel_id: ChannelId, ids: Vec<MessageId>, guild_id: Option<GuildId>)
    message_update(old: Option<Message>, new: Option<Message>, event: MessageUpdateEvent)
    reaction_add(reaction: Reaction)
    reaction_remove(reaction: Reaction)
    reaction_remove_all(channel_id: ChannelId, id: MessageId)
    presence_update(data: PresenceUpdateEvent)
    ready(data: Ready)
    resume(data: ResumedEvent)
    shard_stage_update(data: ShardStageUpdateEvent)
    typing_start(data: TypingStartEvent)
    user_update(old: CurrentUser, new: CurrentUser)
    voice_server_update(data: VoiceServerUpdateEvent)
    voice_state_update(guild_id: Option<GuildId>, old: Option<VoiceState>, new: VoiceState)
    webhook_update(guild_id: GuildId, channel_id: ChannelId)
    interaction_create(interaction: Interaction)
    integration_create(integration: Integration)
    integration_update(integration: Integration)
    integration_delete(integration_id: IntegrationId, guild_id: GuildId, application_id: Option<ApplicationId>)
    stage_instance_create(stage_instance: StageInstance)
    stage_instance_update(stage_instance: StageInstance)
    stage_instance_delete(stage_instance: StageInstance)
    thread_create(thread: GuildChannel)
    thread_update(thread: GuildChannel)
    thread_delete(thread: PartialGuildChannel)
    thread_list_sync(data: ThreadListSyncEvent)
    thread_member_update(thread_member: ThreadMember)
    thread_members_update(thread_members_update: ThreadMembersUpdateEvent)
}
