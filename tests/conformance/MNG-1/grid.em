#$ test: parse-pass
#$ rules: MNG-1
# Module `grid` of ./reject_two_items_of_one_path_are_refused_before_their_instances.em: a generic `at`.

pub fn at[T](x: T) -> T:
    return x
