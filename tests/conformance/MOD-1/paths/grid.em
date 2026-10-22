#$ test: parse-pass
#$ rules: MOD-1
# Module `paths.grid` of ../accept_a_module_and_an_item_of_one_name_in_two_scopes.em and
# ../reject_an_item_named_like_an_imported_module.em.

pub fn at(i: int) -> int:
    return i * 10
