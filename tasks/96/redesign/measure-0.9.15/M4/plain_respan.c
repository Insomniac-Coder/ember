/* plain_respan.c: workload respan, plain C (link with pkg_respan.c). A caller walks 4,096 positions
   in chunks of 16; for each chunk it calls the other package's `integrate_c(pos, vel, n, dt)` and then
   wraps the chunk's positions into [-1000, 1000] itself. 117,188 frames of 256 chunks: 30,000,128 calls.

   What a careful C programmer writes: the chunk's pointer and length stay in the caller's registers
   across the call (C passes them by value; the callee cannot change them). -DRUNTIME_LEN reads the
   chunk length from a volatile, so it is not known when compiling (the default knows it is 16). */
#include "m4.h"

#define POOL 4096
#define CHUNK 16
#define FRAMES 117188

void integrate_c(float *RESTRICT pos, const float *vel, int64_t n, float dt);

static float pos[POOL], vel[POOL];
#ifdef RUNTIME_LEN
static volatile int64_t chunk_len = CHUNK;
#define NAME "plain_respan_rtlen"
#else
#define NAME "plain_respan"
#endif

int main(void) {
    h_start();
#ifdef RUNTIME_LEN
    int64_t len = chunk_len;
#else
    const int64_t len = CHUNK;
#endif
    h_seed(37);
    for (int i = 0; i < POOL; i++) { pos[i] = (float)((int)(h_rand() % 2001) - 1000); vel[i] = (float)((int)(h_rand() % 2001) - 1000) * 0.01f; }
    const float dt = 0.01f;
    for (int f = 0; f < FRAMES; f++) {
        for (int64_t c = 0; c < POOL / CHUNK; c++) {
            float *p = &pos[c * CHUNK];
            integrate_c(p, &vel[c * CHUNK], len, dt);
            for (int64_t i = 0; i < len; i++) {
                float x = p[i];
                if (x > 1000.0f) x -= 2000.0f; else if (x < -1000.0f) x += 2000.0f;
                p[i] = x;
            }
        }
    }
    uint64_t s = 0;
    for (int i = 0; i < POOL; i++) s += (uint64_t)(int64_t)(pos[i] * 1000.0f);
    h_report(NAME, s);
    return 0;
}
