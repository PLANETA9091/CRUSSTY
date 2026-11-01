package org.bukkit.entity;
public interface TextDisplay extends Display {
    void setText(String text);
    void setBackgroundColor(org.bukkit.Color color);
    void setDefaultBackground(boolean def);
    void setSeeThrough(boolean seeThrough);
    void setShadowed(boolean shadowed);
    void setLineWidth(int width);
    void setAlignment(TextAlignment align);
    void setTextOpacity(byte opacity);
    enum TextAlignment { LEFT, CENTER, RIGHT }
}
