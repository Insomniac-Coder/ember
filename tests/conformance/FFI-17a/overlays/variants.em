# The overlay of ../accept_a_variant_crosses_as_an_enum.em: each variant's alternatives named
# (ODR-198), and each call asserted safe.

unsafe overlay cpp "variants.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(variant=["Round", "Boxy"])
    type Shape
    @ffi(variant=["Whole", "Fraction"])
    type Number
    fn area(shape)
    fn bigger(shape)
    fn half(n)
    fn kind_of(number)
