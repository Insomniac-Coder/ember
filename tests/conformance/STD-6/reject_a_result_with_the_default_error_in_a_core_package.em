#$ test: compile-fail
#$ rules: STD-6
## `[STD-6]` — a package with `layers = ["core"]` that writes `Result[int]` names `AnyError`, a name
## of the alloc layer: `E1042`, naming the layer and the line to write.

pub fn read(x: int) -> Result[int]:    #$ error[E1042]: `Result[int]` means `Result[int, AnyError]`, and `AnyError` needs the `alloc` layer, which `STD-6` does not declare
    return Ok(x)

fn main():
    pass
