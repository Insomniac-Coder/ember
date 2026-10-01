## The other half: a child naming its parent's associated type across the
## cycle (D-458).

from support.ring_parent import Scored

pub interface Ranked: Scored:
    fn best(self) -> Score
