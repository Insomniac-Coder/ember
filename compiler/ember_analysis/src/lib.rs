//! MIR analyses (Part XVIII §4.6 onwards).
//!
//! Definite initialisation is here now. The NLL borrow checker (§4.7), the
//! exclusivity analysis (§4.8), drop elaboration (§4.9) and the effect
//! analysis (§4.10) join it in Phases 2 to 4; they all walk the same CFG and
//! want the same helpers.

pub mod definite_init;

pub use definite_init::{check_all as check_definite_init_all, check_definite_init};
