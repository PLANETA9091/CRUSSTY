package org.bukkit;
import java.util.Collection;
import java.util.UUID;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
public final class Bukkit {
    public static Collection<? extends Player> getOnlinePlayers() { throw new UnsupportedOperationException(); }
    public static Player getPlayer(UUID id) { throw new UnsupportedOperationException(); }
    public static boolean dispatchCommand(CommandSender sender, String cmd) { throw new UnsupportedOperationException(); }
    public static CommandSender getConsoleSender() { throw new UnsupportedOperationException(); }
}
