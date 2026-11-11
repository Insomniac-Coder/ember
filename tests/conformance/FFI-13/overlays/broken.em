# The overlay of ../reject_a_wrapper_is_checked_as_ordinary_ember.em.

overlay c "rect.h":
    extend Rect:
        pub fn area() -> i32:
            return this.width * this.h
