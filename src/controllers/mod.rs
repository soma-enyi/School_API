pub mod auth;
pub mod admin;
pub mod student;
pub mod mentor;
pub mod school;

#[cfg(test)]
mod tests;

// Re-export only auth functions for convenience in route files
pub use auth::*;
