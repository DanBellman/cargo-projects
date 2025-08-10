pub mod filesystem;

pub use filesystem::{FileSystem, RealFileSystem};
#[cfg(test)]
pub use filesystem::{MockFileSystem, MockMetadata};