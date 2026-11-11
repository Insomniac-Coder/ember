# The overlay of ../accept_an_extend_block_adds_wrapper_methods.em: ordinary Ember wrapper methods
# over the header's struct ([FFI-13]).

overlay c "rect.h":
    extend Rect:
        pub fn area() -> i32:
            return this.w * this.h

        pub fn scaled(k: i32) -> Rect:
            return Rect(w=this.w * k, h=this.h * k)
