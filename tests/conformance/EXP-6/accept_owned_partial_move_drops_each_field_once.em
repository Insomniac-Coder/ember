#$ test: run-pass
#$ rules: EXP-6, OWN-2, OWN-3, DRP-2
# Moving one owned field out of a plain struct leaves the other field owned by
# the struct. The moved field drops with its new owner and MUST NOT be visited
# again by the source struct's scope-end drop. Exact stdout is a destruction-
# count assertion: the old whole-local analysis printed `1, 2, 1`.

struct Part:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Pair:
    pub first: Part
    pub second: Part

fn main():
    pair = Pair(Part(1), Part(2))
    _first = pair.first
#$ stdout: 1
#$ 2
#$ assert-c: contains("em_Part_drop(&_1.second)")
#$ assert-c: !contains("em_Part_drop(&_1.first)")
