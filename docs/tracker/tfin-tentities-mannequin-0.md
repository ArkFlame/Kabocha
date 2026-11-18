# Final rows from docs/tracker/tentities-mannequin.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- 1 equip via right-click (M)
- 2-4 interaction and push gaps (entity/mod.rs:614, 1064)
- 5 client-only pose and death

## Result
- 1 equip via right-click: OPEN. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/Mannequin.java` is present; the equip path (size M) was not ported in this pass. Not a data gap; needs its own port.
- 2-4 interaction and push gaps: OPEN. Not closed this pass; needs a Mannequin.java audit of `mobInteract` and push handling before editing entity/mod.rs:614 and :1064.
- 5 client-only pose and death: OPEN as a client-only effect; no server change required, not gated in code.

Status: fixed=0 open=3 (rows 1, 2-4). No compile run.
