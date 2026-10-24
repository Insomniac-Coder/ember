#!/bin/sh
# summarize.sh <results file>: median ms and median peak KB per program (5 runs), and whether every
# run of every program printed the same checksum as the others of that program.
awk '
/out=\[/ {
  s=$0; sub(/.*out=\[/, "", s); sub(/\].*/, "", s);
  n=split(s, f, " "); name=f[1];
  for (i=2;i<=n;i++) { split(f[i], kv, "="); v[kv[1]]=kv[2] }
  if (!(name in cnt)) names[++k]=name; cnt[name]++; ms[name, cnt[name]]=v["ms"]; kb[name, cnt[name]]=v["peak_kb"];
  if (!(name in ck)) ck[name]=v["checksum"]; else if (ck[name]!=v["checksum"]) bad[name]=1;
}
function median(arr, name, c,   i, j, t, a) {
  for (i=1;i<=c;i++) a[i]=arr[name, i]+0
  for (i=1;i<=c;i++) for (j=i+1;j<=c;j++) if (a[j]<a[i]) { t=a[i]; a[i]=a[j]; a[j]=t }
  return (c%2) ? a[(c+1)/2] : (a[c/2]+a[c/2+1])/2
}
function mn(arr, name, c,   i, m) { m=arr[name,1]+0; for (i=2;i<=c;i++) if (arr[name,i]+0<m) m=arr[name,i]+0; return m }
function mx(arr, name, c,   i, m) { m=arr[name,1]+0; for (i=2;i<=c;i++) if (arr[name,i]+0>m) m=arr[name,i]+0; return m }
END {
  printf "%-28s %4s %10s %10s %10s %10s  %s\n", "program", "runs", "median_ms", "min_ms", "max_ms", "median_kb", "checksum"
  for (i=1;i<=k;i++) { nm=names[i]; c=cnt[nm];
    printf "%-28s %4d %10.3f %10.3f %10.3f %10d  %s%s\n", nm, c, median(ms,nm,c), mn(ms,nm,c), mx(ms,nm,c), median(kb,nm,c), ck[nm], (nm in bad)?" MISMATCH":"" }
}' "$1"
