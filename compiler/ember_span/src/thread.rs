//! The compiler's threads (D-924): every thread the compiler starts has the compile thread's stack.
//!
//! Checking and lowering recurse once for each level of an expression, and a debug build spends about
//! 65 KB of stack on a level of a binary operator (D-330). A worker started with the 2 MB a host gives
//! a new thread overflowed on a sum of 500 terms that the compile thread checks (`E0112`'s page under
//! `-j 2` and more). The size is reserved, not committed: a thread spends only what a program's
//! nesting uses.

/// The stack of the compile thread and of every thread it starts (D-330, D-924).
pub const COMPILER_STACK: usize = 256 << 20;

/// `scope.spawn(f)`, with the compiler's stack.
pub fn spawn_scoped<'scope, 'env, F, T>(scope: &'scope std::thread::Scope<'scope, 'env>, f: F) -> std::thread::ScopedJoinHandle<'scope, T>
where
    F: FnOnce() -> T + Send + 'scope,
    T: Send + 'scope,
{
    std::thread::Builder::new().stack_size(COMPILER_STACK).spawn_scoped(scope, f).expect("a compiler thread starts")
}
