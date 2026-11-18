# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-c13d0e.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

| parse_result 99-103 | ItemStackTemplate count is validated 1..=99; out of range is a recipe error | Count is silently clamped with min(99) and 0 is accepted | Return None when count is 0 or > 99 | S | STILL OPEN (count range check; S, not attempted) |
| parse_cooking 255-262 | cookingtime/experience are parsed as int/float with codec range checks | t as i32 and e as f32 truncate or wrap a large JSON number | Use i32::try_from(t).ok() and reject non-finite experience | S | STILL OPEN (checked int/float conversion; S, not attempted) |
- STILL OPEN: parse_ingredient tag/item mixing and both-keys rule, parse_category strict enums, crafting_special_* suffix, brewing/transmute/dye/decorated_pot/imbue shapes (structural, not done in this pass).
- FIXED, parse_cooking checked int/float conversion (S, row was marked STILL OPEN): the prior

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
