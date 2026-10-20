# A module of the FFI-30b tests: it re-exports the header's module.
pub import c "../geom.h" as geom

pub fn origin() -> geom.Point:
    return geom.Point(x=0, y=0)
