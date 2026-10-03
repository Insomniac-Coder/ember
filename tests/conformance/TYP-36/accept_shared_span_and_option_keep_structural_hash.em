#$ test: run-pass
#$ rules: HASH-1, TYP-36, TYP-13, SPN-1, SPN-2
#$ profiles: debug, release, shipping
#$ stdout: true true
#$ stdout: true true
# Shared spans retain Hash when their element type does. The generic bound
# and actual hash method are both used, including through a niche Option.

from std.collections import Hash, Hasher, DefaultHasher

fn hash_value[T: Hash](value: T) -> u64:
    h = DefaultHasher.new()
    value.hash(h)
    return h.finish()

fn main():
    first = [1, 2, 3]
    second = [1, 2, 3]
    left = first.as_span()
    right = second.as_span()
    println(hash_value(left) == hash_value(right), hash_value(left[1..3]) == hash_value(right[1..3]))
    first_empty: Option[Span[int]] = Some(first[1..1])
    second_empty: Option[Span[int]] = Some(second[1..1])
    first_none: Option[Span[int]] = None
    second_none: Option[Span[int]] = None
    println(hash_value(first_empty) == hash_value(second_empty), hash_value(first_none) == hash_value(second_none))
