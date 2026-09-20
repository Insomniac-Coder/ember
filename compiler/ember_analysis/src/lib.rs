//! MIR analyses (Part XVIII §4.6 onwards).
//!
//! Definite initialisation (§4.6) and drop elaboration (§4.9) are here. The
//! NLL borrow checker (§4.7), the exclusivity analysis (§4.8) and the effect
//! analysis (§4.10) join them; they all walk the same CFG and want the same
//! helpers.

pub mod borrows;
pub mod cycles;
pub mod access;
pub mod definite_init;
pub mod drops;
pub mod facts;
pub mod regions;
pub mod unused;

pub use borrows::{
    check_all as check_borrows_all, check_all_with_installed_callable_regions,
    insert_shared_accesses_all, install_callable_regions_all, verify_callable_regions_all,
};
pub use access::elide_static_accesses_all;
pub use cycles::lint_strong_cycles;
pub use definite_init::{
    analyze_all as analyze_definite_init_all, analyze_definite_init,
    check_all as check_definite_init_all,
    check_all_with_facts as check_definite_init_all_with_facts, check_definite_init,
    check_definite_init_with_facts, verify_initialization_facts, verify_initialization_facts_all,
};
pub use drops::{elaborate as elaborate_drops, elaborate_all as elaborate_drops_all};
pub use unused::check_all as check_unused_all;
