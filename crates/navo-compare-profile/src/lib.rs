//! Profile of Sidewise: where its files live, which format version they have,
//! and the store of saved sessions. Pure logic with no GUI.

pub mod brand;
pub mod paths;
pub mod recent;
pub mod session;
pub mod session_store;

#[cfg(test)]
mod paths_tests;

#[cfg(test)]
mod recent_tests;

#[cfg(test)]
mod session_tests;

#[cfg(test)]
mod session_store_tests;
