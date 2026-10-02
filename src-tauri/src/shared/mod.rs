pub mod error;
pub mod ids;

pub use error::AppError;
pub use ids::generate_id;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
