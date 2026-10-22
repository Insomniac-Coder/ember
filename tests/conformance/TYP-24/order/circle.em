## A type implementing `order.named.Named`, in a module loaded after it.

import order.named

pub struct Circle:
    pub radius: int

extend Circle implements named.Named:
    fn name(self) -> str:
        return "circle"
