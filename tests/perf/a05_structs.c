#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
typedef struct { double x, y, vx, vy; } Particle;
int main(void) {
    size_t n = 100000;
    Particle* ps = (Particle*)malloc(n * sizeof *ps);
    for (size_t i = 0; i < n; i++) ps[i] = (Particle){ (double)i, 0.0, 1.0, 0.5 };
    for (int step = 0; step < 2000; step++)
        for (size_t i = 0; i < n; i++) { ps[i].x = ps[i].x + ps[i].vx; ps[i].y = ps[i].y + ps[i].vy; }
    double total = 0.0;
    for (size_t i = 0; i < n; i++) total = total + ps[i].x + ps[i].y;
    printf("%lld\n", (long long)total);
    return 0;
}
