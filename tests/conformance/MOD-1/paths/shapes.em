#$ test: parse-pass
#$ rules: MOD-1
# Module `paths.shapes` of ../accept_a_module_and_an_item_of_one_name_in_two_scopes.em: its `grid`
# is `paths.shapes.grid`, and the module `paths.grid` is in the scope of `paths`, not of this module.

pub struct grid:
    pub n: int

    pub fn at(self, i: int) -> int:
        return self.n + i
