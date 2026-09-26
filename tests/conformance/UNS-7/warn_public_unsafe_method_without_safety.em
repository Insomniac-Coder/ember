#$ test: compile-pass
#$ rules: UNS-7
#$ profiles: debug
#$ warning[L3015]: undocumented unsafe obligation

struct Counter:
    value: i32

extend Counter:
    pub unsafe fn next(self) -> i32:
        return self.value + 1

fn main():
    pass
