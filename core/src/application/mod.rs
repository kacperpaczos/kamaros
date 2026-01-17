//! # Application Layer
//!
//! Contains Use Cases (business logic) that orchestrate domain entities
//! and port operations.

pub mod save_checkpoint;
pub mod restore_version;

pub use save_checkpoint::{SaveCheckpointInput, SaveCheckpointOutput, SaveCheckpointUseCase};
pub use restore_version::{RestoreVersionInput, RestoreVersionOutput, RestoreVersionUseCase};

/// Common trait for all use cases
pub trait UseCase {
    type Input;
    type Output;
    type Error;
    
    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
