# Tech-debt sweep leftovers (command, net, plugin, server, world, pumpkin-protocol)

Scope: non-test code in the listed dirs. Test modules (`#[cfg(test)]`) excluded.

## Fixed in this slice
- `command/argument_types/entity_selector/mod.rs` `find_single_player`: `pop().unwrap()` removed (match on `pop()` + `is_empty()`).
- `command/mod.rs` `CommandSender::send_message`: `#[allow(clippy::print_stdout)]` changed to `#[expect(.., reason)]`.

## Leftover TODO / FIXME (size: small each unless noted)
- command/mod.rs:378 command block custom-name check.
- net/bedrock/play/player_action.rs:278 bare TODO.
- net/chunk_sender.rs:575 chunk lifecycle built twice (Java `commit_batch` vs Bedrock). Medium-large.
- net/proxy/mod.rs:6 trait for proxies. Medium.
- net/java/play/cookie_response.rs:6 plugin access to cookies. Medium.
- net/java/play/teleport_to_entity.rs:21 vanilla cross-world entity search. Medium.
- plugin/mod.rs:481 remove a plugin's command nodes on reload. Medium.
- world/mod.rs:2443, 2444 scope block-break packets to loaded chunk viewers; light updates on the wire. Medium.
- world/mod.rs:3298 `canSpawnEntitiesInChunk` vanilla check. Small.

## Panics and expects left in non-test code (need design, not a one-line swap)
- command/context/command_source.rs:325, 334: `world()`/`server()` expect. Public API, many callers. Large.
- net/management/handlers.rs:45: expect on compile-time embedded JSON schema. Acceptable, low risk.
- net/bedrock/level_chunk.rs `read_var_uint`: index panic on short input and panic on overlong VarUInt. Input is network data. Needs Result plumbing. Medium.
- protocol codec `var_*.rs` `NonZero::new(5).expect` in a const. Acceptable.

## Commands (out of scope this pass, live workers)
Non-test unwrap/expect/panic in command/commands/*: bossbar 19, whitelist 8, data 3, op 2, deop 2, banlist 2, ban 1, banip 1, pardon 1, defaultgamemode 1, random 4, fetchprofile 5, compute 2. Entity selector option parser expects (`command/argument_types/entity_selector/option.rs`, 6 sites, `reader.expect`) are parser-level and need `?` on a fallible reader API. Size: medium.

## Lint allowances to convert to expect (size ~40)
- `#[allow(clippy::wildcard_imports)]` on line 1 of about 40 files in net/java/play/*.rs. Mechanical, but needs a compile to confirm each lint still fires. Blocked by live workers on some of those files.
- `#[allow(clippy::too_many_lines)]` in entity_selector/mod.rs, option.rs, place_recipe.rs and several protocol files. Small.

## Dead code and duplication
- Single-occurrence function scan not completed: textual grep counts test names too, so the result was mostly noise. Needs a call-graph tool (e.g. compiler `dead_code` warnings with `cargo check`) to find real dead helpers. Size unknown.
- Duplicated helpers: not identified in this pass.

## Printing
- plugin/mod.rs `ask_permission_confirmation` prints to stdout for an interactive prompt. Already `#[expect(clippy::print_stdout)]`. Intentional, no change.
- packet_decoder.rs:498 `println!` sits inside the test module. No change.
