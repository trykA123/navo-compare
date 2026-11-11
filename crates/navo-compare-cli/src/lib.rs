//! Command line of Sidewise: turns arguments into an `Invocation` and names the exit codes.
//! Pure logic with no GUI, so the app and the console launcher share it.

pub mod args;

#[cfg(test)]
mod args_pick_tests;

#[cfg(test)]
mod args_tests;
