//! Event handlers for the async v0.2 world
//!
//! # How an event reaches your code
//!
//! The server fires an event and the host calls the `handle-event(id, server, event)` export,
//! passing a borrowed [`Event`] resource. The dispatcher looks `id` up in the table that
//! [`Context::register_event_handler`] filled, copies the typed record out of the resource, and
//! gives it to your [`EventHandler`] by value. Whatever the handler returns is written back into
//! the same resource.
//!
//! Nothing is borrowed from the host across that call, so a handler can `.await` as often as it
//! likes and the event data is still its own afterwards.
//!
//! # Where to look
//!
//! * [`EventHandler`] is the trait you implement
//! * [`Context::register_event_handler`] registers it and explains `blocking`
//! * [`define_event!`](crate::define_event) adds an event marker that this module does not ship

use std::{marker::PhantomData, sync::Arc};

pub use crate::host::event::{Event, EventPriority, EventType};
use crate::{
    Context, Server,
    registry::{LocalBoxFuture, Registry},
};

static EVENT_HANDLERS: Registry<dyn ErasedEventHandler> = Registry::new();

/// Ties an event marker type to its WIT record and [`EventType`] discriminant
///
/// Implement it with [`define_event!`](crate::define_event) instead of by hand; a marker is a unit
/// struct that exists so [`EventHandler`] and [`Context::register_event_handler`] can name an
/// event type
pub trait FromIntoEvent: Sized {
    /// The discriminant the host uses to identify this event when a handler is registered
    const EVENT_TYPE: EventType;

    /// The WIT record this event carries
    type Data;

    /// Copies the typed record out of the borrowed event resource
    ///
    /// Returns `None` when the resource carries a different record than this marker. The
    /// dispatcher then leaves the event untouched, so an id that points at the wrong handler is a
    /// host bug the plugin survives instead of a trap
    fn data_from_event(event: &Event) -> Option<Self::Data>;

    /// Writes the typed record back into the borrowed event resource
    ///
    /// Returns `false` when the resource carries a different record than this marker, so the
    /// dispatcher can report the handler error and keep the event the host dispatched
    fn data_into_event(event: &Event, data: Self::Data) -> bool;
}

/// The data type carried by an event
pub type EventData<E> = <E as FromIntoEvent>::Data;

/// A handler for one event type
///
/// Register an implementation with [`Context::register_event_handler`]
///
/// # Examples
///
/// A handler that awaits a permission check before it decides whether the chat goes through:
///
/// ```rust,ignore
/// struct ChatGuard;
///
/// impl EventHandler<PlayerChatEvent> for ChatGuard {
///     async fn handle(
///         &self,
///         _server: Server,
///         mut event: EventData<PlayerChatEvent>,
///     ) -> EventData<PlayerChatEvent> {
///         if !event.player.has_permission("chat.send".to_string()).await {
///             event.cancelled = true;
///         }
///         event
///     }
/// }
/// ```
#[allow(async_fn_in_trait, reason = "guest futures are intentionally not Send")]
pub trait EventHandler<E: FromIntoEvent>: Send + Sync + 'static {
    /// Processes an event and returns the data the host should use afterwards
    ///
    /// The server handle and the event data are owned, so nothing is borrowed from the host across
    /// an await and the handler can suspend as often as it needs to
    ///
    /// The server writes the returned data back into the event resource, and it ignores it for a
    /// handler registered as a notification
    ///
    /// # Reentry
    ///
    /// The host can start another dispatch while this one is suspended, which is why the receiver
    /// is `&`self`; shared mutable state needs interior mutability, and no lock guard may be held
    /// across an await
    async fn handle(&self, server: Server, event: E::Data) -> E::Data;
}

trait ErasedEventHandler: Send + Sync {
    fn handle_erased<'a>(&'a self, server: Server, event: &'a Event) -> LocalBoxFuture<'a, ()>;
}

struct HandlerWrapper<E, H> {
    handler: H,
    _event: PhantomData<fn() -> E>,
}

impl<E: FromIntoEvent, H: EventHandler<E>> ErasedEventHandler for HandlerWrapper<E, H> {
    fn handle_erased<'a>(&'a self, server: Server, event: &'a Event) -> LocalBoxFuture<'a, ()> {
        Box::pin(async move {
            let Some(data) = E::data_from_event(event) else {
                tracing::warn!(
                    event_type = ?E::EVENT_TYPE,
                    "Event handler id does not match the dispatched event"
                );
                return;
            };
            let data = self.handler.handle(server, data).await;
            if !E::data_into_event(event, data) {
                tracing::warn!(
                    event_type = ?E::EVENT_TYPE,
                    "Event handler returned data for a different event"
                );
            }
        })
    }
}

pub(crate) async fn dispatch(event_id: u32, server: Server, event: &Event) {
    // The registry lock is released before the handler runs
    let Some(handler) = EVENT_HANDLERS.get(event_id) else {
        return;
    };
    handler.handle_erased(server, event).await;
}

impl Context {
    /// Registers an event handler and returns its handler id
    ///
    /// The handler is stored in this crate's table under the id, and the host is told the id, the
    /// event type, the priority and `blocking`; registering is a synchronous host call, and only
    /// the handler is async
    ///
    /// `blocking` says whether the dispatcher uses the value the handler returns, and it does not
    /// say the callback blocks an operating system thread: the host keeps the event the handler
    /// wrote back for a blocking handler and ignores it for a notification handler, so a
    /// notification handler must not count on its edits being seen
    ///
    /// `event_priority` decides when this handler runs relative to the others for the same event
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// context.register_event_handler::<PlayerChatEvent, _>(
    ///     ChatGuard,
    ///     EventPriority::Normal,
    ///     true,
    /// );
    /// ```
    pub fn register_event_handler<E, H>(
        &self,
        handler: H,
        event_priority: EventPriority,
        blocking: bool,
    ) -> u32
    where
        E: FromIntoEvent + 'static,
        H: EventHandler<E>,
    {
        let id = EVENT_HANDLERS.register(Arc::new(HandlerWrapper {
            handler,
            _event: PhantomData::<fn() -> E>,
        }));
        self.register_event(id, E::EVENT_TYPE, event_priority, blocking);
        id
    }
}

/// Defines an event marker type that implements [`FromIntoEvent`]
///
/// The marker needs the [`EventType`] case it registers under and the resource accessors that
/// read and write its record: WIT names them `get-<case>` and `set-<case>`, which the binding
/// generator turns into the snake_case method names passed to this macro
///
/// # Examples
///
/// ```rust,ignore
/// pumpkin_plugin_api_v0_2::define_event! {
///     /// A player finished joining the server
///     PlayerJoinEvent => PlayerJoin, get_player_join, set_player_join,
///         pumpkin_plugin_api_v0_2::host::event::PlayerJoinEvent
/// }
/// ```
#[macro_export]
macro_rules! define_event {
    (
        $(#[$meta:meta])*
        $name:ident => $case:ident, $get:ident, $set:ident, $data:ty
    ) => {
        $(#[$meta])*
        pub struct $name;

        impl $crate::events::FromIntoEvent for $name {
            const EVENT_TYPE: $crate::events::EventType = $crate::events::EventType::$case;
            type Data = $data;

            fn data_from_event(event: &$crate::events::Event) -> ::core::option::Option<$data> {
                event.$get().ok()
            }

            fn data_into_event(event: &$crate::events::Event, data: $data) -> bool {
                event.$set(data).is_ok()
            }
        }
    };
}

define_event! {
    /// A player finished joining the server
    PlayerJoinEvent => PlayerJoin, get_player_join, set_player_join,
        crate::host::event::PlayerJoinEvent
}

define_event! {
    /// A player sent a chat message
    PlayerChatEvent => PlayerChat, get_player_chat, set_player_chat,
        crate::host::event::PlayerChatEvent
}

define_event! {
    /// A server tick is about to run
    ServerTickStartEvent => ServerTickStart, get_server_tick_start, set_server_tick_start,
        crate::host::event::ServerTickStartEvent
}
