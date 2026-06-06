package dev.codex.compatprobe;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.Deque;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicInteger;
import org.bukkit.Bukkit;
import org.bukkit.GameMode;
import org.bukkit.Location;
import org.bukkit.Material;
import org.bukkit.World;
import org.bukkit.command.Command;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.EntityType;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.block.BlockBreakEvent;
import org.bukkit.event.block.BlockPlaceEvent;
import org.bukkit.event.entity.EntityDamageByEntityEvent;
import org.bukkit.event.player.PlayerAnimationEvent;
import org.bukkit.event.player.PlayerCommandPreprocessEvent;
import org.bukkit.event.player.PlayerInteractEvent;
import org.bukkit.event.player.PlayerItemHeldEvent;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;
import org.bukkit.event.player.PlayerToggleSneakEvent;
import org.bukkit.event.player.PlayerToggleSprintEvent;
import org.bukkit.event.server.PluginEnableEvent;
import org.bukkit.event.server.ServerLoadEvent;
import org.bukkit.plugin.RegisteredServiceProvider;
import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.inventory.ItemStack;
import java.util.stream.Collectors;

public final class CompatProbePlugin extends JavaPlugin implements Listener {
    private static final int ARENA_CHUNK_LOAD_DEFAULT_MAX_IN_FLIGHT = 4;

    private final AtomicInteger eventSequence = new AtomicInteger();
    private final AtomicInteger blockPlaceEvents = new AtomicInteger();
    private final AtomicInteger blockBreakEvents = new AtomicInteger();
    private final AtomicInteger arenaCommands = new AtomicInteger();
    private final AtomicInteger arenaPreparedPlayers = new AtomicInteger();
    private final AtomicInteger arenaSkippedPlayers = new AtomicInteger();
    private final AtomicInteger arenaChunkLoadsQueued = new AtomicInteger();
    private final AtomicInteger arenaChunkLoadsStarted = new AtomicInteger();
    private final AtomicInteger arenaChunkLoadsCompleted = new AtomicInteger();
    private final AtomicInteger arenaChunkLoadsFailed = new AtomicInteger();
    private final AtomicInteger arenaChunkLoadMerges = new AtomicInteger();
    private final AtomicInteger arenaPreloadCommands = new AtomicInteger();
    private final AtomicInteger arenaPreloadChunkLoadsQueued = new AtomicInteger();
    private final AtomicInteger arenaPreloadChunkLoadsStarted = new AtomicInteger();
    private final AtomicInteger arenaPreloadChunkLoadsCompleted = new AtomicInteger();
    private final AtomicInteger arenaPreloadChunkLoadsFailed = new AtomicInteger();
    private final AtomicInteger mobStormCommands = new AtomicInteger();
    private final AtomicInteger mobStormSpawned = new AtomicInteger();
    private final AtomicInteger compatProbeCommands = new AtomicInteger();
    private final AtomicInteger commandPreprocessEvents = new AtomicInteger();
    private final AtomicInteger itemHeldEvents = new AtomicInteger();
    private final AtomicInteger animationEvents = new AtomicInteger();
    private final AtomicInteger interactEvents = new AtomicInteger();
    private final AtomicInteger entityDamageEvents = new AtomicInteger();
    private final AtomicInteger toggleSprintEvents = new AtomicInteger();
    private final AtomicInteger toggleSneakEvents = new AtomicInteger();
    private final Map<UUID, ArenaPreparation> arenaPreparations = new HashMap<>();
    private final Map<UUID, ArenaPreparation> pendingArenaPreparations = new HashMap<>();
    private final Deque<ArenaChunkLoadRequest> arenaChunkLoadQueue = new ArrayDeque<>();
    private final Map<ArenaChunk, ArenaChunkLoadRequest> pendingArenaChunkLoads = new HashMap<>();
    private final Deque<ArenaPreloadRequest> arenaPreloadQueue = new ArrayDeque<>();
    private final Set<ArenaChunk> arenaPreloadTickets = new HashSet<>();
    private int arenaChunkLoadsInFlight;
    private int arenaPreloadLoadsInFlight;
    private int arenaPreloadMaxInFlight = ARENA_CHUNK_LOAD_DEFAULT_MAX_IN_FLIGHT;
    private int arenaPreloadBatchSequence;

    @Override
    public void onEnable() {
        getLogger().info("COMPAT_PROBE lifecycle=enable");
        getServer().getPluginManager().registerEvents(this, this);
        getServer().getScheduler().runTask(this, () -> getLogger().info("COMPAT_PROBE scheduler=sync ticked=true"));
        getServer().getScheduler().runTaskAsynchronously(this, () -> getLogger().info("COMPAT_PROBE scheduler=async ticked=true"));
        getServer().getServicesManager().getRegistrations(this).forEach(provider ->
            getLogger().info("COMPAT_PROBE service_own=" + provider.getService().getName())
        );
        getServer().getScheduler().runTaskTimer(this, this::logMetrics, 100L, 100L);
    }

    @Override
    public void onDisable() {
        for (ArenaChunk chunk : arenaPreloadTickets) {
            try {
                chunk.world().removePluginChunkTicket(chunk.chunkX(), chunk.chunkZ(), this);
            } catch (RuntimeException ignored) {
            }
        }
        arenaPreloadTickets.clear();
        getLogger().info("COMPAT_PROBE lifecycle=disable");
    }

    @EventHandler
    public void onServerLoad(ServerLoadEvent event) {
        logEvent("ServerLoadEvent", event.getType().name());
    }

    @EventHandler
    public void onPluginEnable(PluginEnableEvent event) {
        logEvent("PluginEnableEvent", event.getPlugin().getName());
    }

    @EventHandler
    public void onPlayerJoin(PlayerJoinEvent event) {
        logEvent("PlayerJoinEvent", event.getPlayer().getName());
    }

    @EventHandler
    public void onPlayerQuit(PlayerQuitEvent event) {
        arenaPreparations.remove(event.getPlayer().getUniqueId());
        pendingArenaPreparations.remove(event.getPlayer().getUniqueId());
        logEvent("PlayerQuitEvent", event.getPlayer().getName());
    }

    @EventHandler(priority = EventPriority.MONITOR)
    public void onBlockPlace(BlockPlaceEvent event) {
        int places = blockPlaceEvents.incrementAndGet();
        logBlockEvent("place", places, blockBreakEvents.get(), event.isCancelled(), event.getPlayer().getName(), places);
    }

    @EventHandler(priority = EventPriority.MONITOR)
    public void onBlockBreak(BlockBreakEvent event) {
        int breaks = blockBreakEvents.incrementAndGet();
        logBlockEvent("break", blockPlaceEvents.get(), breaks, event.isCancelled(), event.getPlayer().getName(), breaks);
    }

    @EventHandler
    public void onPlayerCommandPreprocess(PlayerCommandPreprocessEvent event) {
        commandPreprocessEvents.incrementAndGet();
    }

    @EventHandler
    public void onPlayerItemHeld(PlayerItemHeldEvent event) {
        itemHeldEvents.incrementAndGet();
    }

    @EventHandler
    public void onPlayerAnimation(PlayerAnimationEvent event) {
        animationEvents.incrementAndGet();
    }

    @EventHandler
    public void onPlayerInteract(PlayerInteractEvent event) {
        interactEvents.incrementAndGet();
    }

    @EventHandler
    public void onEntityDamageByEntity(EntityDamageByEntityEvent event) {
        entityDamageEvents.incrementAndGet();
    }

    @EventHandler
    public void onPlayerToggleSprint(PlayerToggleSprintEvent event) {
        toggleSprintEvents.incrementAndGet();
    }

    @EventHandler
    public void onPlayerToggleSneak(PlayerToggleSneakEvent event) {
        toggleSneakEvents.incrementAndGet();
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!command.getName().equalsIgnoreCase("compatprobe")) {
            return false;
        }
        compatProbeCommands.incrementAndGet();
        if (args.length > 0 && args[0].equalsIgnoreCase("spectator")) {
            String prefix = args.length > 1 ? args[1] : "";
            Double teleportAltitude = null;
            if (args.length > 2) {
                try {
                    teleportAltitude = Double.valueOf(args[2]);
                } catch (NumberFormatException ignored) {
                    sender.sendMessage("COMPAT_PROBE spectator error=invalid_altitude value=" + args[2]);
                    return true;
                }
            }
            int changed = 0;
            int matched = 0;
            int teleported = 0;
            for (Player player : Bukkit.getOnlinePlayers()) {
                if (!prefix.isEmpty() && !player.getName().startsWith(prefix)) {
                    continue;
                }
                matched++;
                if (player.getGameMode() != GameMode.SPECTATOR) {
                    player.setGameMode(GameMode.SPECTATOR);
                    changed++;
                }
                if (teleportAltitude != null && Math.abs(player.getLocation().getY() - teleportAltitude.doubleValue()) > 0.5D) {
                    var target = player.getLocation();
                    target.setY(teleportAltitude.doubleValue());
                    if (player.teleport(target)) {
                        player.setFallDistance(0.0F);
                        teleported++;
                    }
                }
            }
            sender.sendMessage("COMPAT_PROBE spectator prefix=" + prefix + " matched=" + matched + " changed=" + changed + " teleported=" + teleported);
            return true;
        }
        if (args.length > 0 && args[0].equalsIgnoreCase("arena-preload")) {
            int expectedPlayers = parseIntArg(args, 1, Bukkit.getOnlinePlayers().size());
            int centerX = parseIntArg(args, 2, 0);
            int targetY = parseIntArg(args, 3, 160);
            int centerZ = parseIntArg(args, 4, 0);
            int spacing = Math.max(1, parseIntArg(args, 5, 4));
            int columns = Math.max(1, parseIntArg(args, 6, Math.max(1, (int) Math.ceil(Math.sqrt(Math.max(1, expectedPlayers))))));
            String materialName = args.length > 7 ? args[7] : "stone";
            int radiusChunks = Math.max(0, parseIntArg(args, 8, 0));
            int maxInFlight = Math.max(1, parseIntArg(args, 9, ARENA_CHUNK_LOAD_DEFAULT_MAX_IN_FLIGHT));
            Material material = Material.matchMaterial(materialName);
            if (material == null) {
                sender.sendMessage("COMPAT_PROBE arena_preload error=invalid_material value=" + materialName);
                return true;
            }
            World world = Bukkit.getWorlds().isEmpty() ? null : Bukkit.getWorlds().get(0);
            if (world == null) {
                sender.sendMessage("COMPAT_PROBE arena_preload error=no_world");
                return true;
            }
            preloadArenaChunks(sender, world, expectedPlayers, centerX, targetY, centerZ, spacing, columns, material, radiusChunks, maxInFlight);
            return true;
        }
        if (args.length > 0 && args[0].equalsIgnoreCase("arena")) {
            String prefix = args.length > 1 ? args[1] : "";
            int expectedPlayers = parseIntArg(args, 2, Bukkit.getOnlinePlayers().size());
            int centerX = parseIntArg(args, 3, 0);
            int targetY = parseIntArg(args, 4, 160);
            int centerZ = parseIntArg(args, 5, 0);
            int spacing = Math.max(1, parseIntArg(args, 6, 4));
            int columns = Math.max(1, parseIntArg(args, 7, Math.max(1, (int) Math.ceil(Math.sqrt(Math.max(1, expectedPlayers))))));
            String materialName = args.length > 8 ? args[8] : "stone";
            Material material = Material.matchMaterial(materialName);
            if (material == null) {
                sender.sendMessage("COMPAT_PROBE arena error=invalid_material value=" + materialName);
                return true;
            }

            List<Player> players = Bukkit.getOnlinePlayers()
                .stream()
                .map(Player.class::cast)
                .filter(player -> prefix.isEmpty() || player.getName().startsWith(prefix))
                .sorted(Comparator.comparing(Player::getName, String.CASE_INSENSITIVE_ORDER))
                .collect(Collectors.toCollection(ArrayList::new));
            int rows = Math.max(1, (int) Math.ceil((double) Math.max(expectedPlayers, players.size()) / columns));
            int startX = centerX - ((columns - 1) * spacing) / 2;
            int startZ = centerZ - ((rows - 1) * spacing) / 2;
            int skipped = 0;
            int indexed = 0;
            int fallbackIndexed = 0;
            int nextFallbackIndex = 0;
            Set<Integer> usedArenaIndexes = new HashSet<>();
            List<ArenaTarget> targets = new ArrayList<>();
            Map<ArenaChunk, List<ArenaTarget>> targetsByChunk = new HashMap<>();
            int pending = 0;

            for (Player player : players) {
                Integer parsedIndex = parseArenaIndex(player.getName(), prefix, expectedPlayers);
                int arenaIndex;
                if (parsedIndex != null && usedArenaIndexes.add(parsedIndex)) {
                    arenaIndex = parsedIndex;
                    indexed++;
                } else {
                    while (usedArenaIndexes.contains(nextFallbackIndex)) {
                        nextFallbackIndex++;
                    }
                    arenaIndex = nextFallbackIndex;
                    usedArenaIndexes.add(arenaIndex);
                    fallbackIndexed++;
                }
                int column = arenaIndex % columns;
                int row = arenaIndex / columns;
                int blockX = startX + column * spacing;
                int blockZ = startZ + row * spacing;
                World world = player.getWorld();
                int standY = targetY + 1;
                if (!needsArenaPrepare(player, world, material, blockX, targetY, blockZ, standY)) {
                    skipped++;
                    continue;
                }
                ArenaPreparation targetPreparation = new ArenaPreparation(world.getUID(), blockX, targetY, blockZ, material);
                ArenaPreparation pendingPreparation = pendingArenaPreparations.get(player.getUniqueId());
                if (pendingPreparation != null && pendingPreparation.matches(world.getUID(), blockX, targetY, blockZ, material)) {
                    pending++;
                    continue;
                }
                pendingArenaPreparations.put(player.getUniqueId(), targetPreparation);
                ArenaTarget target = new ArenaTarget(player.getUniqueId(), world, blockX, targetY, blockZ, material);
                targets.add(target);
                targetsByChunk.computeIfAbsent(new ArenaChunk(world, blockX >> 4, blockZ >> 4), ignored -> new ArrayList<>()).add(target);
            }
            final int matchedPlayers = players.size();
            final int skippedPlayers = skipped;
            final int pendingPlayers = pending;
            final int indexedPlayers = indexed;
            final int fallbackIndexedPlayers = fallbackIndexed;
            ArenaCommandContext arenaContext = new ArenaCommandContext(
                prefix,
                matchedPlayers,
                indexedPlayers,
                fallbackIndexedPlayers,
                expectedPlayers,
                centerX,
                centerZ,
                targetY,
                spacing,
                columns,
                material
            );
            arenaCommands.incrementAndGet();
            arenaSkippedPlayers.addAndGet(skipped);
            if (targets.isEmpty()) {
                sender.sendMessage(
                    "COMPAT_PROBE arena prefix=" + prefix
                        + " matched=" + matchedPlayers
                        + " prepared=0"
                        + " teleported=0"
                        + " skipped=" + skipped
                        + " pending=" + pending
                        + " indexed=" + indexed
                        + " fallbackIndexed=" + fallbackIndexed
                        + " expectedPlayers=" + expectedPlayers
                        + " centerX=" + centerX
                        + " centerZ=" + centerZ
                        + " targetY=" + targetY
                        + " spacing=" + spacing
                        + " columns=" + columns
                        + " material=" + material.getKey().getKey()
                );
                return true;
            }

            List<ArenaTarget> readyTargets = new ArrayList<>();
            int alreadyLoadedChunks = 0;
            int queuedChunkLoads = 0;
            int mergedChunkLoads = 0;
            try {
                for (Map.Entry<ArenaChunk, List<ArenaTarget>> entry : targetsByChunk.entrySet()) {
                    ArenaChunk chunk = entry.getKey();
                    if (chunk.world().isChunkLoaded(chunk.chunkX(), chunk.chunkZ())) {
                        alreadyLoadedChunks++;
                        readyTargets.addAll(entry.getValue());
                        continue;
                    }
                    if (enqueueArenaChunkLoad(new ArenaChunkLoadRequest(sender, arenaContext, chunk, List.copyOf(entry.getValue())))) {
                        queuedChunkLoads++;
                    } else {
                        mergedChunkLoads++;
                    }
                }
            } catch (RuntimeException exception) {
                for (ArenaTarget target : targets) {
                    clearPendingArenaPreparation(target);
                }
                sender.sendMessage(
                        "COMPAT_PROBE arena error=async_chunk_load_submit_failed"
                        + " matched=" + matchedPlayers
                        + " scheduled=" + targets.size()
                        + " chunks=" + targetsByChunk.size()
                        + " reason=" + exception.getClass().getSimpleName()
                );
                return true;
            }

            if (!readyTargets.isEmpty()) {
                completeArena(
                    sender,
                    arenaContext,
                    readyTargets,
                    skippedPlayers,
                    pendingPlayers
                );
            }

            if (queuedChunkLoads > 0) {
                pumpArenaChunkLoads();
            }

            sender.sendMessage(
                "COMPAT_PROBE arena scheduled prefix=" + prefix
                    + " matched=" + matchedPlayers
                    + " scheduled=" + targets.size()
                    + " skipped=" + skipped
                    + " pending=" + pending
                    + " chunks=" + targetsByChunk.size()
                    + " asyncLoads=" + queuedChunkLoads
                    + " mergedLoads=" + mergedChunkLoads
                    + " alreadyLoaded=" + alreadyLoadedChunks
                    + " queueDepth=" + arenaChunkLoadQueue.size()
                    + " inFlight=" + arenaChunkLoadsInFlight
                    + " maxInFlight=" + ARENA_CHUNK_LOAD_DEFAULT_MAX_IN_FLIGHT
                    + " indexed=" + indexed
                    + " fallbackIndexed=" + fallbackIndexed
                    + " expectedPlayers=" + expectedPlayers
                    + " centerX=" + centerX
                    + " centerZ=" + centerZ
                    + " targetY=" + targetY
                    + " spacing=" + spacing
                    + " columns=" + columns
                    + " material=" + material.getKey().getKey()
            );
            return true;
        }
        if (args.length > 0 && args[0].equalsIgnoreCase("mobstorm")) {
            String entityName = args.length > 1 ? args[1] : "zombie";
            EntityType entityType = EntityType.fromName(entityName);
            if (entityType == null || !entityType.isSpawnable()) {
                sender.sendMessage("COMPAT_PROBE mobstorm error=invalid_entity value=" + entityName);
                return true;
            }
            int count = Math.max(0, parseIntArg(args, 2, 0));
            int centerX = parseIntArg(args, 3, 0);
            int targetY = parseIntArg(args, 4, Integer.MIN_VALUE);
            int centerZ = parseIntArg(args, 5, 0);
            int spacing = Math.max(1, parseIntArg(args, 6, 2));
            int columns = Math.max(1, parseIntArg(args, 7, Math.max(1, (int) Math.ceil(Math.sqrt(Math.max(1, count))))));
            World world = Bukkit.getWorlds().isEmpty() ? null : Bukkit.getWorlds().get(0);
            if (world == null) {
                sender.sendMessage("COMPAT_PROBE mobstorm error=no_world");
                return true;
            }
            int rows = Math.max(1, (int) Math.ceil((double) Math.max(1, count) / columns));
            int startX = centerX - ((columns - 1) * spacing) / 2;
            int startZ = centerZ - ((rows - 1) * spacing) / 2;
            List<MobStormSpawn> spawns = new ArrayList<>(count);
            Set<Long> chunkKeys = new HashSet<>();
            for (int idx = 0; idx < count; idx++) {
                int column = idx % columns;
                int row = idx / columns;
                int blockX = startX + column * spacing;
                int blockZ = startZ + row * spacing;
                spawns.add(new MobStormSpawn(blockX, blockZ));
                chunkKeys.add(chunkKey(blockX >> 4, blockZ >> 4));
            }

            if (targetY == Integer.MIN_VALUE) {
                List<CompletableFuture<?>> chunkLoads = new ArrayList<>(chunkKeys.size());
                for (long key : chunkKeys) {
                    chunkLoads.add(world.getChunkAtAsync(unpackChunkX(key), unpackChunkZ(key), true));
                }
                CompletableFuture.allOf(chunkLoads.toArray(CompletableFuture[]::new)).whenComplete((ignored, throwable) -> {
                    if (throwable != null) {
                        Bukkit.getScheduler().runTask(this, () ->
                            sender.sendMessage("COMPAT_PROBE mobstorm error=async_chunk_load_failed requested=" + count + " chunks=" + chunkKeys.size())
                        );
                        return;
                    }
                    Bukkit.getScheduler().runTask(this, () ->
                        completeMobStorm(sender, world, entityType, spawns, Integer.MIN_VALUE, count, centerX, centerZ, spacing, columns)
                    );
                });
                sender.sendMessage(
                    "COMPAT_PROBE mobstorm scheduled entity=" + entityType.getKey().getKey()
                        + " requested=" + count
                        + " chunks=" + chunkKeys.size()
                        + " centerX=" + centerX
                        + " centerZ=" + centerZ
                        + " targetY=surface"
                        + " spacing=" + spacing
                        + " columns=" + columns
                );
                return true;
            }

            completeMobStorm(sender, world, entityType, spawns, targetY, count, centerX, centerZ, spacing, columns);
            return true;
        }
        int registrations = 0;
        for (RegisteredServiceProvider<?> ignored : getServer().getServicesManager().getRegistrations(this)) {
            registrations++;
        }
        sender.sendMessage("COMPAT_PROBE command=ok events=" + eventSequence.get() + " ownServices=" + registrations);
        return true;
    }

    private void preloadArenaChunks(
        CommandSender sender,
        World world,
        int expectedPlayers,
        int centerX,
        int targetY,
        int centerZ,
        int spacing,
        int columns,
        Material material,
        int radiusChunks,
        int maxInFlight
    ) {
        arenaPreloadMaxInFlight = maxInFlight;
        Set<ArenaChunk> chunks = computeArenaChunks(world, expectedPlayers, centerX, centerZ, spacing, columns, radiusChunks);
        int alreadyLoaded = 0;
        int queued = 0;
        int id = ++arenaPreloadBatchSequence;
        ArenaPreloadBatch batch = new ArenaPreloadBatch(
            id,
            sender,
            expectedPlayers,
            chunks.size(),
            centerX,
            centerZ,
            targetY,
            spacing,
            columns,
            material,
            radiusChunks,
            maxInFlight
        );

        arenaPreloadCommands.incrementAndGet();
        for (ArenaChunk chunk : chunks) {
            if (chunk.world().isChunkLoaded(chunk.chunkX(), chunk.chunkZ())) {
                alreadyLoaded++;
                addArenaPreloadTicket(chunk);
                batch.recordLoaded();
                continue;
            }
            arenaPreloadQueue.addLast(new ArenaPreloadRequest(batch, chunk));
            arenaPreloadChunkLoadsQueued.incrementAndGet();
            queued++;
        }

        sender.sendMessage(
            "COMPAT_PROBE arena_preload scheduled"
                + " id=" + id
                + " expectedPlayers=" + expectedPlayers
                + " radiusChunks=" + radiusChunks
                + " maxInFlight=" + maxInFlight
                + " chunks=" + chunks.size()
                + " asyncLoads=" + queued
                + " alreadyLoaded=" + alreadyLoaded
                + " queueDepth=" + arenaPreloadQueue.size()
                + " inFlight=" + arenaPreloadLoadsInFlight
                + " maxInFlight=" + arenaPreloadMaxInFlight
                + " centerX=" + centerX
                + " centerZ=" + centerZ
                + " targetY=" + targetY
                + " spacing=" + spacing
                + " columns=" + columns
                + " material=" + material.getKey().getKey()
        );

        if (queued == 0) {
            batch.sendComplete();
            return;
        }
        pumpArenaPreloads();
    }

    private void completeArena(
        CommandSender sender,
        ArenaCommandContext context,
        List<ArenaTarget> targets,
        int initialSkipped,
        int initialPending
    ) {
        int prepared = 0;
        int teleported = 0;
        int skipped = initialSkipped + initialPending;
        int unavailable = 0;
        int stale = 0;
        int unloaded = 0;

        for (ArenaTarget target : targets) {
            Player player = Bukkit.getPlayer(target.playerId());
            if (player == null || !player.isOnline() || player.getWorld() != target.world()) {
                clearPendingArenaPreparation(target);
                unavailable++;
                continue;
            }

            ArenaPreparation pendingPreparation = pendingArenaPreparations.get(target.playerId());
            if (pendingPreparation == null || !pendingPreparation.matches(target.world().getUID(), target.blockX(), target.targetY(), target.blockZ(), target.material())) {
                stale++;
                continue;
            }

            if (!target.world().isChunkLoaded(target.blockX() >> 4, target.blockZ() >> 4)) {
                clearPendingArenaPreparation(target);
                unloaded++;
                continue;
            }

            int supportY = target.targetY() - 1;
            int standY = target.targetY() + 1;
            if (!needsArenaPrepare(player, target.world(), target.material(), target.blockX(), target.targetY(), target.blockZ(), standY)) {
                clearPendingArenaPreparation(target);
                skipped++;
                continue;
            }

            target.world().getBlockAt(target.blockX(), supportY, target.blockZ()).setType(target.material(), false);
            target.world().getBlockAt(target.blockX(), target.targetY(), target.blockZ()).setType(Material.AIR, false);
            target.world().getBlockAt(target.blockX(), standY, target.blockZ()).setType(Material.AIR, false);
            target.world().getBlockAt(target.blockX(), standY + 1, target.blockZ()).setType(Material.AIR, false);

            Location location = player.getLocation().clone();
            location.setX(target.blockX() + 0.5D);
            location.setY(standY);
            location.setZ(target.blockZ() + 0.5D);
            location.setYaw(0.0F);
            location.setPitch(90.0F);

            player.setGameMode(GameMode.CREATIVE);
            player.setAllowFlight(true);
            player.setFlying(true);
            player.setFallDistance(0.0F);
            player.getInventory().clear();
            player.getInventory().setHeldItemSlot(0);
            player.getInventory().setItemInMainHand(new ItemStack(target.material(), Math.max(1, target.material().getMaxStackSize())));

            if (player.teleport(location)) {
                teleported++;
            }
            arenaPreparations.put(player.getUniqueId(), new ArenaPreparation(target.world().getUID(), target.blockX(), target.targetY(), target.blockZ(), target.material()));
            clearPendingArenaPreparation(target);
            prepared++;
        }

        arenaPreparedPlayers.addAndGet(prepared);

        sender.sendMessage(
            "COMPAT_PROBE arena prefix=" + context.prefix()
                + " matched=" + context.matched()
                + " prepared=" + prepared
                + " teleported=" + teleported
                + " skipped=" + skipped
                + " unavailable=" + unavailable
                + " stale=" + stale
                + " unloaded=" + unloaded
                + " indexed=" + context.indexed()
                + " fallbackIndexed=" + context.fallbackIndexed()
                + " expectedPlayers=" + context.expectedPlayers()
                + " centerX=" + context.centerX()
                + " centerZ=" + context.centerZ()
                + " targetY=" + context.targetY()
                + " spacing=" + context.spacing()
                + " columns=" + context.columns()
                + " material=" + context.material().getKey().getKey()
        );
    }

    private void clearPendingArenaPreparation(ArenaTarget target) {
        ArenaPreparation pendingPreparation = pendingArenaPreparations.get(target.playerId());
        if (pendingPreparation != null && pendingPreparation.matches(target.world().getUID(), target.blockX(), target.targetY(), target.blockZ(), target.material())) {
            pendingArenaPreparations.remove(target.playerId());
        }
    }

    private boolean enqueueArenaChunkLoad(ArenaChunkLoadRequest request) {
        ArenaChunkLoadRequest pendingRequest = pendingArenaChunkLoads.get(request.chunk());
        if (pendingRequest != null) {
            pendingRequest.merge(request.sender(), request.context(), request.targets());
            arenaChunkLoadMerges.incrementAndGet();
            return false;
        }
        pendingArenaChunkLoads.put(request.chunk(), request);
        arenaChunkLoadQueue.addLast(request);
        arenaChunkLoadsQueued.incrementAndGet();
        return true;
    }

    private void pumpArenaChunkLoads() {
        while (arenaChunkLoadsInFlight < ARENA_CHUNK_LOAD_DEFAULT_MAX_IN_FLIGHT && !arenaChunkLoadQueue.isEmpty()) {
            ArenaChunkLoadRequest request = arenaChunkLoadQueue.removeFirst();
            ArenaChunk chunk = request.chunk();
            if (chunk.world().isChunkLoaded(chunk.chunkX(), chunk.chunkZ())) {
                pendingArenaChunkLoads.remove(chunk, request);
                arenaChunkLoadsCompleted.incrementAndGet();
                completeArena(request.sender(), request.context(), request.targets(), 0, 0);
                continue;
            }

            arenaChunkLoadsInFlight++;
            arenaChunkLoadsStarted.incrementAndGet();
            try {
                chunk.world().getChunkAtAsync(chunk.chunkX(), chunk.chunkZ(), true).whenComplete((ignored, throwable) ->
                    Bukkit.getScheduler().runTask(this, () -> {
                        arenaChunkLoadsInFlight--;
                        pendingArenaChunkLoads.remove(chunk, request);
                        if (throwable != null) {
                            failArenaChunkLoad(request, throwable);
                        } else {
                            arenaChunkLoadsCompleted.incrementAndGet();
                            completeArena(request.sender(), request.context(), request.targets(), 0, 0);
                        }
                        pumpArenaChunkLoads();
                    })
                );
            } catch (RuntimeException exception) {
                arenaChunkLoadsInFlight--;
                pendingArenaChunkLoads.remove(chunk, request);
                failArenaChunkLoad(request, exception);
            }
        }
    }

    private void failArenaChunkLoad(ArenaChunkLoadRequest request, Throwable throwable) {
        pendingArenaChunkLoads.remove(request.chunk(), request);
        arenaChunkLoadsFailed.incrementAndGet();
        for (ArenaTarget target : request.targets()) {
            clearPendingArenaPreparation(target);
        }
        request.sender().sendMessage(
            "COMPAT_PROBE arena error=async_chunk_load_failed"
                + " matched=" + request.context().matched()
                + " scheduled=" + request.targets().size()
                + " chunks=1"
                + " chunkX=" + request.chunk().chunkX()
                + " chunkZ=" + request.chunk().chunkZ()
                + " queueDepth=" + arenaChunkLoadQueue.size()
                + " inFlight=" + arenaChunkLoadsInFlight
                + " reason=" + throwable.getClass().getSimpleName()
        );
    }

    private void pumpArenaPreloads() {
        while (arenaPreloadLoadsInFlight < arenaPreloadMaxInFlight && !arenaPreloadQueue.isEmpty()) {
            ArenaPreloadRequest request = arenaPreloadQueue.removeFirst();
            ArenaChunk chunk = request.chunk();
            if (chunk.world().isChunkLoaded(chunk.chunkX(), chunk.chunkZ())) {
                arenaPreloadChunkLoadsCompleted.incrementAndGet();
                addArenaPreloadTicket(chunk);
                request.batch().recordLoaded();
                continue;
            }

            arenaPreloadLoadsInFlight++;
            arenaPreloadChunkLoadsStarted.incrementAndGet();
            try {
                chunk.world().getChunkAtAsync(chunk.chunkX(), chunk.chunkZ(), true).whenComplete((ignored, throwable) ->
                    Bukkit.getScheduler().runTask(this, () -> {
                        arenaPreloadLoadsInFlight--;
                        if (throwable != null) {
                            arenaPreloadChunkLoadsFailed.incrementAndGet();
                            request.batch().recordFailed();
                        } else {
                            arenaPreloadChunkLoadsCompleted.incrementAndGet();
                            addArenaPreloadTicket(chunk);
                            request.batch().recordLoaded();
                        }
                        pumpArenaPreloads();
                    })
                );
            } catch (RuntimeException exception) {
                arenaPreloadLoadsInFlight--;
                arenaPreloadChunkLoadsFailed.incrementAndGet();
                request.batch().recordFailed();
            }
        }
    }

    private void addArenaPreloadTicket(ArenaChunk chunk) {
        try {
            chunk.world().addPluginChunkTicket(chunk.chunkX(), chunk.chunkZ(), this);
            arenaPreloadTickets.add(chunk);
        } catch (RuntimeException ignored) {
        }
    }

    private Set<ArenaChunk> computeArenaChunks(World world, int expectedPlayers, int centerX, int centerZ, int spacing, int columns, int radiusChunks) {
        int rows = Math.max(1, (int) Math.ceil((double) Math.max(1, expectedPlayers) / columns));
        int startX = centerX - ((columns - 1) * spacing) / 2;
        int startZ = centerZ - ((rows - 1) * spacing) / 2;
        Set<ArenaChunk> chunks = new LinkedHashSet<>();
        for (int arenaIndex = 0; arenaIndex < expectedPlayers; arenaIndex++) {
            int column = arenaIndex % columns;
            int row = arenaIndex / columns;
            int blockX = startX + column * spacing;
            int blockZ = startZ + row * spacing;
            int chunkX = blockX >> 4;
            int chunkZ = blockZ >> 4;
            for (int dx = -radiusChunks; dx <= radiusChunks; dx++) {
                for (int dz = -radiusChunks; dz <= radiusChunks; dz++) {
                    chunks.add(new ArenaChunk(world, chunkX + dx, chunkZ + dz));
                }
            }
        }
        return chunks;
    }

    private record ArenaCommandContext(
        String prefix,
        int matched,
        int indexed,
        int fallbackIndexed,
        int expectedPlayers,
        int centerX,
        int centerZ,
        int targetY,
        int spacing,
        int columns,
        Material material
    ) {
    }

    private record ArenaTarget(UUID playerId, World world, int blockX, int targetY, int blockZ, Material material) {
    }

    private record ArenaChunk(World world, int chunkX, int chunkZ) {
    }

    private static final class ArenaChunkLoadRequest {
        private CommandSender sender;
        private ArenaCommandContext context;
        private final ArenaChunk chunk;
        private final List<ArenaTarget> targets;

        private ArenaChunkLoadRequest(CommandSender sender, ArenaCommandContext context, ArenaChunk chunk, List<ArenaTarget> targets) {
            this.sender = sender;
            this.context = context;
            this.chunk = chunk;
            this.targets = new ArrayList<>(targets);
        }

        private CommandSender sender() {
            return sender;
        }

        private ArenaCommandContext context() {
            return context;
        }

        private ArenaChunk chunk() {
            return chunk;
        }

        private List<ArenaTarget> targets() {
            return targets;
        }

        private void merge(CommandSender sender, ArenaCommandContext context, List<ArenaTarget> additions) {
            this.sender = sender;
            this.context = context;
            for (ArenaTarget addition : additions) {
                targets.removeIf(target -> target.playerId().equals(addition.playerId()));
                targets.add(addition);
            }
        }
    }

    private record ArenaPreloadRequest(ArenaPreloadBatch batch, ArenaChunk chunk) {
    }

    private final class ArenaPreloadBatch {
        private final int id;
        private final CommandSender sender;
        private final int expectedPlayers;
        private final int chunks;
        private final int centerX;
        private final int centerZ;
        private final int targetY;
        private final int spacing;
        private final int columns;
        private final Material material;
        private final int radiusChunks;
        private final int maxInFlight;
        private int loaded;
        private int failed;
        private boolean complete;

        private ArenaPreloadBatch(
            int id,
            CommandSender sender,
            int expectedPlayers,
            int chunks,
            int centerX,
            int centerZ,
            int targetY,
            int spacing,
            int columns,
            Material material,
            int radiusChunks,
            int maxInFlight
        ) {
            this.id = id;
            this.sender = sender;
            this.expectedPlayers = expectedPlayers;
            this.chunks = chunks;
            this.centerX = centerX;
            this.centerZ = centerZ;
            this.targetY = targetY;
            this.spacing = spacing;
            this.columns = columns;
            this.material = material;
            this.radiusChunks = radiusChunks;
            this.maxInFlight = maxInFlight;
        }

        private void recordLoaded() {
            loaded++;
            sendCompleteIfReady();
        }

        private void recordFailed() {
            failed++;
            sendCompleteIfReady();
        }

        private void sendCompleteIfReady() {
            if (loaded + failed >= chunks) {
                sendComplete();
            }
        }

        private void sendComplete() {
            if (complete) {
                return;
            }
            complete = true;
            sender.sendMessage(
                "COMPAT_PROBE arena_preload complete"
                    + " id=" + id
                    + " expectedPlayers=" + expectedPlayers
                    + " radiusChunks=" + radiusChunks
                    + " maxInFlight=" + maxInFlight
                    + " chunks=" + chunks
                    + " loaded=" + loaded
                    + " failed=" + failed
                    + " tickets=" + arenaPreloadTickets.size()
                    + " centerX=" + centerX
                    + " centerZ=" + centerZ
                    + " targetY=" + targetY
                    + " spacing=" + spacing
                    + " columns=" + columns
                    + " material=" + material.getKey().getKey()
            );
        }
    }

    private void completeMobStorm(
        CommandSender sender,
        World world,
        EntityType entityType,
        List<MobStormSpawn> spawns,
        int targetY,
        int requested,
        int centerX,
        int centerZ,
        int spacing,
        int columns
    ) {
        int spawned = 0;
        for (MobStormSpawn spawn : spawns) {
            int spawnY = targetY == Integer.MIN_VALUE ? world.getHighestBlockYAt(spawn.blockX(), spawn.blockZ()) + 1 : targetY;
            Location location = new Location(world, spawn.blockX() + 0.5D, spawnY, spawn.blockZ() + 0.5D);
            world.spawnEntity(location, entityType);
            spawned++;
        }
        mobStormCommands.incrementAndGet();
        mobStormSpawned.addAndGet(spawned);
        sender.sendMessage(
            "COMPAT_PROBE mobstorm entity=" + entityType.getKey().getKey()
                + " requested=" + requested
                + " spawned=" + spawned
                + " centerX=" + centerX
                + " centerZ=" + centerZ
                + " targetY=" + (targetY == Integer.MIN_VALUE ? "surface" : Integer.toString(targetY))
                + " spacing=" + spacing
                + " columns=" + columns
        );
    }

    private static long chunkKey(int chunkX, int chunkZ) {
        return ((long) chunkX << 32) ^ (chunkZ & 0xffffffffL);
    }

    private static int unpackChunkX(long key) {
        return (int) (key >> 32);
    }

    private static int unpackChunkZ(long key) {
        return (int) key;
    }

    private boolean needsArenaPrepare(Player player, World world, Material material, int blockX, int targetY, int blockZ, int standY) {
        ArenaPreparation preparation = arenaPreparations.get(player.getUniqueId());
        if (preparation == null || !preparation.matches(world.getUID(), blockX, targetY, blockZ, material)) {
            return true;
        }
        Location location = player.getLocation();
        if (Math.abs(location.getX() - (blockX + 0.5D)) > 0.25D
            || Math.abs(location.getY() - standY) > 0.5D
            || Math.abs(location.getZ() - (blockZ + 0.5D)) > 0.25D) {
            return true;
        }
        if (player.getGameMode() != GameMode.CREATIVE || !player.getAllowFlight() || !player.isFlying()) {
            return true;
        }
        if (player.getInventory().getHeldItemSlot() != 0) {
            return true;
        }
        ItemStack mainHand = player.getInventory().getItemInMainHand();
        return mainHand.getType() != material || mainHand.getAmount() <= 0;
    }

    private record MobStormSpawn(int blockX, int blockZ) {
    }

    private void logEvent(String eventName, String detail) {
        int sequence = eventSequence.incrementAndGet();
        getLogger().info("COMPAT_PROBE event=" + eventName + " sequence=" + sequence + " detail=" + detail);
    }

    private void logBlockEvent(String type, int places, int breaks, boolean cancelled, String playerName, int typeCount) {
        if (!shouldLogBlockEvent(typeCount)) {
            return;
        }
        getLogger().info(
            "COMPAT_PROBE block_event type=" + type
                + " places=" + places
                + " breaks=" + breaks
                + " cancelled=" + cancelled
                + " player=" + playerName
        );
    }

    private static boolean shouldLogBlockEvent(int count) {
        return count <= 8 || (count & (count - 1)) == 0 || count % 1000 == 0;
    }

    private void logMetrics() {
        double[] tps = Bukkit.getTPS();
        long usedMemory = Runtime.getRuntime().totalMemory() - Runtime.getRuntime().freeMemory();
        int loadedChunks = 0;
        int livingEntities = 0;
        for (World world : Bukkit.getWorlds()) {
            loadedChunks += world.getLoadedChunks().length;
            livingEntities += world.getLivingEntities().size();
        }
        SendPressureSnapshot sendPressure = collectSendPressureSnapshot();
        getLogger().info(String.format(
            Locale.ROOT,
            "COMPAT_PROBE metrics online=%d loadedChunks=%d tps1=%.2f tps5=%.2f tps15=%.2f avgTickMs=%.2f usedMemMiB=%d blockPlaces=%d blockBreaks=%d arenaCommands=%d arenaPrepared=%d arenaSkipped=%d arenaChunkLoadsQueued=%d arenaChunkLoadsStarted=%d arenaChunkLoadsCompleted=%d arenaChunkLoadsFailed=%d arenaChunkLoadsInFlight=%d arenaChunkLoadMerges=%d arenaPreloadCommands=%d arenaPreloadChunkLoadsQueued=%d arenaPreloadChunkLoadsStarted=%d arenaPreloadChunkLoadsCompleted=%d arenaPreloadChunkLoadsFailed=%d arenaPreloadChunkLoadsInFlight=%d arenaPreloadTickets=%d mobStormCommands=%d mobStormSpawned=%d livingEntities=%d compatProbeCommands=%d playerCommands=%d itemHeldEvents=%d animationEvents=%d interactEvents=%d entityDamageEvents=%d toggleSprintEvents=%d toggleSneakEvents=%d sendPressurePlayers=%d sendPressureConnections=%d sendPressureChunkSenders=%d connectionPendingActionsMax=%d connectionPendingOutboundBytesMax=%d connectionPendingOutboundBytesReadCount=%d connectionPendingOutboundBytesUnavailableCount=%d connectionBytesBeforeWritableMax=%d connectionBytesBeforeWritableReadCount=%d connectionBytesBeforeWritableUnavailableCount=%d connectionBytesBeforeUnwritableMin=%d connectionBytesBeforeUnwritableReadCount=%d connectionBytesBeforeUnwritableUnavailableCount=%d connectionNonWritable=%d chunkSenderPendingChunksMax=%d chunkSenderPendingChunksReadCount=%d chunkSenderPendingChunksUnavailableCount=%d chunkSenderUnacknowledgedBatchesMax=%d chunkSenderBatchQuotaMax=%.2f chunkSenderDesiredChunksPerTickMax=%.2f chunkSenderMaxUnacknowledgedBatchesMax=%d chunkSenderChannelNotWritablePendingChunksPeak=%d chunkSenderChannelNotWritablePendingChunksPeakReadCount=%d chunkSenderChannelNotWritablePendingChunksPeakUnavailableCount=%d chunkSenderChannelNotWritableSkipsMax=%d chunkSenderChannelNotWritableSkipsReadCount=%d chunkSenderChannelNotWritableSkipsUnavailableCount=%d chunkSenderNearUnwritablePendingChunksPeak=%d chunkSenderNearUnwritablePendingChunksPeakReadCount=%d chunkSenderNearUnwritablePendingChunksPeakUnavailableCount=%d chunkSenderNearUnwritableSkipsMax=%d chunkSenderNearUnwritableSkipsReadCount=%d chunkSenderNearUnwritableSkipsUnavailableCount=%d",
            Bukkit.getOnlinePlayers().size(),
            loadedChunks,
            tps.length > 0 ? tps[0] : 0.0D,
            tps.length > 1 ? tps[1] : 0.0D,
            tps.length > 2 ? tps[2] : 0.0D,
            Bukkit.getAverageTickTime(),
            usedMemory / 1024L / 1024L,
            blockPlaceEvents.get(),
            blockBreakEvents.get(),
            arenaCommands.get(),
            arenaPreparedPlayers.get(),
            arenaSkippedPlayers.get(),
            arenaChunkLoadsQueued.get(),
            arenaChunkLoadsStarted.get(),
            arenaChunkLoadsCompleted.get(),
            arenaChunkLoadsFailed.get(),
            arenaChunkLoadsInFlight,
            arenaChunkLoadMerges.get(),
            arenaPreloadCommands.get(),
            arenaPreloadChunkLoadsQueued.get(),
            arenaPreloadChunkLoadsStarted.get(),
            arenaPreloadChunkLoadsCompleted.get(),
            arenaPreloadChunkLoadsFailed.get(),
            arenaPreloadLoadsInFlight,
            arenaPreloadTickets.size(),
            mobStormCommands.get(),
            mobStormSpawned.get(),
            livingEntities,
            compatProbeCommands.get(),
            commandPreprocessEvents.get(),
            itemHeldEvents.get(),
            animationEvents.get(),
            interactEvents.get(),
            entityDamageEvents.get(),
            toggleSprintEvents.get(),
            toggleSneakEvents.get(),
            sendPressure.players,
            sendPressure.connections,
            sendPressure.chunkSenders,
            sendPressure.connectionPendingActionsMax,
            sendPressure.connectionPendingOutboundBytesMax,
            sendPressure.connectionPendingOutboundBytesReadCount,
            sendPressure.connectionPendingOutboundBytesUnavailableCount,
            sendPressure.connectionBytesBeforeWritableMax,
            sendPressure.connectionBytesBeforeWritableReadCount,
            sendPressure.connectionBytesBeforeWritableUnavailableCount,
            sendPressure.connectionBytesBeforeUnwritableMin,
            sendPressure.connectionBytesBeforeUnwritableReadCount,
            sendPressure.connectionBytesBeforeUnwritableUnavailableCount,
            sendPressure.connectionNonWritable,
            sendPressure.chunkSenderPendingChunksMax,
            sendPressure.chunkSenderPendingChunksReadCount,
            sendPressure.chunkSenderPendingChunksUnavailableCount,
            sendPressure.chunkSenderUnacknowledgedBatchesMax,
            sendPressure.chunkSenderBatchQuotaMax,
            sendPressure.chunkSenderDesiredChunksPerTickMax,
            sendPressure.chunkSenderMaxUnacknowledgedBatchesMax,
            sendPressure.chunkSenderChannelNotWritablePendingChunksPeak,
            sendPressure.chunkSenderChannelNotWritablePendingChunksPeakReadCount,
            sendPressure.chunkSenderChannelNotWritablePendingChunksPeakUnavailableCount,
            sendPressure.chunkSenderChannelNotWritableSkipsMax,
            sendPressure.chunkSenderChannelNotWritableSkipsReadCount,
            sendPressure.chunkSenderChannelNotWritableSkipsUnavailableCount,
            sendPressure.chunkSenderNearUnwritablePendingChunksPeak,
            sendPressure.chunkSenderNearUnwritablePendingChunksPeakReadCount,
            sendPressure.chunkSenderNearUnwritablePendingChunksPeakUnavailableCount,
            sendPressure.chunkSenderNearUnwritableSkipsMax,
            sendPressure.chunkSenderNearUnwritableSkipsReadCount,
            sendPressure.chunkSenderNearUnwritableSkipsUnavailableCount
        ));
    }

    private SendPressureSnapshot collectSendPressureSnapshot() {
        SendPressureSnapshot snapshot = new SendPressureSnapshot();
        for (Player player : Bukkit.getOnlinePlayers()) {
            snapshot.players++;
            Object handle = invokeNoArg(player, "getHandle");
            if (handle == null) {
                continue;
            }

            Object packetListener = readFieldRecursive(handle, "connection");
            if (packetListener == null) {
                continue;
            }

            Object networkConnection = readFieldRecursive(packetListener, "connection");
            if (networkConnection != null) {
                snapshot.connections++;
                snapshot.captureConnection(networkConnection);
            }

            Object chunkSender = readFieldRecursive(packetListener, "chunkSender");
            if (chunkSender != null) {
                snapshot.chunkSenders++;
                snapshot.captureChunkSender(chunkSender);
            }
        }
        return snapshot;
    }

    private static Object invokeNoArg(Object target, String methodName) {
        if (target == null) {
            return null;
        }

        for (Class<?> type = target.getClass(); type != null; type = type.getSuperclass()) {
            try {
                Method method = type.getDeclaredMethod(methodName);
                method.setAccessible(true);
                return method.invoke(target);
            } catch (NoSuchMethodException ignored) {
            } catch (ReflectiveOperationException | RuntimeException ignored) {
                return null;
            }
        }
        return null;
    }

    private static Object readFieldRecursive(Object target, String fieldName) {
        if (target == null) {
            return null;
        }

        for (Class<?> type = target.getClass(); type != null; type = type.getSuperclass()) {
            try {
                Field field = type.getDeclaredField(fieldName);
                field.setAccessible(true);
                return field.get(target);
            } catch (NoSuchFieldException ignored) {
            } catch (IllegalAccessException | RuntimeException ignored) {
                return null;
            }
        }
        return null;
    }

    private static Integer invokeInt(Object target, String methodName) {
        Object value = invokeNoArg(target, methodName);
        return value instanceof Number number ? Integer.valueOf(number.intValue()) : null;
    }

    private static Long invokeLong(Object target, String methodName) {
        Object value = invokeNoArg(target, methodName);
        return value instanceof Number number ? Long.valueOf(number.longValue()) : null;
    }

    private static Float invokeFloat(Object target, String methodName) {
        Object value = invokeNoArg(target, methodName);
        return value instanceof Number number ? Float.valueOf(number.floatValue()) : null;
    }

    private static Boolean invokeBoolean(Object target, String methodName) {
        Object value = invokeNoArg(target, methodName);
        return value instanceof Boolean bool ? bool : null;
    }

    private static Long readChannelOutboundBytes(Object channel) {
        if (channel == null) {
            return null;
        }
        Object unsafe = invokeNoArg(channel, "unsafe");
        Object outboundBuffer = invokeNoArg(unsafe, "outboundBuffer");
        Object totalPendingWriteBytes = invokeNoArg(outboundBuffer, "totalPendingWriteBytes");
        return totalPendingWriteBytes instanceof Number number ? Long.valueOf(number.longValue()) : null;
    }

    private static final class SendPressureSnapshot {
        private int players;
        private int connections;
        private int chunkSenders;
        private int connectionPendingActionsMax;
        private long connectionPendingOutboundBytesMax = -1L;
        private int connectionPendingOutboundBytesReadCount;
        private int connectionPendingOutboundBytesUnavailableCount;
        private long connectionBytesBeforeWritableMax = -1L;
        private int connectionBytesBeforeWritableReadCount;
        private int connectionBytesBeforeWritableUnavailableCount;
        private long connectionBytesBeforeUnwritableMin = -1L;
        private int connectionBytesBeforeUnwritableReadCount;
        private int connectionBytesBeforeUnwritableUnavailableCount;
        private int connectionNonWritable;
        private int chunkSenderPendingChunksMax;
        private int chunkSenderPendingChunksReadCount;
        private int chunkSenderPendingChunksUnavailableCount;
        private int chunkSenderUnacknowledgedBatchesMax;
        private float chunkSenderBatchQuotaMax;
        private float chunkSenderDesiredChunksPerTickMax;
        private int chunkSenderMaxUnacknowledgedBatchesMax;
        private int chunkSenderChannelNotWritablePendingChunksPeak;
        private int chunkSenderChannelNotWritablePendingChunksPeakReadCount;
        private int chunkSenderChannelNotWritablePendingChunksPeakUnavailableCount;
        private long chunkSenderChannelNotWritableSkipsMax;
        private int chunkSenderChannelNotWritableSkipsReadCount;
        private int chunkSenderChannelNotWritableSkipsUnavailableCount;
        private int chunkSenderNearUnwritablePendingChunksPeak;
        private int chunkSenderNearUnwritablePendingChunksPeakReadCount;
        private int chunkSenderNearUnwritablePendingChunksPeakUnavailableCount;
        private long chunkSenderNearUnwritableSkipsMax;
        private int chunkSenderNearUnwritableSkipsReadCount;
        private int chunkSenderNearUnwritableSkipsUnavailableCount;

        private void captureConnection(Object packetListener) {
            Integer pendingActions = invokeInt(packetListener, "getPendingActionsCount");
            if (pendingActions == null) {
                Object pendingActionsField = readFieldRecursive(packetListener, "pendingActions");
                pendingActions = invokeInt(pendingActionsField, "size");
            }
            if (pendingActions != null) {
                connectionPendingActionsMax = Math.max(connectionPendingActionsMax, pendingActions.intValue());
            }

            Boolean writable = invokeBoolean(packetListener, "isChannelWritable");
            if (writable == null) {
                Object channel = readFieldRecursive(packetListener, "channel");
                writable = channel == null ? null : invokeBoolean(channel, "isWritable");
            }
            if (writable != null && !writable.booleanValue()) {
                connectionNonWritable++;
            }

            Long pendingOutboundBytes = invokeLong(packetListener, "getPendingOutboundBytes");
            if (pendingOutboundBytes == null || pendingOutboundBytes.longValue() < 0L) {
                Object channel = readFieldRecursive(packetListener, "channel");
                pendingOutboundBytes = readChannelOutboundBytes(channel);
            }
            if (pendingOutboundBytes != null && pendingOutboundBytes.longValue() >= 0L) {
                connectionPendingOutboundBytesReadCount++;
                connectionPendingOutboundBytesMax = Math.max(connectionPendingOutboundBytesMax, pendingOutboundBytes.longValue());
            } else {
                connectionPendingOutboundBytesUnavailableCount++;
            }

            Long bytesBeforeWritable = invokeLong(packetListener, "getBytesBeforeWritable");
            if (bytesBeforeWritable == null || bytesBeforeWritable.longValue() < 0L) {
                Object channel = readFieldRecursive(packetListener, "channel");
                bytesBeforeWritable = channel == null ? null : invokeLong(channel, "bytesBeforeWritable");
            }
            if (bytesBeforeWritable != null && bytesBeforeWritable.longValue() >= 0L) {
                connectionBytesBeforeWritableReadCount++;
                connectionBytesBeforeWritableMax = Math.max(connectionBytesBeforeWritableMax, bytesBeforeWritable.longValue());
            } else {
                connectionBytesBeforeWritableUnavailableCount++;
            }

            Long bytesBeforeUnwritable = invokeLong(packetListener, "getBytesBeforeUnwritable");
            if (bytesBeforeUnwritable == null || bytesBeforeUnwritable.longValue() < 0L) {
                Object channel = readFieldRecursive(packetListener, "channel");
                bytesBeforeUnwritable = channel == null ? null : invokeLong(channel, "bytesBeforeUnwritable");
            }
            if (bytesBeforeUnwritable != null && bytesBeforeUnwritable.longValue() >= 0L) {
                connectionBytesBeforeUnwritableReadCount++;
                if (connectionBytesBeforeUnwritableMin < 0L || bytesBeforeUnwritable.longValue() < connectionBytesBeforeUnwritableMin) {
                    connectionBytesBeforeUnwritableMin = bytesBeforeUnwritable.longValue();
                }
            } else {
                connectionBytesBeforeUnwritableUnavailableCount++;
            }
        }

        private void captureChunkSender(Object chunkSender) {
            Integer pendingChunks = invokeInt(chunkSender, "getPendingChunkCount");
            if (pendingChunks == null) {
                Object pendingChunksField = readFieldRecursive(chunkSender, "pendingChunks");
                pendingChunks = invokeInt(pendingChunksField, "size");
            }
            if (pendingChunks != null) {
                chunkSenderPendingChunksReadCount++;
                chunkSenderPendingChunksMax = Math.max(chunkSenderPendingChunksMax, pendingChunks.intValue());
            } else {
                chunkSenderPendingChunksUnavailableCount++;
            }

            Integer unacknowledgedBatches = invokeInt(chunkSender, "getUnacknowledgedBatchCount");
            if (unacknowledgedBatches == null) {
                Object field = readFieldRecursive(chunkSender, "unacknowledgedBatches");
                unacknowledgedBatches = field instanceof Number number ? Integer.valueOf(number.intValue()) : null;
            }
            if (unacknowledgedBatches != null) {
                chunkSenderUnacknowledgedBatchesMax = Math.max(chunkSenderUnacknowledgedBatchesMax, unacknowledgedBatches.intValue());
            }

            Float batchQuota = invokeFloat(chunkSender, "getCurrentBatchQuota");
            if (batchQuota == null) {
                Object field = readFieldRecursive(chunkSender, "batchQuota");
                batchQuota = field instanceof Number number ? Float.valueOf(number.floatValue()) : null;
            }
            if (batchQuota != null) {
                chunkSenderBatchQuotaMax = Math.max(chunkSenderBatchQuotaMax, batchQuota.floatValue());
            }

            Float desiredChunksPerTick = invokeFloat(chunkSender, "getDesiredChunksPerTick");
            if (desiredChunksPerTick == null) {
                Object field = readFieldRecursive(chunkSender, "desiredChunksPerTick");
                desiredChunksPerTick = field instanceof Number number ? Float.valueOf(number.floatValue()) : null;
            }
            if (desiredChunksPerTick != null) {
                chunkSenderDesiredChunksPerTickMax = Math.max(chunkSenderDesiredChunksPerTickMax, desiredChunksPerTick.floatValue());
            }

            Integer maxUnacknowledgedBatches = invokeInt(chunkSender, "getMaxUnacknowledgedBatches");
            if (maxUnacknowledgedBatches == null) {
                Object field = readFieldRecursive(chunkSender, "maxUnacknowledgedBatches");
                maxUnacknowledgedBatches = field instanceof Number number ? Integer.valueOf(number.intValue()) : null;
            }
            if (maxUnacknowledgedBatches != null) {
                chunkSenderMaxUnacknowledgedBatchesMax = Math.max(chunkSenderMaxUnacknowledgedBatchesMax, maxUnacknowledgedBatches.intValue());
            }

            Integer channelNotWritablePendingChunksPeak = invokeInt(chunkSender, "getChannelNotWritablePendingChunksPeak");
            if (channelNotWritablePendingChunksPeak == null) {
                Object field = readFieldRecursive(chunkSender, "channelNotWritablePendingChunksPeak");
                channelNotWritablePendingChunksPeak = field instanceof Number number ? Integer.valueOf(number.intValue()) : null;
            }
            if (channelNotWritablePendingChunksPeak != null) {
                chunkSenderChannelNotWritablePendingChunksPeakReadCount++;
                chunkSenderChannelNotWritablePendingChunksPeak = Math.max(chunkSenderChannelNotWritablePendingChunksPeak, channelNotWritablePendingChunksPeak.intValue());
            } else {
                chunkSenderChannelNotWritablePendingChunksPeakUnavailableCount++;
            }

            Long channelNotWritableSkips = invokeLong(chunkSender, "getChannelNotWritableSkips");
            if (channelNotWritableSkips == null) {
                Object field = readFieldRecursive(chunkSender, "channelNotWritableSkips");
                channelNotWritableSkips = field instanceof Number number ? Long.valueOf(number.longValue()) : null;
            }
            if (channelNotWritableSkips != null) {
                chunkSenderChannelNotWritableSkipsReadCount++;
                chunkSenderChannelNotWritableSkipsMax = Math.max(chunkSenderChannelNotWritableSkipsMax, channelNotWritableSkips.longValue());
            } else {
                chunkSenderChannelNotWritableSkipsUnavailableCount++;
            }

            Integer channelNearUnwritablePendingChunksPeak = invokeInt(chunkSender, "getChannelNearUnwritablePendingChunksPeak");
            if (channelNearUnwritablePendingChunksPeak == null) {
                Object field = readFieldRecursive(chunkSender, "channelNearUnwritablePendingChunksPeak");
                channelNearUnwritablePendingChunksPeak = field instanceof Number number ? Integer.valueOf(number.intValue()) : null;
            }
            if (channelNearUnwritablePendingChunksPeak != null) {
                chunkSenderNearUnwritablePendingChunksPeakReadCount++;
                chunkSenderNearUnwritablePendingChunksPeak = Math.max(chunkSenderNearUnwritablePendingChunksPeak, channelNearUnwritablePendingChunksPeak.intValue());
            } else {
                chunkSenderNearUnwritablePendingChunksPeakUnavailableCount++;
            }

            Long channelNearUnwritableSkips = invokeLong(chunkSender, "getChannelNearUnwritableSkips");
            if (channelNearUnwritableSkips == null) {
                Object field = readFieldRecursive(chunkSender, "channelNearUnwritableSkips");
                channelNearUnwritableSkips = field instanceof Number number ? Long.valueOf(number.longValue()) : null;
            }
            if (channelNearUnwritableSkips != null) {
                chunkSenderNearUnwritableSkipsReadCount++;
                chunkSenderNearUnwritableSkipsMax = Math.max(chunkSenderNearUnwritableSkipsMax, channelNearUnwritableSkips.longValue());
            } else {
                chunkSenderNearUnwritableSkipsUnavailableCount++;
            }
        }
    }

    private int parseIntArg(String[] args, int index, int fallback) {
        if (index >= args.length) {
            return fallback;
        }
        try {
            return Integer.parseInt(args[index]);
        } catch (NumberFormatException ignored) {
            return fallback;
        }
    }

    private Integer parseArenaIndex(String playerName, String prefix, int expectedPlayers) {
        if (prefix.isEmpty() || !playerName.startsWith(prefix)) {
            return null;
        }
        String suffix = playerName.substring(prefix.length());
        if (suffix.isEmpty()) {
            return null;
        }
        for (int i = 0; i < suffix.length(); i++) {
            if (!Character.isDigit(suffix.charAt(i))) {
                return null;
            }
        }
        try {
            int parsed = Integer.parseInt(suffix);
            if (parsed < 0 || parsed >= Math.max(1, expectedPlayers)) {
                return null;
            }
            return parsed;
        } catch (NumberFormatException ignored) {
            return null;
        }
    }

    private static final class ArenaPreparation {
        private final UUID worldId;
        private final int blockX;
        private final int targetY;
        private final int blockZ;
        private final Material material;

        private ArenaPreparation(UUID worldId, int blockX, int targetY, int blockZ, Material material) {
            this.worldId = worldId;
            this.blockX = blockX;
            this.targetY = targetY;
            this.blockZ = blockZ;
            this.material = material;
        }

        private boolean matches(UUID worldId, int blockX, int targetY, int blockZ, Material material) {
            return this.worldId.equals(worldId)
                && this.blockX == blockX
                && this.targetY == targetY
                && this.blockZ == blockZ
                && this.material == material;
        }
    }
}
