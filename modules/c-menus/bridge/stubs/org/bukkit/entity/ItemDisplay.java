package org.bukkit.entity;
import org.bukkit.inventory.ItemStack;
public interface ItemDisplay extends Display {
    void setItemStack(ItemStack stack);
    void setItemDisplayTransform(ItemDisplayTransform ctx);
    enum ItemDisplayTransform { NONE, THIRDPERSON_LEFTHAND, THIRDPERSON_RIGHTHAND, FIRSTPERSON_LEFTHAND, FIRSTPERSON_RIGHTHAND, HEAD, GUI, GROUND, FIXED }
}
