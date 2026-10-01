## A child interface in another module than its parent, naming the parent's
## associated type, and a type meeting both (D-458).

from support.measured import Measured

pub interface Weighed: Measured:
    fn heaviest(self) -> Unit

pub struct Crate:
    pub weight: int

extend Crate implements Measured:
    type Unit = int

    fn measure(self) -> int:
        return self.weight

extend Crate implements Weighed:
    fn heaviest(self) -> int:
        return self.weight * 3
