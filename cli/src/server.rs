use crate::lib;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn read_config(server: &PathBuf) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(server.join("crussty.toml")).ok()?;
    Some(parse_tomlish(&text))
}

fn parse_tomlish(text: &str) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let v = v.trim().trim_matches('"').to_string();
        let key = k.trim().to_string();
        if section.is_empty() {
            obj.insert(key, serde_json::Value::String(v));
        } else {
            let entry = obj
                .entry(section.clone())
                .or_insert_with(|| serde_json::Value::Object(Default::default()));
            if let serde_json::Value::Object(m) = entry {
                m.insert(key, serde_json::Value::String(v));
            }
        }
    }
    serde_json::Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_section_with_quotes_stripped() {
        let v = parse_tomlish("[server]\nkernel = \"purpur-1.21.10.jar\"\nmemory = \"2G\"\n");
        assert_eq!(v["server"]["kernel"], "purpur-1.21.10.jar");
        assert_eq!(v["server"]["memory"], "2G");
    }

    #[test]
    fn skips_comments_and_empty_lines() {
        let v = parse_tomlish("# comment\n\n   \n# another\n[server]\nmemory = \"2G\"\n");
        assert_eq!(v["server"]["memory"], "2G");
        assert_eq!(v.as_object().unwrap().len(), 1);
    }

    #[test]
    fn top_level_keys_without_section() {
        let v = parse_tomlish("port = \"25565\"\nname = crussty\n");
        assert_eq!(v["port"], "25565");
        assert_eq!(v["name"], "crussty");
    }

    #[test]
    fn unknown_sections_create_nested_objects() {
        let v = parse_tomlish("[custom]\nfoo = \"bar\"\n");
        assert_eq!(v["custom"]["foo"], "bar");
    }

    #[test]
    fn section_name_is_trimmed() {
        let v = parse_tomlish("[ server ]\nmemory = \"4G\"\n");
        assert_eq!(v["server"]["memory"], "4G");
    }

    #[test]
    fn lines_without_equals_are_skipped() {
        let v = parse_tomlish("just text\n[server]\nmemory = \"2G\"\n");
        assert_eq!(v["server"]["memory"], "2G");
        assert_eq!(v.get("just").is_none(), true);
    }
}

fn cfg_str(cfg: &serde_json::Value, section: &str, key: &str, fallback: &str) -> String {
    cfg.get(section)
        .and_then(|s| s.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or(fallback)
        .to_string()
}

pub fn run() -> i32 {
    let server = lib::server_dir();
    let Some(cfg) = read_config(&server) else {
        eprintln!("crussty: no crussty.toml here — run 'crussty init' first");
        return 1;
    };
    let kernel = cfg_str(&cfg, "server", "kernel", "purpur-1.21.10.jar");
    let memory = cfg_str(&cfg, "server", "memory", "2G");

    let jar = server.join("versions").join(&kernel);
    if !jar.exists() {
        eprintln!("crussty: kernel jar not found at {}", jar.display());
        return 1;
    }
    if !lib::jvm_pids(&server).is_empty() {
        eprintln!("crussty: server already running (see 'crussty stop')");
        return 1;
    }

    let pid_file = format!(
        "/tmp/crussty-{}.pid",
        server.file_name().unwrap_or_default().to_string_lossy()
    );
    let trace = format!("onload.pidFile={pid_file}");

    let log_path = server.join("logs").join("latest.log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap_or_else(|e| panic!("cannot open {}: {e}", log_path.display()));
    let _ = log_file.set_len(0);
    let mut log_out = log_file;

    let mut child = match Command::new("java")
        .current_dir(&server)
        .arg(format!("-Xms{memory}"))
        .arg(format!("-Xmx{memory}"))
        .arg(format!("-agentpath:./libcrussty_runtime.so={trace}"))
        .arg("-jar")
        .arg("launcher/launcher.jar")
        .arg("--nogui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("crussty: failed to start java: {e}");
            return 1;
        }
    };

    println!("crussty: server started (pid {})", child.id());
    println!("crussty: console attached — type commands (Ctrl+D to detach)");

    tee(child.stdout.take(), Box::new(std::io::stdout()), &mut log_out);
    tee(child.stderr.take(), Box::new(std::io::stderr()), &mut log_out);

    let mut con = child.stdin.take();
    let mut line = String::new();
    loop {
        line.clear();
        let n = std::io::stdin().read_line(&mut line).unwrap_or(0);
        if n == 0 {
            break;
        }
        if line.trim() == "stop" {
            println!("crussty: sending stop");
            let _ = con.as_mut().map(|c| c.write_all(b"stop\n"));
            break;
        }
        if let Some(con) = con.as_mut() {
            let _ = con.write_all(line.as_bytes());
            let _ = con.flush();
        }
    }
    let _ = child.wait();
    0
}

fn tee<R: Read + Send + 'static>(
    pipe: Option<R>,
    mut out: impl Write + Send + 'static,
    log: &mut std::fs::File,
) {
    let Some(pipe) = pipe else { return };
    let mut buf = BufReader::new(pipe);
    let mut shared = log.try_clone().unwrap();
    std::thread::spawn(move || {
        let mut line = String::new();
        loop {
            line.clear();
            let n = buf.read_line(&mut line).unwrap_or(0);
            if n == 0 {
                break;
            }
            let _ = write!(out, "{line}");
            let _ = out.flush();
            let _ = writeln!(shared, "{line}");
        }
    });
}

fn tail_pos(file: &mut std::fs::File, lines: u64) -> u64 {
    use std::io::Seek;
    let len = std::io::Seek::seek(file, std::io::SeekFrom::End(0)).unwrap_or(0);
    if len == 0 {
        return 0;
    }
    let mut pos = len;
    let mut seen = 0u64;
    let mut buf = [0u8; 4096];
    while pos > 0 && seen <= lines {
        let chunk = std::cmp::min(pos, buf.len() as u64) as usize;
        pos -= chunk as u64;
        let _ = file.seek(std::io::SeekFrom::Start(pos));
        let _ = std::io::Read::read_exact(&mut std::io::Read::by_ref(file), &mut buf[..chunk]);
        seen += buf[..chunk].iter().filter(|&&b| b == b'\n').count() as u64;
    }
    if seen > lines {
        while pos < len {
            let _ = file.seek(std::io::SeekFrom::Start(pos));
            let mut b = [0u8; 1];
            if std::io::Read::read_exact(&mut std::io::Read::by_ref(file), &mut b).is_err() {
                break;
            }
            pos += 1;
            if b[0] == b'\n' {
                break;
            }
        }
    }
    pos
}

pub fn stop() -> i32 {
    let server = lib::server_dir();
    let pids = lib::jvm_pids(&server);
    if pids.is_empty() {
        eprintln!("crussty: no server running");
        return 1;
    }
    for pid in pids {
        println!("crussty: SIGTERM -> {pid}");
        let _ = Command::new("kill").arg("-TERM").arg(pid.to_string()).status();
    }
    0
}

pub fn log(follow: bool) -> i32 {
    let server = lib::server_dir();
    let path = server.join("logs").join("latest.log");
    let mut file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("crussty: no logs/latest.log yet (start with 'crussty run')");
            return 1;
        }
    };
    let start = if follow { tail_pos(&mut file, 40) } else { 0 };
    let mut reader = BufReader::new(&mut file);
    let mut pos = start;
    loop {
        if let Err(_) = std::io::Seek::seek(&mut reader, std::io::SeekFrom::Start(pos)) {
            break;
        }
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                if !follow {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
            Ok(n) => {
                pos += n as u64;
                print!("{line}");
                let _ = std::io::stdout().flush();
            }
            Err(_) => break,
        }
    }
    0
}

pub fn reload() -> i32 {
    let server = lib::server_dir();
    let pids = lib::jvm_pids(&server);
    if pids.is_empty() {
        eprintln!("crussty: no server running");
        return 1;
    }
    for pid in pids {
        println!("crussty: hot-reload (SIGUSR1) -> {pid}");
        let _ = Command::new("kill").arg("-USR1").arg(pid.to_string()).status();
    }
    0
}