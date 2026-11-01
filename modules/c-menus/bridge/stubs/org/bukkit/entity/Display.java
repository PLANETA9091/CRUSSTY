package org.bukkit.entity;
import org.bukkit.util.Transformation;
public interface Display extends Entity {
    void setTransformation(Transformation t);
    void setInterpolationDelay(int delay);
    void setInterpolationDuration(int dur);
    void setTeleportDuration(int dur);
    void setViewRange(float range);
    void setShadowRadius(float r);
    void setShadowStrength(float s);
    void setBrightness(Brightness b);
    void setBillboard(Billboard mode);
    enum Billboard { FIXED, VERTICAL, HORIZONTAL, CENTER }
    class Brightness {
        public Brightness(int blockLight, int skyLight) {}
    }
}
