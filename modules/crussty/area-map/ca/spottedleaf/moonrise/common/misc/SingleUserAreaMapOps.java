package ca.spottedleaf.moonrise.common.misc;

/**
 * Native-batch apply for {@link SingleUserAreaMap#update(int, int, int)}.
 *
 * The Crussty CE fork replaces the JVM's per-cell difference enumeration
 * (SingleUserAreaMap.update -> addCallback/removeCallback) with a Rust
 * enumeration that emits the same Add/Remove ops and chunk keys into caller
 * buffers, then the unchanged Java apply loop runs them. This class is the
 * Java half of that contract:
 *
 *   1. keep scratch buffers (upper bound: disjoint old+new squares),
 *   2. call PaperNativeAreaMap.nativeUpdateOpsBatch(fromX, fromZ, oldD,
 *      toX, toZ, newD, ops, keys) to enumerate the difference,
 *   3. apply addCallback/removeCallback per returned op.
 *
 * Hot path is allocation-free after warmup: scratch buffers are grow-only
 * per-thread (SingleUserAreaMap is single-user but maps may live on different
 * threads, so a plain static would race; ThreadLocal is bootstrap-visible).
 * The apply loop is deliberately the absolute minimum: per returned op it
 * reads one long + one byte and makes one abstract call — the same shape the
 * kernel's own callbacks already pay.
 *
 * Compile with `--release 8` (major 52) against RuntimeStubs.java so the
 * class loads from the bootstrap loader via DefineClass. Generated artifact:
 * src/area_map_ops.class (rebuild with area-map/build.sh, keep in sync).
 */
public final class SingleUserAreaMapOps {
    private SingleUserAreaMapOps() {}

    private static final class Scratch {
        byte[] ops;
        long[] keys;
    }

    private static final ThreadLocal<Scratch> SCRATCH = new ThreadLocal<Scratch>() {
        @Override
        protected Scratch initialValue() {
            return new Scratch();
        }
    };

    /** Upper bound on the number of difference ops for rect sizes oldD/newD. */
    static int maxOps(int oldD, int newD) {
        long oldSide = 2L * oldD + 1L;
        long newSide = 2L * newD + 1L;
        long cap = oldSide * oldSide + newSide * newSide;
        return cap > Integer.MAX_VALUE ? Integer.MAX_VALUE : (int) cap;
    }

    /**
     * Replace {@code update}'s enumeration: emit the difference natively and
     * apply callbacks from the produced op list. Signature must match the
     * invokestatic emitted by the SingleUserAreaMap.update byte hook.
     */
    static void run(
        SingleUserAreaMap map, int fromX, int fromZ, int oldD,
        int toX, int toZ, int newD, Object param
    ) {
        if (fromX == Integer.MIN_VALUE) {
            return; // never initialized: native enumerates nothing, no callbacks
        }
        int cap = maxOps(oldD, newD);
        Scratch s = SCRATCH.get();
        if (s.ops.length < cap) {
            s.ops = new byte[cap];
            s.keys = new long[cap];
        }
        byte[] ops = s.ops;
        long[] keys = s.keys;
        int n = PaperNativeAreaMap.nativeUpdateOpsBatch(fromX, fromZ, oldD, toX, toZ, newD, ops, keys);
        if (n < 0) {
            return; // native error (buffer too small cannot happen for cap)
        }
        for (int i = 0; i < n; i++) {
            long key = keys[i];
            int x = (int) key;          // chunk_as_long: x in the low 32 bits
            int z = (int) (key >>> 32); // z in the high 32 bits
            if (ops[i] == 0) {
                map.addCallback(param, x, z);    // AreaOp::Add
            } else {
                map.removeCallback(param, x, z); // AreaOp::Remove
            }
        }
    }
}