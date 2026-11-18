# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-6aac9c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN: gap 1 (M) needs the vanilla NBT key (`DisplayTile` vs newer `DisplayState`) and block-state tracked-data serializer confirmed against decompiled `AbstractMinecart`; `.vendor/` is empty and no cargo run was allowed.
STILL OPEN: gap 2 (M) needs non-player dimension change support, which does not exist yet; L-scale infrastructure risk.
STILL OPEN: gap 3 (S) vanilla `MinecartFurnace`/`MinecartCommandBlock` tick ordering is marked verify and not confirmed against decompiled source.
STILL OPEN: gap 4 (S) vanilla entry point for flint-and-steel on `MinecartTNT` is marked verify.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
