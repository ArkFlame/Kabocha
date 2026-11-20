# Multi-version support (operator guide)

Kabocha can accept Minecraft Java Edition clients older than the version it natively targets. `pumpkin-multiversion` translates their packets on the fly; newer clients connect normally.

## Enabling and disabling

Multi-version support is planned to be controlled by the `multiversion.enabled` key in `pumpkin.toml` (defined in `crates/pumpkin-config`). It defaults to `true`, so older clients are accepted unless you opt out:

```toml
[multiversion]
enabled = false
```

Set `enabled = false` (or omit the section to keep the default `true`). Disabling translation means only clients at the server's native version can connect; older clients will be rejected.

## Supported versions

The translated range is **1.7.2 up to the version before the current one**. The current version is served natively. The precise list of versions, protocols and id tables is tracked in [`VERSION_SUPPORT.tsv`](VERSION_SUPPORT.tsv); consult that file for the authoritative table rather than this summary.

## Known limitations

- **No live testing yet.** The translation layer has not been exercised against a running server with real older clients; treat it as best-effort and report discrepancies.
- **1.7.x chunk and light caveats.** Chunk and lighting formats changed substantially in these versions. Some chunk/light data may be incomplete or differ from vanilla for 1.7.x clients.

## Credits

Ported from [pumpkin-java-multiversion](https://github.com/Pumpkin-MC/pumpkin-java-multiversion) by Pumpkin-MC (MIT OR Apache-2.0). Protocol mapping data comes from the ViaVersion projects (ViaVersion, ViaBackwards, ViaRewind, ViaVersion/Mappings, GPL-3.0). See `crates/pumpkin-multiversion/assets/NOTICE.md` for full licensing and attribution.
