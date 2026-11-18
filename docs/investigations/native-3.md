# Native agent 3 summary

Status: UNVERIFIED (no code changes, no compile).

Work list gap: `/tmp/kabocha-runs/native-3.md` contains no row list; it is a pointer stub. The 28 rows were reconstructed from `LEFT OPEN` / `STILL OPEN` markers in `docs/tracker/tblocks-*.md` latest Result sections.

Reconstructed rows and decisions:
- tblocks-chests row 3 (lid event count u8): NOT A GAP. Wire is `writeByte` in vanilla `ClientboundBlockEventPacket.java:37-38`. Recorded in `docs/tracker/tblocks-chests.md`.
- tblocks-creaking-heart rows 4-6 (creaking goals, hurt particles, piston push reaction): BLOCKED. Push reaction needs generated `pumpkin-data` Block field; creaking goals need a Brain/Behaviour system (`CreakingAi.java:30-85`) Pumpkin lacks; particles are client-only.
- tblocks-beds rows 39-42 and bed.rs rule rows: BLOCKED-DATA. `BedRule` fields live in `crates/pumpkin-data/src/generated/dimension.rs`, which must not be hand-edited.
- tblocks-dispenser gap 3 (feature flags): BLOCKED-DATA. Needs item feature-set data from Extractor.
- tblocks-jukebox #1 (dynamic jukebox_song registry): BLOCKED. Needs cross-crate registry access at four call sites.
- tblocks-structure-block rows (entity capture, collision palette split, strict placement, empty palette with entities): LEFT OPEN. Multi-file structure-template port, too large for an uncompiled pass.
- tblocks-lodestone rows 1, 3, 6 (compass.rs): out of unit, left open.
- tblocks-vault gaps 3, 6 (display cycle, ominous activation): client visual or unported event; left open.
- tblocks-beacon rows 36, 38, 39: client beam rendering and world handle for sound; left open.
- tblocks-campfire row 3 (cooked result split): latent, left open.
- tblocks-fire gap 3 (TNT before priming, deliberate ordering): left open.
- tblocks-chiseled-bookshelf rows 4-6: ordering, block-entity stack read, gametest harness; left open.
- tblocks-daylight-detector-b4adbb rows 1 and 3 and tblocks-daylight-detector 6px occlusion (L): left open.
- tblocks-sculk-sensor row 1 (occlusion test): reported as STILL OPEN in its note; not re-verified this pass.
- tblocks-pistons row 4 (players excluded from push): intentional divergence, left open.
- tblocks-note-block row 1: premise false per its own note, NOT A GAP.

Not done: the rest of the 28 cross-file fixes. A single compile is needed to land any of them safely, and the list could not be recovered from disk.
