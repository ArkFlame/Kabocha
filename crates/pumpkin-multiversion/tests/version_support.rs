//! Automated guards for `docs/VERSION_SUPPORT.tsv`.

use std::{
    collections::HashSet,
    fs,
    io::{Error, ErrorKind, Result},
    path::PathBuf,
};

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_multiversion::legacy::KNOWN_VERSIONS;

/// One data row of the tracker; the audit columns are ignored by these guards.
struct Row {
    version: String,
    protocol: String,
    status: String,
}

/// Builds an `InvalidData` error with a fixed message.
fn invalid_data(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

/// Absolute path to the tracker, resolved from this crate's manifest.
fn tracker_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/VERSION_SUPPORT.tsv")
}

/// Reads and splits the tracker, skipping its header row.
fn read_rows() -> Result<Vec<Row>> {
    let text = fs::read_to_string(tracker_path())?;
    let mut rows = Vec::new();
    for line in text.lines().skip(1).filter(|line| !line.is_empty()) {
        let mut fields = line.split('\t');
        let version = fields
            .next()
            .ok_or_else(|| invalid_data("missing `version`"))?;
        let protocol = fields
            .next()
            .ok_or_else(|| invalid_data("missing `protocol`"))?;
        let _era = fields.next().ok_or_else(|| invalid_data("missing `era`"))?;
        let status = fields
            .next()
            .ok_or_else(|| invalid_data("missing `status`"))?;
        rows.push(Row {
            version: version.to_owned(),
            protocol: protocol.to_owned(),
            status: status.to_owned(),
        });
    }
    Ok(rows)
}

/// Every translated client version is tracked with `supported` status.
#[test]
fn supported_versions_are_tracked() -> Result<()> {
    let rows = read_rows()?;
    for version in pumpkin_multiversion::supported_versions() {
        let name = version.to_string();
        assert!(
            rows.iter()
                .any(|row| row.version == name && row.status == "supported"),
            "missing `supported` row for {name}"
        );
    }
    Ok(())
}

/// Bumping `CURRENT_MC_VERSION` requires carrying its predecessor forward.
#[test]
fn superseded_version_is_tracked() -> Result<()> {
    let rows = read_rows()?;
    let previous = KNOWN_VERSIONS
        .iter()
        .copied()
        .filter(|&version| version < CURRENT_MC_VERSION)
        .max()
        .ok_or_else(|| invalid_data("`CURRENT_MC_VERSION` is the oldest known version"))?;
    let name = previous.to_string();
    assert!(
        rows.iter()
            .any(|row| row.version == name && row.status == "supported"),
        "CURRENT_MC_VERSION moved: add the superseded version to legacy::KNOWN_VERSIONS and docs/VERSION_SUPPORT.tsv"
    );
    Ok(())
}

/// Every row parses its protocol and no version appears twice.
#[test]
fn rows_are_integer_protocols_and_unique() -> Result<()> {
    let rows = read_rows()?;
    let mut seen = HashSet::new();
    for row in &rows {
        let version = &row.version;
        let protocol = &row.protocol;
        assert!(seen.insert(version), "duplicate row for {version}");
        assert!(
            protocol.parse::<i32>().is_ok(),
            "invalid protocol `{protocol}` for {version}"
        );
    }
    Ok(())
}
