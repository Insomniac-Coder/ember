# The overlay of ../reject_a_variant_the_overlay_does_not_name_whole.em: one name for two
# alternatives, and a variant one of whose alternatives does not map by value.

unsafe overlay cpp "unnamed_variants.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(variant=["Only"])
    type Pair
    @ffi(variant=["Number", "Text"])
    type Texty
