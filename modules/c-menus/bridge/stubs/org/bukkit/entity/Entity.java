package org.bukkit.entity;
import org.bukkit.Location;
import org.bukkit.World;
public interface Entity {
    void remove();
    boolean isValid();
    int getEntityId();
    World getWorld();
    boolean teleport(Location loc);
    void setPersistent(boolean persistent);
}
