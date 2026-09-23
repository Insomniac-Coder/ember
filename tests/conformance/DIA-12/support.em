#$ rules: MOD-3
#$ test: parse-pass
# The qualified nested-path case reaches `support.io` through this module's
# ordinary namespace import; it introduces no alternate name-resolution path.
import support.io
