# Diagnostic only (not a measurement in the report's tables): copies a W2 model and makes it print
# build / walk / re-parent times to stderr. Usage: python -I add_timers.py <model.c> <out.c>
import sys, re
NL = chr(92) + "n"
src = open(sys.argv[1]).read()
src = src.replace('#include "../ember_model.h"', '#include "ember_model.h"')
src = src.replace("    uint64_t depth = 0;",
    "    LARGE_INTEGER qa, qf; QueryPerformanceFrequency(&qf); QueryPerformanceCounter(&qa);\n"
    "    double t_build = (double)(qa.QuadPart - h_t0.QuadPart) * 1000.0 / qf.QuadPart, t_walk = 0, t_rep = 0;\n"
    "    uint64_t depth = 0;", 1)
lines = src.split("\n")
out = []
i = 0
while i < len(lines):
    l = lines[i]
    if re.match(r"\s+for \(.*(Node \*n = nodes;|uint32_t j = 0|size_t j = 0)", l):
        ind = re.match(r"(\s*)", l).group(1)
        out.append(ind + "{ LARGE_INTEGER s; QueryPerformanceCounter(&s);")
        j = i
        while True:
            out.append(lines[j])
            if "depth++" in lines[j]:
                break
            j += 1
        # the walk may continue with closing braces at deeper indents
        while lines[j + 1].strip() == "}" and len(lines[j + 1]) - len(lines[j + 1].lstrip()) > len(ind):
            j += 1
            out.append(lines[j])
        out.append(ind + "LARGE_INTEGER e; QueryPerformanceCounter(&e); t_walk += (double)(e.QuadPart - s.QuadPart) * 1000.0 / qf.QuadPart; }")
        out.append(ind + "{ LARGE_INTEGER s; QueryPerformanceCounter(&s);")
        k = j + 1
        depth = 0
        while True:
            out.append(lines[k])
            depth += lines[k].count("{") - lines[k].count("}")
            if depth == 0 and "}" in lines[k]:
                break
            k += 1
        out.append(ind + "LARGE_INTEGER e; QueryPerformanceCounter(&e); t_rep += (double)(e.QuadPart - s.QuadPart) * 1000.0 / qf.QuadPart; }")
        i = k + 1
        continue
    out.append(l)
    i += 1
src = "\n".join(out)
src = src.replace("    uint64_t sum = depth;",
    '    fprintf(stderr, "build=%.1f walk=%.1f reparent=%.1f' + NL + '", t_build, t_walk, t_rep);\n'
    "    uint64_t sum = depth;", 1)
open(sys.argv[2], "w").write(src)
