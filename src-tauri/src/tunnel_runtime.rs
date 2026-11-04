use std::{
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
};

const APPROVED: [&str; 2] = ["/usr/bin/sing-box", "/usr/local/bin/sing-box"];

pub fn find_runtime() -> Result<PathBuf, String> {
    for path in APPROVED {
        let binary = PathBuf::from(path);
        if binary.exists() {
            verify_runtime(binary.parent().expect("absolute engine path"))?;
            return Ok(binary);
        }
    }
    Err("sing-box не установлен. Установите sing-box 1.14+ из официального репозитория.".into())
}

/// Reject executable substitution. TUN privileges must be explicitly configured
/// by the system administrator on a root-owned file, never on our GUI.
pub fn verify_runtime(directory: &Path) -> Result<(), String> {
    let binary = directory.join("sing-box");
    let resolved = binary.canonicalize()
        .map_err(|_| "Не найден системный sing-box.".to_string())?;
    if !APPROVED.iter().any(|p| resolved == Path::new(p)) {
        return Err("Неожиданный путь VPN-движка. Ссылки на сторонние исполняемые файлы запрещены.".into());
    }
    let meta = std::fs::metadata(&resolved)
        .map_err(|e| format!("Невозможно проверить sing-box: {e}"))?;
    if !meta.is_file() || meta.uid() != 0 || meta.permissions().mode() & 0o022 != 0 {
        return Err("sing-box должен принадлежать root и быть недоступным для записи обычным пользователям.".into());
    }
    let ver = Command::new(&resolved).arg("version").output()
        .map_err(|e| format!("Не удалось проверить версию sing-box: {e}"))?;
    if !ver.status.success() {
        return Err("sing-box не запускается. Проверьте системную установку.".into());
    }
    let stdout = String::from_utf8_lossy(&ver.stdout);
    let version = stdout.split_whitespace()
        .find(|s| s.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .ok_or("Неизвестная версия sing-box.")?;
    let parts: Vec<_> = version.split('.').take(2)
        .map(|s| s.parse::<u32>())
        .collect();
    if parts.len() != 2 || parts[0].as_ref().is_err() || parts[1].as_ref().is_err()
        || *parts[0].as_ref().unwrap() < 1
        || (*parts[0].as_ref().unwrap() == 1 && *parts[1].as_ref().unwrap() < 14)
    {
        return Err("Для Linux TUN требуется sing-box версии 1.14 или новее.".into());
    }
    let caps = Command::new("getcap").arg(&resolved).output()
        .map_err(|_| "Не найден getcap. Установите libcap2-bin.".to_string())?;
    let text = String::from_utf8_lossy(&caps.stdout);
    if !caps.status.success() || !text.contains("cap_net_admin")
        || !text.contains("cap_net_raw") || !(text.contains("=ep") || text.contains("+ep"))
    {
        return Err("sing-box нужны CAP_NET_ADMIN и CAP_NET_RAW: sudo setcap cap_net_admin,cap_net_raw+ep /usr/bin/sing-box".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_untrusted_executable_paths() {
        assert!(verify_runtime(Path::new("/tmp")).is_err());
    }
}
