mod config;
mod error;
mod pb;
mod types;
mod utils;

pub use config::*;
pub use error::*;
pub use pb::*;
pub use types::*;
pub use utils::*;

pub trait Validator {
  fn validate(&self) -> Result<(), Error>;
}
