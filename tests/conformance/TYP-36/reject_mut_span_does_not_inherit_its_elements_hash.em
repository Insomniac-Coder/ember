#$ test: compile-fail
#$ rules: HASH-1, TYP-17, TYP-36, SPN-3
# A pass-only generic body isolates the capability predicate: no unavailable
# hash method is called. The old predicate incorrectly admitted this call.

from std.collections import Hash

fn requires_hash[T: Hash](value: T):
    pass

fn main():
    values = [1, 2]
    view = values.as_mut_span()
    requires_hash(view) #$ error[E2040]
