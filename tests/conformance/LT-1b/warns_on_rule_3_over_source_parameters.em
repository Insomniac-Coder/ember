#$ test: compile-pass
#$ rules: LT-1b, LT-1, MAN-3
# ODR-024 — `L3014` counts source parameters: a borrowed `Array` and `String`
# are two, and so are a static method's; a borrowed receiver takes rule 1, and
# a `mut` `Copy` parameter is no source at all.

fn longer(a: Array[int], b: String) -> Span[int]:    #$ warning[L3014]: return region is the intersection of 2 parameters
    return a

struct Table:
    rows: Array[int]

    fn pick(a: Array[int], b: Array[int]) -> Span[int]:    #$ warning[L3014]: return region is the intersection of 2 parameters
        return a

    fn head(self, other: Array[int]) -> Span[int]:
        return self.rows

fn next_token(mut pos: int, src: str) -> str:
    pos += 1
    return src

fn main():
    pass
