/* Not a variant: text generation only, so the per-variant work can be read as (variant - gen_only). */
#include "common.h"
int main(void) {
    h_start();
    size_t tn; char *text = make_text(&tn);
    uint64_t s = 0; for (size_t i = 0; i < tn; i += 4096) s += (unsigned char)text[i];
    h_report("a25-gen-only", s + tn);
    return 0;
}
