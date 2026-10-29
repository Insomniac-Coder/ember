#$ test: run-pass
#$ rules: FFI-6, BLD-FFI-1, FFI-8
#$ warning[L3015]: `geometry_mid` is a C function the program calls in unsafe code
#$ warning[L3015]: `geometry_area` is a C function the program calls in unsafe code
#$ warning[L3015]: `geometry_grow` is a C function the program calls in unsafe code
#$ warning[L3015]: `geometry_shape` is a C function the program calls in unsafe code
#$ warning[L3015]: `geometry_count_tagged` is a C function the program calls in unsafe code
#$ warning[L3015]: `geometry_scale` is a C function the program calls in unsafe code
#$ warning[L3015]: `geometry_polygon` is a C function the program calls in unsafe code
#$ warning[L3015]: `geometry_sides` is a C function the program calls in unsafe code
#$ warning[L3015]: `type` is a C function the program calls in unsafe code
#$ warning[L3015]: `geometry_polygon_free` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 16 mm 2 4
#$ stdout: 64 3 3 1 3
#$ stdout: 1 true true 0 5.0
#$ stdout: 6 4
#$ stdout: 1 3
# `[FFI-6]` — `import c` parses the header with libclang and exposes its functions, structs, enums,
# typedefs, globals and constant macros as a module, bound to `c` here. `[BLD-FFI-1]` —
# `import c "./geometry.c"` compiles `geometry.c` and imports `geometry.h`. `[FFI-8]` — a
# struct is a `@layout(c)` struct built and passed by value, an enum `@non_exhaustive`, an
# incomplete struct an opaque type used behind a pointer, a C name that is an Ember keyword a raw
# identifier.

import c "./geometry.c"

fn main():
    println(c.GEOMETRY_MAX, c.GEOMETRY_UNITS.to_str().unwrap(), c.GEOMETRY_FLAG_B, c.Shape.SHAPE_AREA as i32)
    r = c.Rect(min=c.Point(x=0, y=0), max=c.Point(x=8, y=8), tag=[0, 0, 0, 0])
    a = c.Point(x=1, y=2)
    b = c.Point(x=5, y=5)
    unsafe:  # SAFETY(ffi): every pointer is to a live local, and the polygon is freed once
        m = c.geometry_mid(ref_to_ptr(ref a), ref_to_ptr(ref b))
        println(c.geometry_area(r), m.x, m.y, c.geometry_calls, c.geometry_version)
        c.geometry_grow(ref_to_ptr(ref mut r), 1)
        shape = c.geometry_shape(1)
        println(c.geometry_count_tagged(ref_to_ptr(ref r), 1), shape == c.SHAPE_FIRST, c.geometry_shape(9) != c.Shape.SHAPE_DOT, r.min.x + 1, c.geometry_scale(2.0))
        polygon = c.geometry_polygon(6)
        println(c.geometry_sides(polygon as *c.Polygon), c.r#type(3))
        c.geometry_polygon_free(polygon)
    println(r.tag[0], r.max.y - 6)
