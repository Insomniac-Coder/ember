#$ test: compile-fail
#$ rules: TXT-2, FFI-31b
#$ profiles: debug
## `[TXT-2]`, `[FFI-31b]` - an export that takes a `StrView` would be handed bytes C never validated:
## `E5063`, with the `Span[u8]` and `to_str()?` to write instead.

pub extern "C" fn take(s: StrView) -> i32:    #$ error[E5063]: `s` would be text C never validated; take `s: Span[u8]` and write `s.to_str()?` for the `StrView`
    return 0

fn main():
    pass
