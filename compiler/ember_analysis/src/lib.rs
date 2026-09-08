//! MIR analyses (Part XVIII §4.6 onwards).
//!
//! Definite initialisation (§4.6) and drop elaboration (§4.9) are here. The
//! NLL borrow checker (§4.7), the exclusivity analysis (§4.8) and the effect
//! analysis (§4.10) join them; they all walk the same CFG and want the same
//! helpers.

pub mod borrows;
pub mod definite_init;
pub mod drops;
pub mod unused;

pub use definite_init::{check_all as check_definite_init_all, check_definite_init};
pub use drops::{elaborate as elaborate_drops, elaborate_all as elaborate_drops_all};
pub use borrows::check_all as check_borrows_all;
pub use unused::check_all as check_unused_all;
