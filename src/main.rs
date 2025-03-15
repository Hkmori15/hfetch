use std::fs;
use std::process::Command;
use std::str;

// Use const ASCII-art for better perfomance
const ASCII_LOGO: &str = "\
\x1b[1;35m.------.
\x1b[1;35m|H.--. |
\x1b[1;35m| :/\\: |
\x1b[1;35m| (__) |
\x1b[1;35m| '--'H|
\x1b[1;35m`------'
\x1b[0m";

fn get_hostname() -> String {
    match fs::read_to_string("/etc/hostname") {
        Ok(hostname) => hostname.trim().to_string(),
        Err(_) => {
            // Fallback
            match Command::new("hostname").output() {
                Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
                Err(_) => String::from("unknown"),
            }
        }
    }
}

fn get_distro_name() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .replace("PRETTY_NAME=", "")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }

    if let Ok(content) = fs::read_to_string("/etc/lsb-release") {
        for line in content.lines() {
            if line.starts_with("DISTRIB_DESCRIPTION=") {
                return line
                    .replace("DISTRIB_DESCRIPTION=", "")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }

    String::from("unknown")
}

fn get_kernel_version() -> String {
    match Command::new("uname").arg("-r").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => match fs::read_to_string("/proc/version") {
            Ok(version) => {
                let parts: Vec<&str> = version.split_whitespace().collect();

                if parts.len() >= 3 {
                    parts[2].to_string()
                } else {
                    String::from("unknown")
                }
            }
            Err(_) => String::from("unknown"),
        },
    }
}

fn get_init_system() -> String {
    if fs::metadata("/run/systemd/system").is_ok() {
        return String::from("systemd");
    }

    if let Ok(output) = Command::new("ps").args(["ax"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);

        if output_str.contains(" runit") || output_str.contains("/runit") {
            return String::from("runit");
        }
    }

    if fs::metadata("/etc/runit").is_ok()
        || fs::metadata("/etc/sv").is_ok()
        || fs::metadata("/run/runit").is_ok()
        || fs::metadata("/var/service").is_ok()
    {
        return String::from("runit");
    }

    if fs::metadata("/etc/init.d").is_ok() && fs::metadata("/etc/runlevels").is_ok() {
        return String::from("openrc");
    }

    if fs::metadata("/etc/init").is_ok() {
        if let Ok(entries) = fs::read_dir("/etc/init") {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "conf" {
                        return String::from("upstart");
                    }
                }
            }
        }
    }

    if fs::metadata("/etc/s6").is_ok() || fs::metadata("/bin/s6-svscan").is_ok() {
        return String::from("s6");
    }

    if fs::metadata("/etc/dinit.d").is_ok() || fs::metadata("/bin/dinit").is_ok() {
        return String::from("dinit");
    }

    if fs::metadata("/sbin/init").is_ok() {
        if let Ok(output) = Command::new("readlink").args(["-f", "/sbin/init"]).output() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if path.contains("systemd") {
                return String::from("systemd");
            } else if path.contains("upstart") {
                return String::from("upstart");
            } else if path.contains("openrc") {
                return String::from("openrc");
            } else if path.contains("runit") {
                return String::from("runit");
            } else if path.contains("s6") {
                return String::from("s6");
            } else if path.contains("dinit") {
                return String::from("dinit");
            } else {
                return String::from("unknown");
            }
        }
    }

    String::from("unknown")
}

fn get_package_count() -> (u32, u32) {
    let mut native_count = 0;
    let mut flatpak_count = 0;

    // Debian/Ubuntu
    if native_count == 0 {
        if let Ok(output) = Command::new("dpkg-query")
            .args(["-f", "%{binary:Package}\n", "-W"])
            .output()
        {
            if output.status.success() {
                native_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        }
    }

    // Fedora
    if native_count == 0 {
        if let Ok(output) = Command::new("rpm").args(["-qa"]).output() {
            if output.status.success() {
                native_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        }
    }

    // Arch
    if native_count == 0 {
        if let Ok(output) = Command::new("pacman").args(["-Q"]).output() {
            if output.status.success() {
                native_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        }
    }

    // Void
    if native_count == 0 {
        if let Ok(output) = Command::new("xbps-query").args(["-l"]).output() {
            if output.status.success() {
                native_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        }
    }

    // Alpine
    if native_count == 0 {
        if let Ok(output) = Command::new("apk").args(["info"]).output() {
            if output.status.success() {
                native_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        }
    }

    // Gentoo
    if native_count == 0 {
        if let Ok(output) = Command::new("qlist").args(["-I"]).output() {
            if output.status.success() {
                native_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        } else if let Ok(output) = Command::new("ls").args(["-d", "/var/db/pkg/*/*"]).output() {
            if output.status.success() {
                native_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        }
    }

    // NixOS
    if native_count == 0 {
        if let Ok(output) = Command::new("nix-store")
            .args(["--query", "--requisites", "/run/current-system"])
            .output()
        {
            if output.status.success() {
                native_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        }
    }

    // OpenSUSE
    if native_count == 0 {
        if let Ok(output) = Command::new("zypper")
            .args(["search", "--installed-only"])
            .output()
        {
            if output.status.success() {
                // Skip header and whitespace lines
                let count = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| line.starts_with('i'))
                    .count() as u32;

                if count > 0 {
                    native_count = count;
                }
            }
        }
    }

    // Solus
    if native_count == 0 {
        if let Ok(output) = Command::new("eopkg").args(["list-installed"]).output() {
            if output.status.success() {
                let count = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.starts_with("Installed"))
                    .filter(|line| !line.is_empty())
                    .count() as u32;

                if count > 0 {
                    native_count = count;
                }
            }
        }
    }

    // Clear
    if native_count == 0 {
        if let Ok(output) = Command::new("swupd").args(["bundle-list"]).output() {
            if output.status.success() {
                let count = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.contains("bundles installed"))
                    .filter(|line| !line.is_empty())
                    .count() as u32;

                if count > 0 {
                    native_count = count;
                }
            }
        }
    }

    // Flatpak
    if let Ok(output) = Command::new("flatpak").args(["list"]).output() {
        if output.status.success() {
            flatpak_count = String::from_utf8_lossy(&output.stdout).lines().count() as u32;
        }
    }

    (native_count, flatpak_count)
}

fn get_mem_info() -> (f64, f64) {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        let mut total: f64 = 0.0;
        let mut free: f64 = 0.0;
        let mut buffers: f64 = 0.0;
        let mut cached: f64 = 0.0;
        let mut found_count = 0;

        for line in content.lines() {
            if line.starts_with('M') {
                if line.starts_with("MemTotal:") {
                    total = parse_mem_value(line);
                    found_count += 1;
                } else if line.starts_with("MemFree:") {
                    free = parse_mem_value(line);
                    found_count += 1;
                }
            } else if line.starts_with('B') && line.starts_with("Buffers:") {
                buffers = parse_mem_value(line);
                found_count += 1;
            } else if line.starts_with('C')
                && line.starts_with("Cached:")
                && !line.starts_with("SwapCached:")
            {
                cached = parse_mem_value(line);
                found_count += 1;
            }

            if found_count == 4 {
                break;
            }
        }

        // Calculate used mem: accounting for buffers/cache
        let used = total - free - buffers - cached;
        let total_gb = total / 1024.0 / 1024.0;
        let used_gb = used / 1024.0 / 1024.0;

        return (used_gb, total_gb);
    }

    (0.0, 0.0)
}

// Support func for parse mem value from str to float
fn parse_mem_value(line: &str) -> f64 {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() >= 2 {
        if let Ok(value) = parts[1].parse::<f64>() {
            return value;
        }
    }

    0.0
}

fn print_info(
    hostname: &str,
    distro: &str,
    kernel: &str,
    init: &str,
    native_pkgs: u32,
    flatpak_pkgs: u32,
    used_mem: f64,
    total_mem: f64,
) {
    // Get terminal width for formatting
    let term_width = get_terminal_width();

    let logo = get_ascii_logo();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len();

    let info = [
        format!("\x1b[1;34mhostname:\x1b[0m {}", hostname),
        format!("\x1b[1;34mdistro:\x1b[0m {}", distro),
        format!("\x1b[1;34mkernel:\x1b[0m {}", kernel),
        format!("\x1b[1;34minit:\x1b[0m {}", init),
        format!(
            "\x1b[1;34mpackages:\x1b[0m native: {} | flatpak: {}",
            native_pkgs, flatpak_pkgs
        ),
        format!(
            "\x1b[1;34mmemory:\x1b[0m {:.2} GB | {:.2} GB",
            used_mem, total_mem
        ),
    ];

    // We print logo and info side by side
    let max_info_lines = info.len();
    let max_lines = std::cmp::max(logo_height, max_info_lines);

    for i in 0..max_lines {
        if i < logo_height {
            print!("{}", logo_lines[i]);

            let logo_width = strip_ansi_escape_codes(logo_lines[i]).chars().count();
            let padding = if logo_width < term_width / 2 {
                4 // fixed padding
            } else {
                2
            };

            print!("{}", " ".repeat(padding));
        } else {
            // If logo is shorter than info just add appropiate padding
            let max_logo_width = logo_lines
                .iter()
                .map(|line| strip_ansi_escape_codes(line).chars().count())
                .max()
                .unwrap_or(0);

            print!("{}", " ".repeat(max_logo_width + 4));
        }

        if i < max_info_lines {
            println!("{}", info[i]);
        } else {
            println!();
        }
    }
}

fn strip_ansi_escape_codes(s: &str) -> String {
    let mut res = String::new();
    let mut in_escape = false;

    for c in s.chars() {
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            res.push(c);
        }
    }

    res
}

fn get_terminal_width() -> usize {
    if let Ok(output) = Command::new("stty").args(["size"]).output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = output_str.split_whitespace().collect();

            if parts.len() >= 2 {
                if let Ok(width) = parts[1].parse::<usize>() {
                    return width;
                }
            }
        }
    }

    80 // default width
}

fn get_ascii_logo() -> &'static str {
    ASCII_LOGO
}

fn main() {
    let hostname = get_hostname();
    let distro = get_distro_name();
    let kernel = get_kernel_version();
    let init = get_init_system();
    let (native_pkgs, flatpak_pkgs) = get_package_count();
    let (used_mem, total_mem) = get_mem_info();

    print_info(
        &hostname,
        &distro,
        &kernel,
        &init,
        native_pkgs,
        flatpak_pkgs,
        used_mem,
        total_mem,
    );
}
