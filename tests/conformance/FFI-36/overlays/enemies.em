# The overlay of ../accept_adopt_ends_through_the_overlays_destroys.em: the function that ends an
# `Enemy`, whose destructor is not public.

unsafe overlay cpp "enemies.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(destroys(destroy_enemy))
    type Enemy
