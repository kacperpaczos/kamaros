//! # Application Layer
//!
//! Contains Use Cases (business logic) that orchestrate domain entities
//! and port operations.

pub mod save_checkpoint;
pub mod restore_version;
pub mod garbage_collect;

pub use save_checkpoint::{SaveCheckpointInput, SaveCheckpointOutput, SaveCheckpointUseCase};
pub use restore_version::{RestoreVersionInput, RestoreVersionOutput, RestoreVersionUseCase};
pub use garbage_collect::{GcUseCase, GcResult};
pub mod export_archive;
pub mod import_archive;

pub use export_archive::{ExportArchiveUseCase, ExportArchiveInput};
pub use import_archive::{ImportArchiveUseCase, ImportArchiveInput, ImportArchiveOutput};

/// Common trait for all use cases
pub trait UseCase {
    type Input;
    type Output;
    type Error;
    
    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
