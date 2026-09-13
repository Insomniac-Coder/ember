#$ test: compile-fail
#$ rules: SPN-3
#$ error[E2020]: `Span[i32]` has no method `reborrow` in this phase

# Shared Span is already Copy; SPN-3's explicit reborrow operation belongs to
# the move-only MutSpan surface.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    shared = values.as_span()
    _other = shared.reborrow()
