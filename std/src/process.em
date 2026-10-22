## `std.process` — the process: its command line and how it ends (`[STD-24]`).

## `[STD-24]` — ends the process with status `code` at once: the standard streams are flushed, and
## no destructor or `defer` block runs. Returning from `main` is the clean exit.
pub fn exit(code: int) -> !:
    exit_now(code)

## Ends the process abnormally at once, with the platform's abnormal status, after flushing the
## standard streams.
pub fn abort() -> !:
    abort_now()

## `[FN-8]` — the command line, the program's name first, decoded as UTF-8 with each invalid
## sequence replaced by U+FFFD: what `main`'s `args` holds.
## It allocates the arguments' text.
pub fn args() -> Array[String]:
    return args_utf8()

## `[FN-8]` — the command line's own bytes, nothing replaced. On Windows the command line is
## UTF-16, and its bytes are its WTF-8: UTF-8, with a lone surrogate kept as its three bytes.
## It allocates the arguments' text.
pub fn args_os() -> Array[Array[u8]]:
    return args_bytes()

## The runtime's own (`ember_hir::RuntimeFn`): a call is the runtime's, and these staging bodies
## never run.
fn exit_now(code: int) -> !:
    panic("the runtime's")

fn abort_now() -> !:
    panic("the runtime's")

## It allocates a new `Array`.
fn args_utf8() -> Array[String]:
    return []

## It allocates a new `Array`.
fn args_bytes() -> Array[Array[u8]]:
    return []
