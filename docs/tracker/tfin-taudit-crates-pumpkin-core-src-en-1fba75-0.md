# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-1fba75.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: owner.rs

- STILL OPEN: projectile NBT persistence (owner/LeftOwner/HasBeenShot have no caller, owner.rs:218-249, verified by grep: no call sites for ProjectileOwner::write_nbt outside owner.rs in pumpkin-core entity).
- STILL OPEN: EntityReference weak/UUID pre-registration owner (owner.rs:33-38), is_pickable predicate (owner.rs:189), vanish check (owner.rs:210-216), restore_from hook (owner.rs:112-118).
- STILL OPEN: lastDeflectedBy entity_id, marked None in note (no action).
Supersedes the earlier partial Result above: every STILL OPEN gap is already fixed in the

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
