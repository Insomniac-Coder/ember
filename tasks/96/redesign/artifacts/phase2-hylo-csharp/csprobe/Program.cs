// Phase 2 probe: what C#'s second-class references allow, refuse, and check at run time.
// Build: dotnet build (offline, empty package source). ERRORS=1 builds the refused cases instead.
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

interface IPool { int Count { get; } }

sealed class Pool<T> : IPool {
    public T[] dense = new T[2];
    public int count;
    public int Count => count;
    public ref T Add(T v) {
        if (count == dense.Length) Array.Resize(ref dense, dense.Length * 2); // the array moves
        dense[count] = v;
        return ref dense[count++];
    }
}

struct Position { public float X, Y; }

sealed class Registry {
    readonly Dictionary<Type, IPool> pools = new();
    Pool<T> PoolOf<T>() {
        if (!pools.TryGetValue(typeof(T), out var p)) { p = new Pool<T>(); pools.Add(typeof(T), p); }
        return (Pool<T>)p;                                  // run-time checked downcast (A13)
    }
    public ref T Emplace<T>(T v) => ref PoolOf<T>().Add(v);
    // A5: a reference into a pool found through a map lookup, returned to the caller. Accepted:
    // the array lives on the GC heap, so its ref-safe-context is the caller's.
    public ref T Get<T>(int i) => ref PoolOf<T>().dense[i];
}

static class Program {
    static void Main() {
        var reg = new Registry();
        reg.Emplace(new Position { X = 1 });
        reg.Emplace(new Position { X = 2 });

        // A2: hold a reference, grow the pool (the array is replaced), write through the reference.
        ref Position p = ref reg.Get<Position>(0);
        reg.Emplace(new Position { X = 3 });                 // Array.Resize: dense is a new array now
        p.X = 100;                                           // writes into the OLD array, kept alive by the GC
        Console.WriteLine($"A2 stale ref: wrote 100, pool now reads {reg.Get<Position>(0).X}");

        // A2 with the library's own view: CollectionsMarshal.AsSpan, then Add.
        var list = new List<int> { 1, 2 };
        Span<int> s = CollectionsMarshal.AsSpan(list);
        for (int k = 0; k < 10; k++) list.Add(k);            // the list's array is replaced
        s[0] = 42;
        Console.WriteLine($"A2 stale span: wrote 42, list[0] = {list[0]}");

        // A3: two mutable references to the same element: no rule against it.
        int[] a = { 5 };
        ref int r1 = ref a[0]; ref int r2 = ref a[0];
        r1 += 1; r2 += 1;
        Console.WriteLine($"A3 aliasing: a[0] = {a[0]}");

        // A4: changing a List while a foreach runs: a run-time version check.
        try { foreach (var x in list) if (x == 1) list.Add(99); }
        catch (InvalidOperationException e) { Console.WriteLine($"A4 run-time check: {e.GetType().Name}"); }

        // A4 by index: allowed; the loop sees the new elements (Python-list-like).
        var spawn = new List<int> { 0 };
        for (int i = 0; i < spawn.Count && i < 5; i++) spawn.Add(i + 1);
        Console.WriteLine($"A4 index loop: {spawn.Count} items");

        // A12: a closure captures a local by reference (the local is hoisted to the heap).
        int counter = 0;
        Action bump = () => counter++;
        bump(); bump();
        Console.WriteLine($"A12 capture: counter = {counter}");

        // A1: a reference to one field while a method changes another field of the same object.
        var w = new World();
        ref int hp = ref w.hp;
        w.Tick();                                            // writes w.ticks; no conflict rule
        hp += 1;
        Console.WriteLine($"A1 whole-object: hp = {w.hp}, ticks = {w.ticks}");
#if ERRORS
        Refused.Run();
#endif
    }
}

sealed class World { public int hp = 10, ticks; public void Tick() { ticks++; } }

#if ERRORS
ref struct View { public Span<int> items; }
#if ERRORS_FIELD
sealed class Holder { public Span<int> items; }              // CS8345: a ref struct field in a class
#endif
static class Refused {
    static ref int LocalRef() { int x = 1; return ref x; }   // CS8168: returning a ref to a local
    static Span<int> StackSpan() { Span<int> b = stackalloc int[4]; return b; } // CS8352
    struct S { int f; public ref int F() => ref f; }         // CS8170: struct 'this' is scoped
    public static void Run() {
        Span<int> sp = new int[3];
        Func<int> g = () => sp[0];                           // CS8175: a ref struct captured by a lambda
        var arr = new View[1];                               // CS0611: array of a ref struct
        List<Span<int>> l = null;                            // CS9244: ref struct as a type argument
    }
}
#endif
