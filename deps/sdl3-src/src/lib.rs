#![no_std]

#[cfg(not(windows))]
pub const SOURCE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../SDL");
#[cfg(windows)]
pub const SOURCE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\..\\SDL");

pub const REVISION: &str = "SDL-release-3.2.28-SISR";
pub const VERSION: &str = "3.2.28";
pub const REVISION_TAG: &str = "release-3.2.28";
pub const REVISION_TAG_BASE: &str = "release";
pub const REVISION_OFFSET: &str = "0"; // TODO:
pub const REVISION_HASH: &str = "SISR"; // TODO:
