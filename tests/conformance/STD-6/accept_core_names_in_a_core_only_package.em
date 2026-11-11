#$ test: compile-pass
#$ rules: STD-6
## `[STD-6]` — core is scalars, `Option`, `Result`, views, fixed-capacity containers and math: a
## package that declares `layers = ["core"]` may use all of it, with the error type named.

pub struct SensorError:
    code: int

pub fn read(x: int) -> Result[int, SensorError]:
    if x < 0:
        return Err(SensorError(code=x))
    return Ok(x)

pub fn pick(values: ref [int; 3]) -> Option[int]:
    if values[0] > values[1]:
        return Some(values[0])
    return None
