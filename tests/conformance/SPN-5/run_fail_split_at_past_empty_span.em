#$ test: run-fail
#$ rules: SPN-5, SPN-2
#$ profiles: debug, release, shipping
#$ panics: index 1 is out of bounds for a length of 0

# `split_at(len)` is valid even for an empty view; only an index greater than
# the length panics.
fn main():
    values: Array[i32] = Array[i32]()
    span = values.as_span()
    _parts = span.split_at(1)
