#$ test: parse-pass
#$ rules: MOD-1
# Module `inner` (`[MOD-1]`: `inner/mod.em`, as `inner/` has a submodule) of
# ../reject_an_item_and_a_module_of_one_path.em: its `cells` is `inner.cells`, the submodule's path.

pub fn cells() -> int:
    return 1
