## Half of an import cycle (`[MOD-4]`): the parent imports its child's module.

from support.ring_child import Ranked

pub interface Scored:
    type Score
    fn score(self) -> Score

pub fn top(r: int) -> int:
    return r
