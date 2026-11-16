---
name: kabocha-vanilla-port
description: Port or fix gameplay behaviour against decompiled vanilla source. Use for block/item/entity/mob parity work.
---
# Vanilla port
- Source: version from `CURRENT_MC_VERSION` (crates/pumpkin-data/src/generated/packet.rs). Download server jar via piston-meta manifest, unpack bundler, decompile with Vineflower `-only=<pkg>` into `~/.cache/kabocha/vanilla/<ver>/` (outside repo; reuse across sessions).
- Follow the whole call chain from the entry point incl. overrides and base-class side effects. Check early-return ordering, who else calls changed code, what the client predicts.
- Values: pumpkin-data constants, never pasted Java literals; Java `static final` stays a Rust const with the Java name.
- Mobs/AI: read CONTRIBUTING.md#mob-ai and AGENTS.md reach rules. Keep events firing (plugin/api/events/).
- Pumpkin quirk: player movement is client-authoritative.
