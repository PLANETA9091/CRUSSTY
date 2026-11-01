//! Glob matching for class-name patterns.
//!
//! Supported: `*` (any run of chars, including '/'), `?` (exactly one char).
//! Examples: "org/bukkit/**", "net/minecraft/server/dedicated/DedicatedServer*",
//! "org/bukkit/Bukkit".

pub fn matches(pattern: &str, text: &str) -> bool {
    let p = pattern.as_bytes();
    let t = text.as_bytes();
    let (mut pi, mut ti) = (0usize, 0usize);
    let mut star: Option<usize> = None;
    let mut mark = 0usize;
    while ti < t.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == b'*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn exact() {
        assert!(matches("org/bukkit/Bukkit", "org/bukkit/Bukkit"));
        assert!(!matches("org/bukkit/Bukkit", "org/bukkit/CraftServer"));
    }

    #[test]
    fn star_crosses_segments() {
        assert!(matches("org/bukkit/**", "org/bukkit/Bukkit"));
        assert!(matches("org/bukkit/**", "org/bukkit/a/b/C"));
        assert!(matches(
            "net/minecraft/server/**",
            "net/minecraft/server/MinecraftServer"
        ));
        assert!(!matches("org/bukkit/**", "org/spigotmc/CraftServer"));
    }

    #[test]
    fn suffix_star() {
        assert!(matches("DedicatedServer*", "DedicatedServer"));
        assert!(matches("DedicatedServer*", "DedicatedServerX"));
        assert!(!matches("DedicatedServer*", "XServer"));
    }

    #[test]
    fn question_mark() {
        assert!(matches("org/bukkit/Bukki?", "org/bukkit/Bukkit"));
        assert!(!matches("org/bukkit/Bukki?", "org/bukkit/Bukkit2"));
    }
}
