# A14: a reference to an inline local of the frame held across yield (must stay refused if frames move).
gen fn counter() -> Generator[int]:
    n = 5
    r = ref n
    yield 1
    yield r + 1

fn main():
    for x in counter():
        println(x)
