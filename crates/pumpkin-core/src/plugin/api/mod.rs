pub mod context;
pub mod events;
pub mod gui;
pub mod tab_list;
pub mod title;

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context as TaskContext, Poll},
};

pub use context::*;
pub use events::*;
pub use tab_list::*;
pub use title::*;

/// Struct representing metadata for a plugin.
///
/// This struct contains essential information about a plugin, including its name,
/// version, authors, and a description. It is generic over a lifetime `'s` to allow
/// for string slices that are valid for the lifetime of the plugin metadata.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// The name of the plugin.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
    /// The authors of the plugin.
    pub authors: Vec<String>,
    /// A description of the plugin.
    pub description: String,
    /// The dependencies of the plugin.
    pub dependencies: Vec<String>,
    /// The permissions requested by the plugin.
    pub permissions: Vec<String>,
}

/// This type represents a future for the plugin.
pub type PluginFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Wraps a [`PluginFuture`] so each poll runs inside the given tokio runtime.
///
/// Native plugins own a global runtime (`GLOBAL_RUNTIME` in `#[plugin_impl]`), but
/// their lifecycle futures are polled by the server runtime. Without entering the
/// plugin runtime, tokio I/O created inside `#[plugin_method]` aborts with
/// "there is no reactor running".
pub struct RuntimeScopedFuture<'a, T> {
    handle: tokio::runtime::Handle,
    future: PluginFuture<'a, T>,
}

impl<'a, T> RuntimeScopedFuture<'a, T> {
    pub fn new(handle: tokio::runtime::Handle, future: PluginFuture<'a, T>) -> Self {
        Self { handle, future }
    }
}

impl<T> Future for RuntimeScopedFuture<'_, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<T> {
        let this = self.get_mut();
        let _guard = this.handle.enter();
        this.future.as_mut().poll(cx)
    }
}

/// Trait representing a plugin with asynchronous lifecycle methods.
///
/// This trait defines the required methods for a plugin, including hooks for when
/// the plugin is loaded and unloaded.
pub trait Plugin: Send + Sync + 'static {
    /// Asynchronous method called when the plugin is loaded.
    ///
    /// This method initializes the plugin within the server context.
    #[expect(unused)]
    fn on_load(&self, server: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move { Ok(()) })
    }

    /// Asynchronous method called when the plugin is unloaded.
    ///
    /// This method cleans up resources when the plugin is removed from the server context.
    #[expect(unused)]
    fn on_unload(&self, server: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move { Ok(()) })
    }

    /// Asynchronous method called when the plugin receives an IPC message.
    ///
    /// This processes the message, and optionally returns a response
    #[expect(unused)]
    fn on_ipc_message(
        &self,
        sender: &str,
        message: &[u8],
    ) -> PluginFuture<'_, Result<Vec<u8>, String>> {
        Box::pin(async move { Err("This plugin cannot receive messages.".to_string()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_scoped_future_polls_inside_plugin_runtime() {
        let runtime = tokio::runtime::Runtime::new().expect("plugin runtime");
        let expected = runtime.handle().id();
        let future = RuntimeScopedFuture::new(
            runtime.handle().clone(),
            Box::pin(async move { tokio::runtime::Handle::current().id() }),
        );
        let mut future = Box::pin(future);
        let waker = std::task::Waker::noop();
        let mut context = std::task::Context::from_waker(waker);
        assert_eq!(
            future.as_mut().poll(&mut context),
            std::task::Poll::Ready(expected)
        );
    }
}
