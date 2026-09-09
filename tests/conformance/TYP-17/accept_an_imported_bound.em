#$ test: run-pass
#$ rules: TYP-17, IFC-3, MOD-3
## "Inside a generic body, only operations provided by the bounds are
## permitted." The bound, the interface it names, its supertrait and the
## implementation may each live in a different module: `[MOD-4]` allows import
## cycles inside a package, so no load order makes a per-module pass correct.

from std.core import Ordering, Ord, Eq

struct Version:
    major: i32

extend Version implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.major == other.major

extend Version implements Ord:
    fn cmp(self, other: Self) -> Ordering:
        if self.major < other.major: return Ordering.Less
        if self.major > other.major: return Ordering.Greater
        return Ordering.Equal

fn newer[T: Ord](a: T, b: T) -> T:
    match a.cmp(b):
        Greater => return a
        _ => return b

fn main():
    println(newer(Version(3), Version(7)).major)
#$ stdout: 7
