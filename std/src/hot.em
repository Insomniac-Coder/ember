## `std.hot` — what a program sees of hot reload (Annex B).
##
## A package built with `[build] reload` is a shared library whose functions are reached through
## permanent thunks. A reload runs in four phases: plan (compare the old and new schemas; refuse
## here, with a reason), prepare (build every migrated instance; may fail and change nothing),
## commit (publish; cannot fail) and reclaim (drop what the old image owned). The runtime does all of
## it; a program meets it in two places:
##
## `[HR-38]` — `ReloadError`, what a type's `migrate_from` returns when it cannot build the new value
## from the old (`[HR-16]`), and what a refused or failed reload reports: `SchemaRefused` names the
## type and why (`[HR-14]`), `Migration` the type, which instance and the message of the check that
## failed (`[HR-35]`), `Allocation` that the transaction arena ran out (`[HR-36]`), `Foreign` the
## failure of a foreign call (`[HR-39]`), `Timeout` a resource still in use past its bound
## (`[HR-24]`). `[HR-39]` — a foreign call's failure becomes `Foreign` through `?`: every
## `std.ffi.ForeignFailure` (a C++ exception's `CppError`, a C status's error) converts into one,
## its text the failure's.
##
## `[HR-33]` — `checkpoint()`, the point where `ember run --hot` may apply a reload: between frames
## of a game, say. It returns at once when no reload is waiting, and in a build without reload it
## does nothing. The compiler knows it by its name here and calls the runtime.

from std.ffi import ForeignFailure

pub enum ReloadError:
    SchemaRefused(type_name: String, reason: String)
    Migration(type_name: String, instance: u64, message: String)
    Allocation
    Foreign(error: String)
    Timeout

extend[E: ForeignFailure] ReloadError implements From[E]:
    fn from(owned value: E) -> ReloadError:
        return ReloadError.Foreign(f"{value}")

pub fn checkpoint():
    pass
