#$ test: parse-fail
#$ rules: LT-6
# "Explicit named lifetimes (`fn f['a, 'b](…)`) are **reserved for v2**; the
# grammar reserves the `'ident` token form." `[LEX-22]` is what makes this a
# distinct token rather than a malformed char literal, which is why the
# diagnostic can name the feature instead of complaining about a quote.

fn longest['a](x: str) -> str:     #$ error[E0007]: named lifetimes are not supported in this version
    return x

fn main():
    println(1)
