# pumpkin-multiversion

Built-in Java Edition protocol translation for Kabocha, ported from Pumpkin-MC's `pumpkin-java-multiversion` plugin.
It lets clients older than the server's native version connect by translating packet ids and payloads in the tick.

Supported clients: 1.7.2 up to the version before the current one (the current version is native and needs no translation).

The server core calls `outgoing_play`, `incoming_play`, `outgoing_connection` and `incoming_connection` in `src/lib.rs` when it queues or decodes packets, and `supported_versions` to report the supported client range.

The exact supported version list lives in `docs/VERSION_SUPPORT.tsv`; keep it in sync with the translated range.

When Kabocha moves to a new Minecraft release, the superseded version must be carried forward as a translated older client in the same change (protocol number, id tables and packet translations).

A guard test in `tests/version_support.rs` fails the build if the tracker is missing the version just before `CURRENT_MC_VERSION`.
See `docs/MULTIVERSION.md` for operator configuration and known limitations.
