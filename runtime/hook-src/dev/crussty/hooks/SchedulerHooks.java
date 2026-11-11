package dev.crussty.hooks;

/**
 * Native bridge required by the scheduler-interception transform rules
 * (runtime/src/platform/scheduler.rs). Compiled with --release 8 and
 * embedded in the runtime via include_bytes!; the runtime defines this
 * class in the system class loader at agent start and registers the
 * natives against its own code pointers.
 */
public final class SchedulerHooks {
    private SchedulerHooks() {}

    /** Injected at the top of MinecraftServer.tickServer — one call per main tick. */
    public static native void onTick();

    /** Injected at the top of ServerLevel.tick — one call per dimension per tick. */
    public static native void onLevelTick();

    /** Injected at Paper regionized / CraftScheduler submission entries. */
    public static native void onTaskScheduled();

    /** Injected at the top of LevelTicks.tick — the scheduled block/fluid drain. */
    public static native void onBlockTicks();
}