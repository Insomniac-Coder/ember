#$ test: compile-fail
#$ rules: BLD-11, STD-6
## `[BLD-11]`, `[STD-6]` — a package that declares `layers = ["core"]` links only `core`: an `Array`
## (the allocator's) is `E1021`, naming the layer and the call chain, and the help is to add the layer
## to the package's layers or do without it. This directory's manifest declares `["core"]`.

pub fn readings() -> int:
    values = [1, 2, 3]    #$ error[E1021]: `readings` needs the `alloc` layer, which is not linked in this build
    values.push(owns 4)
    return values.len()
