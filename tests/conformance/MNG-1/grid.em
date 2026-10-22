#$ test: parse-pass
#$ rules: MNG-1
# Module `grid` of ./reject_two_instances_of_one_symbol_are_both_named.em: a generic `at`.

pub fn at[T](x: T) -> T:
    return x
