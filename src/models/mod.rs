pub mod user;
pub mod course;
pub mod application;
pub mod enrollment;
pub mod assignment;
pub mod submission;
pub mod material;
pub mod attendance;

#[cfg(test)]
mod tests;

pub use user::*;
pub use course::*;
pub use application::*;
pub use enrollment::*;
pub use assignment::*;
pub use submission::*;
pub use material::*;
pub use attendance::*;
