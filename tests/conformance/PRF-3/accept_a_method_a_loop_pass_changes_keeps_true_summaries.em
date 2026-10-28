#$ test: run-pass
#$ rules: PRF-3, CLS-7
#$ profiles: debug, development, release
#$ stdout: 3
# D-920, `[PRF-3]` — with the rewrites a profile turns on, the callable-region summaries describe the
# MIR as it is when they are checked: a loop pass that changes a body after they were made (here a
# method's copies propagated) has them made again. The compiler panicked here ("callable-region
# metadata disagrees with the MIR body") in `development` and `release`.

struct Rng:
    state: int

    fn next_float(mut self) -> float:
        self.state = (self.state * 1103515245 + 12345) % 2147483648
        return self.state as float / 2147483648.0

    fn range(mut self, low: float, high: float) -> float:
        return low + (high - low) * self.next_float()

class Fountain:
    rng: Rng = Rng(state=7)
    count: int = 0

    fn spawn(self, count: int):
        for _ in 0..count:
            if self.rng.range(-1.0, 1.0) < 2.0:
                self.count += 1

fn main():
    fountain = Fountain()
    fountain.spawn(3)
    println(fountain.count)
