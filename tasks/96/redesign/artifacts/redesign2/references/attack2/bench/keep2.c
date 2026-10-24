/* keep2.c: attack on the first attack's saved version of A22 6.5 ("reuse the temporary's buffer only when the
   result fills at least half of it"). Which "half": of the buffer's length, or of its capacity?
   The Ember function modelled:
       fn read_name(f: File) -> String:
           buf = String.with_capacity(65536)     # a line buffer, as a buffered reader keeps one
           f.read_line(buf)
           return buf.trim()                     # 6.5: the view of a local that ends here becomes the result
   Python: `return f.readline().strip()` keeps a string of the line's size.
     -DHALF_LEN : reuse when the result is at least half the buffer's LENGTH (the line is 20..80 bytes, the
                  trimmed result is most of it, so it is reused, with its 64 KiB capacity)
     -DHALF_CAP : reuse when the result is at least half the buffer's CAPACITY (otherwise an exact copy)
     -DCOPY     : always the exact copy (today's `.to_string()`)
   2,000 names are kept by the caller. Prints the bytes the kept strings hold (sum of capacities). */
#include "harness.h"
#include <string.h>
#define NL 2000u
#define CAP 65536u
typedef struct { char *p; size_t len, cap; } Str;
static Str read_name(void) {
    Str b = { (char *)malloc(CAP), 0, CAP };
    size_t n = 20 + h_rand() % 61, lead = h_rand() % 3, trail = h_rand() % 3;
    for (size_t i = 0; i < n; i++) b.p[i] = (i < lead || i >= n - trail) ? ' ' : (char)('a' + h_rand() % 26);
    b.len = n;
    size_t s = 0, e = n;
    while (s < e && b.p[s] == ' ') s++;
    while (e > s && b.p[e - 1] == ' ') e--;
    size_t k = e - s;
#if defined(HALF_LEN)
    int reuse = 2 * k >= b.len;
#elif defined(HALF_CAP)
    int reuse = 2 * k >= b.cap;
#else
    int reuse = 0;
#endif
    if (reuse) { memmove(b.p, b.p + s, k); b.len = k; return b; }
    Str c = { (char *)malloc(k ? k : 1), k, k }; memcpy(c.p, b.p + s, k); free(b.p);
    return c;
}
int main(void) {
    h_start(); h_seed(9);
    Str *names = (Str *)malloc(NL * sizeof(Str));
    for (uint32_t i = 0; i < NL; i++) names[i] = read_name();
    uint64_t sum = 0, kept = 0;
    for (uint32_t i = 0; i < NL; i++) { sum += names[i].len * 31 + (uint8_t)names[i].p[0]; kept += names[i].cap; }
    PROCESS_MEMORY_COUNTERS pmc; GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof pmc);
    /* peak commit: memory the process holds whether or not it touched it (a 64 KiB buffer touched on one page) */
    printf("kept_bytes=%llu commit_peak_kb=%llu ", (unsigned long long)kept, (unsigned long long)(pmc.PeakPagefileUsage / 1024));
#if defined(HALF_LEN)
    h_report("keep2-half-of-length", sum);
#elif defined(HALF_CAP)
    h_report("keep2-half-of-capacity", sum);
#else
    h_report("keep2-copy", sum);
#endif
    for (uint32_t i = 0; i < NL; i++) free(names[i].p);
    return 0;
}
