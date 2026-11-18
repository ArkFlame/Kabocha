# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN (row 30, S): needs an inventory `canHold` capacity helper, none exists in `pumpkin-inventory`; not added here.
STILL OPEN (rows 33-35, S): vanilla emits these particles only in the `isClientSide()` branch (`Arrow.java:88-95`, `AbstractArrow.java:250`); client-only, no server-side vanilla equivalent.
STILL OPEN (row 36, S): no Rust `onItemBreak` hook for arrows, and no arrow caller sets a non-default hit sound (`AbstractArrow.java:599`), so persisting it is a no-op today.
STILL OPEN (row 37, S): claim contradicted by vanilla. The deflect branch is an `else if` at `AbstractArrow.java:554`, not nested under the pierce check; the Rust code already matches.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
