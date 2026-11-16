---
name: kabocha-protocol-codegen
description: Packet changes, generated pumpkin-data, WIT/plugin API changes, Minecraft version bumps.
---
# Protocol / codegen
- Read crates/pumpkin-protocol/AGENTS.md before packets, tools/pumpkin-codegen/AGENTS.md before generators, crates/pumpkin-plugin-wit/AGENTS.md before WIT.
- Never hand-edit crates/pumpkin-data/src/generated/. Regenerate: `cargo run --locked -p pumpkin-codegen` (WIT: `-- wit`). Commit only generated diffs your change caused.
- Missing data not in assets/ -> needs Extractor change; note in QUEUE.md as blocked, do not hardcode.
- WIT changes are public: additive only; update host v0_1/v0_2, SDK, runtime together; bump PLUGIN_API_VERSION on breaking native API.
