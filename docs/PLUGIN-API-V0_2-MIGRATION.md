# Migrating a WASM plugin from v0.1 to v0.2

The v0.2 plugin ABI is the **async** ABI. The package name changes from
`pumpkin:plugin@0.1.0` to `pumpkin:plugin@0.2.0`, and the host picks the ABI
from that package version. v0.1 stays frozen: it is not removed and existing
plugins keep working unchanged until they opt in.

Source of truth for every signature quoted here:
`crates/pumpkin-plugin-wit/v0.1/` and `crates/pumpkin-plugin-wit/v0.2/`.
Diff them with:

```sh
diff -r crates/pumpkin-plugin-wit/v0.1 crates/pumpkin-plugin-wit/v0.2
```

## 1. Enable the v0.2 host

The host side is behind a feature flag:

```toml
# crates/pumpkin/Cargo.toml
plugin-v0_2 = ["pumpkin-wasm-host/v0_2"]
```

```sh
cargo build -p pumpkin --features plugin-v0_2
```

Native plugins are versioned separately by `PLUGIN_API_VERSION`
(`crates/pumpkin-core/src/plugin/mod.rs`); a native plugin built against a
different `PLUGIN_API_VERSION` is rejected at load. That is not a migration:
native plugins do not use WIT, but they are affected by the same-world field
changes listed under "New APIs".

## 2. What actually changed

The ABI is additive plus async. Only four functions were removed, and one
interface (`gametest`) was added.

* **v0.2 declares 240 `async func` entries; v0.1 declares none.** The host
  awaits handlers, so blocking work inside a handler no longer stalls the
  plugin thread — but every call site must now be awaited.
* **All plugin exports became `async func`** except `init-plugin` and
  `metadata`, which stay synchronous.
* **Plugin exports return the event they were given** (`handle-event`), with
  new rules for when host applies the changes.
* **`init-plugin` no longer sets anything up**: it only wires the plugin. Any
  setup that awaits host calls belongs in `on-load`.
* **`item-stack.constructor` and the other pure-data interfaces are no longer
  pre-imported by the plugin world.** A plugin that constructs item stacks now
  imports `item-stack` itself (the constructor still exists at
  `v0.2/item-stack.wit:36`). See §3.4.

### 2.1 async per interface

`async func` counts, v0.1 → v0.2:

| interface | v0.1 | v0.2 |
|---|---|---|
| `player` | 0 | 57 |
| `item-stack` | 0 | 48 |
| `world` | 0 | 37 |
| `inventory` | 0 | 30 |
| `boss-bar` | 0 | 14 |
| `server` | 0 | 14 |
| `plugin` (exports) | 0 | 13 |
| `gui` | 0 | 11 |
| `datapack` | 0 | 4 |
| `enchantments` | 0 | 4 |
| `recipe` | 0 | 3 |
| `block-entity` | 0 | 1 |
| `command` | 0 | 1 |
| `display` | 0 | 1 |
| `gametest` (new) | – | 1 |
| `ipc` | 0 | 1 |

Getters stay synchronous. Methods that touch the live world (teleport, vehicle
and passenger changes, swimming state, inventory mutation, entity spawn/remove,
scheduler, GUI, boss bars, server lookups) became async. Do not guess: if a
call site fails to compile after the version bump, the binding generator is
telling you that function is async.

### 2.2 Export signatures

From `crates/pumpkin-plugin-wit/v0.2/plugin.wit`:

```wit
export init-plugin: func();                                  // still sync
export on-load: async func(context: context) -> result<_, string>;
export on-unload: async func(context: context) -> result<_, string>;
export metadata;                                             // pure snapshot
export handle-event: async func(event-id: u32, server: server-instance, event: event) -> event;
export handle-command: async func(command-id: u32, sender: command-sender, server: server-instance, args: consumed-args) -> result<s32, command-error>;
export handle-command-suggestion: async func(handler-id: u32, sender: command-sender, server: server-instance, request: suggestion-request) -> command-suggestions;
export handle-task: async func(handler-id: u32, server: server-instance);
```

Rules that come with that shape:

* `metadata` must never call host imports; it is a snapshot, not a hook.
* `handle-event` must return a value of the **same variant case** it was
  dispatched with. A different case is treated as a handler error and the
  original event is kept.
* Modifications to the returned event are applied only for handlers registered
  with `blocking: true` (see `register-event` in `v0.2/context.wit`); for
  non-blocking handlers the returned event is ignored.

### 2.3 Removed functions

| interface | removed in v0.2 | replacement |
|---|---|---|
| `context` | `register-event-with-handle`, `unregister-event` | none; v0.2 `register-event` is keyed by `handler-id` only, and handlers are torn down with the plugin |
| `world` | `read-section`, `refresh-biomes` | none in v0.2; use `get-biome`/`set-biome` and the block APIs |

### 2.4 New APIs worth using

* `gametest.wit` – new interface, a one-to-one mapping of
  `@minecraft/server-gametest` (MIT, Microsoft), for automated in-world tests.
* Entity leash API: `is-leashed`, `get-leash-holder`, `get-leashed-entities`.
* Entity validity helpers: `is-valid`, `is-dead`, powder-snow freeze ticks,
  `is-in-water-or-rain`.
* `common.hand` is now used by `world`; `world` also exposes
  `status-effect.{status-effect-instance, status-effect-type}`.

The remaining ~1150 added lines are new members on existing resources; browse
`v0.2/*.wit` rather than porting v0.1 notes.

## 3. Migrating a Rust plugin

### 3.1 Bump the WIT target

Point your `wit-bindgen` invocation (or `wit-bindgen-cli`/`cargo-component`
manifest) at `pumpkin:plugin@0.2.0`. Regenerate bindings; do not hand-edit them.

### 3.2 Move awaited setup from `init-plugin` to `on-load`

`init-plugin` stays synchronous, so it cannot await host calls. If your plugin
did registration there because v0.1 allowed it, move that code into `on_load`.

### 3.3 Await the exports

Generated `Guest` methods are async in v0.2:

```rust
async fn on_load(context: Context) -> Result<(), String> {
    // Host calls that are async in the generated bindings are awaited here;
    // use the method names the generator emits for your target language.
    let world = context.world();      // sync getter
    world.set_time(1000).await;       // async mutation, name per bindings
    Ok(())
}
```

The exported `Plugin` trait surface in `crates/pumpkin-plugin-api` is unchanged
in shape (`on_load(&self, Context) -> Result<(), String>`); the runtime awaits
your handler and drives the WIT futures. Keep the body synchronous-looking and
await only host calls.

### 3.4 Import `item-stack` yourself

v0.2's plugin world no longer pre-imports `item-stack`, `uuid`, `event`,
`common`, `data-components`, `entity-types`, `bedrock-packets`, `biomes`,
`java-packets`, `particles`, `permission` or `sounds`. Add the ones you build
values from to your own WIT world:

```wit
world my-plugin {
    import pumpkin:plugin/item-stack@0.2.0;
    ...
}
```

Import only what you construct: values you receive from the host are typed
without any extra import.

### 3.5 Chainable builder methods

WIT resource methods take borrowed `self` and return nothing; v0.2 does not
change that. To keep fluent builder source syntax, enable `wit-bindgen`'s
`chainable_methods` support instead of returning `Self` from WIT. See
`crates/pumpkin-plugin-wit/AGENTS.md`.

### 3.6 Event handlers

Return the event you were handed, even when you change nothing:

```rust
// blocking handler: mutate and return the same case
Event::BlockBreak(mut e) => {
    e.cancelled = true;
    Event::BlockBreak(e)
}
```

Returning a different case, or returning early without the event, is a handler
error on the v0.2 host and the original event is kept.

## 4. Checklist

- [ ] WIT target is `pumpkin:plugin@0.2.0`; bindings regenerated.
- [ ] Host built with `--features plugin-v0_2`.
- [ ] Setup that needs `await` moved from `init-plugin` to `on-load`.
- [ ] World-mutating host calls are awaited; pure getters untouched.
- [ ] `handle-event` returns the same variant case as dispatched.
- [ ] Missing interfaces (`item-stack`, `uuid`, …) imported explicitly.
- [ ] Removed `context`/`world` functions replaced or dropped (§2.3).
- [ ] Plugin tested in-game; v0.1 behaviour is unchanged for plugins that do
      not migrate, so you can ship both versions side by side.
