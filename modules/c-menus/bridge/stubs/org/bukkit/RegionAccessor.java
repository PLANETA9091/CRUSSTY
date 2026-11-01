package org.bukkit;
import org.bukkit.entity.Entity;
public interface RegionAccessor {
    <T extends Entity> T spawn(Location loc, Class<T> clazz);
}
