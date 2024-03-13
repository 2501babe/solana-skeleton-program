pub mod instruction;
pub mod processor;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub use solana_program;

solana_program::declare_id!("4fed255TTjQF4UhhrHTV3y3uk3NaHz8vUaQzgG1hyRhZ");
