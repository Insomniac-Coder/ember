#$ test: run-pass
#$ rules: HR-12, HR-12a
#$ profiles: debug, release
#$ stdout: 3 6
#$ assert-c: contains("ember_obj_header_v40, ember_reload_protocol_v1")
#$ assert-c: contains("(uint32_t)sizeof(ember_obj_header)")
# Under `reload = "all"` (this directory's manifest) objects carry the 40-byte header whose last 16
# bytes link them into the live-instance list. The image names the header it was built for by the
# runtime function it links against, so an image and a runtime that disagree fail to link: the
# runtime defines only its own (its header file refuses any other size at compile time).

class Cell:
    value: int = 0

fn fill(n: int) -> Array[Cell]:
    cells: Array[Cell] = []
    for i in 0..n:
        c = Cell()
        c.value = i + 1
        cells.push(c)
    return cells

fn main():
    cells = fill(3)
    sum = 0
    for c in cells:
        sum += c.value
    println(cells.len(), sum)
