#! language "0.9.7"
## `std.borrow` — explicit, callback-scoped composition of independently
## borrowed views.  These are ordinary generic library functions; they do not
## own storage or introduce a second view/ownership representation.

## `[LT-8]` / `[LT-8a]` — shared view forwarding.  An omitted parameter mode
## remains the ordinary shared-borrow mode, including for the callback's
## `Span` arguments.
pub fn with_views2[A, B, R](a: Span[A], b: Span[B],
                            f: @latebound fn(Span[A], Span[B]) -> R) -> R:
    return f(a, b)

pub fn with_views3[A, B, C, R](a: Span[A], b: Span[B], c: Span[C],
                               f: @latebound fn(Span[A], Span[B], Span[C]) -> R) -> R:
    return f(a, b, c)

pub fn with_views4[A, B, C, D, R](a: Span[A], b: Span[B], c: Span[C], d: Span[D],
                                  f: @latebound fn(Span[A], Span[B], Span[C], Span[D]) -> R) -> R:
    return f(a, b, c, d)

## `[LT-8]` / `[LT-11]` — every mutable input and callback parameter is an
## ordinary `mut` reborrow.  `_mut` selects this all-mutable API family; it
## neither consumes a view nor grants an aliasing exception.
pub fn with_views2_mut[A, B, R](mut a: MutSpan[A], mut b: MutSpan[B],
                                f: @latebound fn(mut MutSpan[A], mut MutSpan[B]) -> R) -> R:
    return f(a, b)

pub fn with_views3_mut[A, B, C, R](mut a: MutSpan[A], mut b: MutSpan[B],
                                   mut c: MutSpan[C],
                                   f: @latebound fn(mut MutSpan[A], mut MutSpan[B],
                                         mut MutSpan[C]) -> R) -> R:
    return f(a, b, c)

pub fn with_views4_mut[A, B, C, D, R](mut a: MutSpan[A], mut b: MutSpan[B],
                                      mut c: MutSpan[C], mut d: MutSpan[D],
                                      f: @latebound fn(mut MutSpan[A], mut MutSpan[B],
                                            mut MutSpan[C], mut MutSpan[D]) -> R) -> R:
    return f(a, b, c, d)
