package dev.crussty.cmenus;

import org.bukkit.Bukkit;
import org.bukkit.Color;
import org.bukkit.Location;
import org.bukkit.World;
import org.bukkit.entity.Display;
import org.bukkit.entity.Entity;
import org.bukkit.entity.ItemDisplay;
import org.bukkit.entity.Player;
import org.bukkit.entity.TextDisplay;
import org.bukkit.inventory.ItemStack;
import org.bukkit.inventory.meta.ItemMeta;
import org.bukkit.util.Transformation;

import org.joml.Quaternionf;
import org.joml.Vector3f;

/**
 * c-menus Java bridge.
 *
 * The CRUSSTY JNI wrapper (jvmti-bindings JniEnv) can only call methods that
 * RETURN void/int/long/boolean/object. Every c-menus operation that needs
 * float/double returns or multi-object construction is flattened into this
 * class: Rust calls the static methods below through call_static_*_method and
 * passes/primitives freely (arguments may be any primitive — only the return
 * type is restricted).
 *
 * Contract: NO method here may ever throw. Every body is wrapped in
 * try/catch(Throwable) so a pending JNI exception can never leak into the
 * runtime's tick pipeline. Failures return null / -1 / false.
 *
 * Compiled with `javac --release 21` against the minimal API stubs in
 * bridge/stubs (see scripts/build_bridge.sh) — no external jars needed. The
 * .class artifact is committed under bridge-build/ and embedded into the .so
 * at compile time (include_bytes!), then defined in the JVM via DefineClass.
 * Keep the Java syntax simple (Java 8 level).
 */
public final class Bridge {

    private Bridge() {}

    // ------------------------------------------------------------------
    // Entity lifecycle
    // ------------------------------------------------------------------

    /** Spawn a TextDisplay; returns the entity or null. */
    public static Object spawnTextDisplay(Object world, double x, double y, double z) {
        try {
            World w = (World) world;
            Location loc = new Location(w, x, y, z);
            return w.spawn(loc, TextDisplay.class);
        } catch (Throwable t) {
            return null;
        }
    }

    /**
     * Spawn an ItemDisplay carrying a paper item with the given custom model
     * data (the resource pack maps CMD values onto c-menus panel/item models
     * via a range_dispatch item definition). Returns the entity or null.
     */
    public static Object spawnItemDisplay(Object world, double x, double y, double z, int cmd) {
        try {
            World w = (World) world;
            Location loc = new Location(w, x, y, z);
            ItemDisplay e = w.spawn(loc, ItemDisplay.class);
            ItemStack stack = new ItemStack(org.bukkit.Material.PAPER);
            ItemMeta meta = stack.getItemMeta();
            meta.setCustomModelData(Integer.valueOf(cmd));
            stack.setItemMeta(meta);
            e.setItemStack(stack);
            e.setItemDisplayTransform(ItemDisplay.ItemDisplayTransform.FIXED);
            return e;
        } catch (Throwable t) {
            return null;
        }
    }

    /** Spawn an ItemDisplay carrying a vanilla item (material name, e.g. "minecraft:ring" custom or "IRON_INGOT"). */
    public static Object spawnItemVanilla(Object world, double x, double y, double z, String material) {
        try {
            World w = (World) world;
            Location loc = new Location(w, x, y, z);
            ItemDisplay e = w.spawn(loc, ItemDisplay.class);
            org.bukkit.Material m = org.bukkit.Material.matchMaterial(material);
            if (m == null) { e.remove(); return null; }
            e.setItemStack(new ItemStack(m));
            e.setItemDisplayTransform(ItemDisplay.ItemDisplayTransform.FIXED);
            return e;
        } catch (Throwable t) {
            return null;
        }
    }

    public static void removeEntity(Object e) {
        try {
            ((Entity) e).remove();
        } catch (Throwable t) {
            // ignore
        }
    }

    public static int entityValid(Object e) {
        try {
            return ((Entity) e).isValid() ? 1 : 0;
        } catch (Throwable t) {
            return 0;
        }
    }

    public static int entityId(Object e) {
        try {
            return ((Entity) e).getEntityId();
        } catch (Throwable t) {
            return -1;
        }
    }

    // ------------------------------------------------------------------
    // Display rendering
    // ------------------------------------------------------------------

    /**
     * Apply a full Display transformation with client-side interpolation.
     * yaw/pitch/roll are degrees (left rotation). interpDelay is ticks
     * before the interpolation starts (-1 = immediate); interpDur is the
     * client-side interpolation length in ticks.
     */
    public static void applyTransform(Object display,
                                      double tx, double ty, double tz,
                                      double sx, double sy, double sz,
                                      float yawDeg, float pitchDeg, float rollDeg,
                                      int interpDelay, int interpDur) {
        try {
            Display d = (Display) display;
            Transformation t = new Transformation(
                    new Vector3f((float) tx, (float) ty, (float) tz),
                    new Quaternionf().rotationYXZ(
                            (float) Math.toRadians(yawDeg),
                            (float) Math.toRadians(pitchDeg),
                            (float) Math.toRadians(rollDeg)),
                    new Vector3f((float) sx, (float) sy, (float) sz),
                    new Quaternionf());
            d.setInterpolationDelay(interpDelay);
            d.setInterpolationDuration(interpDur);
            d.setTransformation(t);
        } catch (Throwable t2) {
            // ignore
        }
    }

    /** Teleport a display with client-side position interpolation (ticks). */
    public static void teleport(Object display, double x, double y, double z, int dur) {
        try {
            Display d = (Display) display;
            d.setTeleportDuration(dur);
            d.teleport(new Location(d.getWorld(), x, y, z));
        } catch (Throwable t) {
            // ignore
        }
    }

    /** Static display properties: view range, shadow, brightness, persistence. */
    public static void displayProps(Object display, float viewRange, float shadowR,
                                    float shadowS, int brightSky, int brightBlock,
                                    boolean persistent) {
        try {
            Display d = (Display) display;
            if (viewRange > 0.0f) {
                d.setViewRange(viewRange);
            }
            d.setShadowRadius(shadowR);
            d.setShadowStrength(shadowS);
            if (brightSky >= 0 && brightBlock >= 0) {
                d.setBrightness(new Display.Brightness(brightSky, brightBlock));
            } else {
                d.setBrightness(null);
            }
            d.setPersistent(persistent);
        } catch (Throwable t) {
            // ignore
        }
    }

    /** Facing mode: 0 fixed, 1 center billboard, 2 vertical, 3 horizontal. */
    public static void billboard(Object display, int mode) {
        try {
            Display d = (Display) display;
            switch (mode) {
                case 1: d.setBillboard(Display.Billboard.CENTER); break;
                case 2: d.setBillboard(Display.Billboard.VERTICAL); break;
                case 3: d.setBillboard(Display.Billboard.HORIZONTAL); break;
                default: d.setBillboard(Display.Billboard.FIXED); break;
            }
        } catch (Throwable t) {
            // ignore
        }
    }

    /** Style a TextDisplay. bgArgb full alpha 0x00 -> null background. align: 0 left, 1 center, 2 right. */
    public static void applyText(Object display, String legacyText, int bgArgb,
                                 boolean seeThrough, boolean shadowed, int lineWidth,
                                 int align, byte opacity) {
        try {
            TextDisplay d = (TextDisplay) display;
            d.setText(legacyText);
            if (bgArgb == 0) {
                d.setBackgroundColor(null);
            } else {
                d.setBackgroundColor(Color.fromARGB(bgArgb));
            }
            d.setDefaultBackground(false);
            d.setSeeThrough(seeThrough);
            d.setShadowed(shadowed);
            if (lineWidth > 0) {
                d.setLineWidth(lineWidth);
            }
            if (align == 1) {
                d.setAlignment(TextDisplay.TextAlignment.CENTER);
            } else if (align == 2) {
                d.setAlignment(TextDisplay.TextAlignment.RIGHT);
            } else {
                d.setAlignment(TextDisplay.TextAlignment.LEFT);
            }
            d.setTextOpacity(opacity);
        } catch (Throwable t) {
            // ignore
        }
    }

    /** Set the item carried by an ItemDisplay to paper + custom model data. */
    public static void itemCmd(Object display, int cmd) {
        try {
            ItemDisplay d = (ItemDisplay) display;
            ItemStack stack = new ItemStack(org.bukkit.Material.PAPER);
            ItemMeta meta = stack.getItemMeta();
            meta.setCustomModelData(Integer.valueOf(cmd));
            stack.setItemMeta(meta);
            d.setItemStack(stack);
        } catch (Throwable t) {
            // ignore
        }
    }

    // ------------------------------------------------------------------
    // Player probing (strings avoid float-returning JNI calls)
    // ------------------------------------------------------------------

    /** Eye transform as "x,y,z,yaw,pitch" (yaw/pitch degrees) or null. */
    public static String playerEye(Object player) {
        try {
            Location l = ((Player) player).getEyeLocation();
            return l.getX() + "," + l.getY() + "," + l.getZ() + ","
                    + l.getYaw() + "," + l.getPitch();
        } catch (Throwable t) {
            return null;
        }
    }

    /** Server-side player feet position as "x,y,z,yaw,pitch" or null. */
    public static String playerLoc(Object player) {
        try {
            Location l = ((Player) player).getLocation();
            return l.getX() + "," + l.getY() + "," + l.getZ() + ","
                    + l.getYaw() + "," + l.getPitch();
        } catch (Throwable t) {
            return null;
        }
    }

    /** All online players as "uuid|name|world" separated by ';'. */
    public static String playerSnapshot() {
        try {
            StringBuilder sb = new StringBuilder();
            for (Player p : Bukkit.getOnlinePlayers()) {
                if (sb.length() > 0) {
                    sb.append(';');
                }
                sb.append(p.getUniqueId().toString()).append('|')
                        .append(p.getName()).append('|')
                        .append(p.getWorld().getName());
            }
            return sb.toString();
        } catch (Throwable t) {
            return "";
        }
    }

    /** Resolve an online player by uuid string; null when offline. */
    public static Object playerByUuid(String uuid) {
        try {
            return Bukkit.getPlayer(java.util.UUID.fromString(uuid));
        } catch (Throwable t) {
            return null;
        }
    }

    /** World a player is in (name), or "". */
    public static String playerWorld(Object player) {
        try {
            return ((Player) player).getWorld().getName();
        } catch (Throwable t) {
            return "";
        }
    }

    /** The World object of a player (for spawning displays). */
    public static Object playerWorldObj(Object player) {
        try {
            return ((Player) player).getWorld();
        } catch (Throwable t) {
            return null;
        }
    }

    /** Resync the client inventory (after swallowed key presses). */
    public static void updateInv(Object player) {
        try {
            ((Player) player).updateInventory();
        } catch (Throwable t) {
            // ignore
        }
    }

    // ------------------------------------------------------------------
    // Player-facing actions
    // ------------------------------------------------------------------

    /** Push the resource pack. sha1hex is 40 hex chars or "" for none. */
    public static void sendPack(Object player, String url, String sha1hex, boolean force) {
        try {
            Player p = (Player) player;
            byte[] hash = null;
            if (sha1hex != null && sha1hex.length() == 40) {
                hash = new byte[20];
                for (int i = 0; i < 20; i++) {
                    hash[i] = (byte) Integer.parseInt(sha1hex.substring(i * 2, i * 2 + 2), 16);
                }
            }
            p.setResourcePack(url, hash, force);
        } catch (Throwable t) {
            try {
                byte[] hash = new byte[20];
                if (sha1hex != null && sha1hex.length() == 40) {
                    for (int i = 0; i < 20; i++) {
                        hash[i] = (byte) Integer.parseInt(sha1hex.substring(i * 2, i * 2 + 2), 16);
                    }
                }
                ((Player) player).setResourcePack(url, hash);
            } catch (Throwable t2) {
                // ignore
            }
        }
    }

    /** Legacy-colored chat message. */
    public static void msg(Object player, String legacy) {
        try {
            ((Player) player).sendMessage(legacy);
        } catch (Throwable t) {
            // ignore
        }
    }

    /** Console command dispatch. */
    public static void consoleCmd(String cmd) {
        try {
            Bukkit.dispatchCommand(Bukkit.getConsoleSender(), cmd);
        } catch (Throwable t) {
            // ignore
        }
    }

    /** Play a custom/namespaced sound at the player. */
    public static void sound(Object player, String key, float volume, float pitch) {
        try {
            Player p = (Player) player;
            p.playSound(p.getLocation(), key, volume, pitch);
        } catch (Throwable t) {
            // ignore
        }
    }

    /** Find a Bukkit material by name; returns 1 when it exists. */
    public static int materialExists(String name) {
        try {
            return org.bukkit.Material.matchMaterial(name) != null ? 1 : 0;
        } catch (Throwable t) {
            return 0;
        }
    }
}
