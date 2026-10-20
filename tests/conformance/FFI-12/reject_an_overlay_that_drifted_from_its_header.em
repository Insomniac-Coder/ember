#$ test: compile-fail
#$ rules: FFI-12
#$ error[E5010]: `tally.h` declares no function `tally_total`
#$ error[E5010]: parameter `n` of `tally_sum` is `count` in `tally.h`
#$ error[E5010]: `tally_scale` takes 2 parameters in `tally.h`, and the entry names 1
#$ error[E5010]: `tally.h` declares no opaque type `Counter`
#$ error[E5010]: `tally.h` declares no `tally_reset` to hide
# `[FFI-12]` — an overlay entry whose name, parameter count or parameter names do not match the
# header is `E5010`, so an overlay cannot drift from the header it describes.

import c "tally.h" with (overlay = "overlays/drifted.em")

fn main():
    pass
