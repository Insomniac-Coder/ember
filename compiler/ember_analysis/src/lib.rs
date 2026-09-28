//! MIR analyses (Part XVIII §4.6 onwards).
//!
//! Definite initialisation (§4.6) and drop elaboration (§4.9) are here. The
//! NLL borrow checker (§4.7), the exclusivity analysis (§4.8) and the effect
//! analysis (§4.10) join them; they all walk the same CFG and want the same
//! helpers.

pub mod borrows;
pub mod callable_arguments;
pub mod check_hoisting;
pub mod cycles;
pub mod access;
pub mod definite_init;
pub mod drops;
pub mod facts;
pub mod inline;
pub mod loop_access;
pub mod loop_version;
pub mod long_access_lint;
pub mod range_facts;
pub mod regions;
pub mod stores;
pub mod uncounted_handles;
pub mod unused;

pub use borrows::{
    convert_quiet_calls_all,
    check_all as check_borrows_all, check_all_with_installed_callable_regions,
    insert_shared_accesses_all, install_callable_regions_all, verify_callable_regions_all,
};
pub use access::{elide_static_accesses_all, remove_never_firing_checks_all};
pub use loop_access::hoist_loop_accesses_all;
pub use check_hoisting::hoist_invariant_checks_all;
pub use loop_version::version_bounds_checked_loops_all;
pub use uncounted_handles::mark_uncounted_handles_all;
pub use range_facts::remove_proven_checks_all;
pub use callable_arguments::specialize_callable_arguments_all;
pub use inline::inline_single_calls_all;
pub use long_access_lint::lint_long_term_access_across_dynamic_calls_all;
pub use cycles::{
    inspect_ownership_graph, lint_strong_cycles, OwnershipCycle, OwnershipEdge, OwnershipEdgeKind,
    OwnershipInspection,
};
pub use definite_init::{
    analyze_all as analyze_definite_init_all, analyze_definite_init,
    check_all as check_definite_init_all,
    check_all_with_facts as check_definite_init_all_with_facts, check_definite_init,
    check_definite_init_with_facts, verify_initialization_facts, verify_initialization_facts_all,
};
pub use drops::{
    check_drop_self_escapes_all, elaborate as elaborate_drops, elaborate_all as elaborate_drops_all,
};
pub use unused::check_all as check_unused_all;
