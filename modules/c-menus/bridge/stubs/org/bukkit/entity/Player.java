package org.bukkit.entity;
import java.util.UUID;
import org.bukkit.Location;
public interface Player extends Entity {
    Location getEyeLocation();
    Location getLocation();
    UUID getUniqueId();
    String getName();
    void setResourcePack(String url, byte[] hash, boolean force);
    void setResourcePack(String url, byte[] hash);
    void sendMessage(String message);
    void playSound(Location loc, String sound, float volume, float pitch);
    void updateInventory();
}
