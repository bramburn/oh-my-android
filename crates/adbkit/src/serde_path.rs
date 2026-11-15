//! Serialize a `PathBuf` as a display string.
//!
//! `PathBuf`'s own `Serialize` is lossy/erroring for non-UTF-8 paths, and these values cross IPC to be
//! *shown*, never to be re-parsed — so a display string is the honest representation.

use std::path::Path;

use serde::Serializer;

pub fn serialize<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&path.display().to_string())
}
