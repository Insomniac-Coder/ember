# A module of the FFI-30b tests: its import of the header is its own.
import c "../geom.h" as geom

pub fn origin_sum() -> i32:
    unsafe:  # SAFETY(ffi): a by-value call
        return geom.geom_sum(geom.Point(x=0, y=0))
