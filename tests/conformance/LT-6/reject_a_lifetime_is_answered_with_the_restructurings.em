#$ test: parse-fail
#$ rules: LT-6, LEX-22
# `[LT-6]` — where a relationship between regions cannot be inferred, the diagnostic names the
# restructuring that expresses it: an owned result, an index instead of a view, a view struct, or
# `from`. A lifetime-looking `'a` is an unterminated character literal (`[LEX-22]`), whose note says so.

fn first(x: StrView) -> StrView:
    y = 'a    #$ error[E0008]: return an owned value, or use an index, a view struct or `from` [LT-6]
    return x
