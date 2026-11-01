package org.bukkit;
import org.bukkit.entity.Entity;
public interface World extends RegionAccessor {
    <T extends Entity> T spawn(Location loc, Class<T> clazz);
    String getName();
}
