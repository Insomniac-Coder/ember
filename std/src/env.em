## `std.env` — the process's environment (`[STD-24]`).

## `[STD-24]` — the value of the environment variable `name`, or `None` when it is not set. A value
## that is not UTF-8 has each invalid sequence replaced by U+FFFD.
## It allocates a new `String`.
pub fn var(name: str) -> Option[String]:
    value = String()
    if lookup(name, value):
        return Some(value)
    return None

## The runtime's own (`ember_hir::RuntimeFn`): whether `name` is set, its value appended to `out`.
## This staging body never runs.
fn lookup(name: str, mut out: String) -> bool:
    return false
