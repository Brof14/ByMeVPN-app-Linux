use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::{json, Value};
use uuid::Uuid;

fn text<'a>(value: &'a Value, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| value.get(*name).and_then(Value::as_str).filter(|s| !s.is_empty()))
}

fn valid_host(host: &str) -> bool {
    if host.is_empty() || host.contains(['/', '@', '?', '#', ' ', '\\']) {
        return false;
    }
    url::Url::parse(&format!("https://{host}"))
        .is_ok_and(|url| url.host_str().is_some() && url.port().is_none())
}

/// Linux sing-box 1.14+ TUN configuration for the same session contract used
/// by the Windows desktop. Unsupported protocols are intentionally rejected.
pub fn build_tun_config_ext(
    session: &Value,
    bypass_ru: bool,
    ad_block: &str,
    dns_mode: &str,
) -> Result<Value, String> {
    if text(session, &["protocol"]).unwrap_or("vless") != "vless"
        || text(session, &["security"]).unwrap_or("reality") != "reality"
        || !matches!(text(session, &["network"]).unwrap_or("tcp"), "tcp" | "raw")
    {
        return Err("Сервер использует пока не поддерживаемый протокол Linux.".into());
    }
    let host = text(session, &["server", "endpoint"])
        .filter(|host| valid_host(host))
        .ok_or("Сервер вернул некорректный адрес VPN.")?;
    let sni = text(session, &["server_name", "sni"])
        .filter(|sni| valid_host(sni))
        .ok_or("Сервер не вернул корректное имя Reality.")?;
    let uuid = text(session, &["uuid", "id"])
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or("Сервер вернул некорректный UUID.")?;
    let key = text(session, &["public_key", "server_public_key"])
        .filter(|key| URL_SAFE_NO_PAD.decode(key).is_ok_and(|v| v.len() == 32))
        .ok_or("Сервер вернул некорректный ключ Reality.")?;
    let port = session.get("port").and_then(Value::as_u64).unwrap_or(443);
    if !(1..=65535).contains(&port) {
        return Err("Сервер вернул некорректный порт VPN.".into());
    }
    let short_id = text(session, &["short_id"]).unwrap_or("");
    if short_id.len() > 16 || short_id.len() % 2 != 0
        || !short_id.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("Сервер вернул некорректный Reality short ID.".into());
    }
    let flow = text(session, &["flow"]).unwrap_or("xtls-rprx-vision");
    if flow != "xtls-rprx-vision" {
        return Err("Сервер использует неподдерживаемый режим VLESS.".into());
    }
    let dns_ip = match dns_mode {
        "cloudflare" => "1.1.1.1",
        "adguard" => "94.140.14.14",
        "google" => "8.8.8.8",
        _ => return Err("Неизвестный DNS-провайдер.".into()),
    };
    if !matches!(ad_block, "none" | "basic" | "recommended") {
        return Err("Неизвестный режим блокировки рекламы.".into());
    }
    let mut rules = vec![
        json!({"protocol": "dns", "action": "hijack-dns"}),
        json!({"ip_is_private": true, "action": "route", "outbound": "direct"}),
    ];
    // Windows uses geosite:category-ru/geoip:ru. Linux's suffix-based
    // implementation is deliberately narrower; see release gate.
    if bypass_ru {
        rules.push(json!({
            "domain_suffix": [".ru", ".su", ".xn--p1ai", "gosuslugi.ru"],
            "action": "route",
            "outbound": "direct"
        }));
    }
    if ad_block != "none" {
        let mut blocked = vec![
            "doubleclick.net", "googlesyndication.com", "googleadservices.com",
            "adnxs.com", "adsrvr.org", "criteo.com",
        ];
        if ad_block == "recommended" {
            blocked.extend(["taboola.com", "outbrain.com", "scorecardresearch.com"]);
        }
        rules.insert(1, json!({"domain_suffix": blocked, "action": "reject"}));
    }

    Ok(json!({
        "log": {"level": "warn", "timestamp": true},
        "dns": {
            "servers": [{
                "tag": "remote",
                "type": "udp",
                "server": dns_ip,
                "server_port": 53,
                "detour": "proxy"
            }],
            "final": "remote",
            "strategy": "prefer_ipv4"
        },
        "inbounds": [{
            "type": "tun",
            "tag": "tun",
            "interface_name": "bymevpn",
            "address": ["172.31.255.1/30", "fdfe:dcba:9876::1/126"],
            "mtu": 1400,
            "auto_route": true,
            "auto_redirect": true,
            "strict_route": true,
            "dns_mode": "hijack"
        }],
        "outbounds": [
            {
                "type": "vless",
                "tag": "proxy",
                "server": host,
                "server_port": port,
                "uuid": uuid.to_string(),
                "flow": flow,
                "tls": {
                    "enabled": true,
                    "server_name": sni,
                    "utls": {
                        "enabled": true,
                        "fingerprint": text(session, &["fingerprint"]).unwrap_or("chrome")
                    },
                    "reality": {
                        "enabled": true,
                        "public_key": key,
                        "short_id": short_id
                    }
                }
            },
            {"type": "direct", "tag": "direct"}
        ],
        "route": {
            "auto_detect_interface": true,
            "final": "proxy",
            "rules": rules
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Value {
        json!({
            "protocol":"vless",
            "server":"192.0.2.1",
            "port":443,
            "uuid":"11111111-1111-4111-8111-111111111111",
            "security":"reality",
            "server_name":"example.com",
            "public_key":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "short_id":"0123456789abcdef",
            "network":"tcp"
        })
    }

    #[test]
    fn config_routes_linux_tun_through_reality() {
        let result = build_tun_config_ext(&session(), true, "recommended", "cloudflare").unwrap();
        assert_eq!(result["inbounds"][0]["type"], "tun");
        assert_eq!(result["inbounds"][0]["auto_redirect"], true);
        assert_eq!(result["route"]["auto_detect_interface"], true);
        assert_eq!(result["outbounds"][0]["tls"]["reality"]["enabled"], true);
        assert_eq!(result["dns"]["servers"][0]["detour"], "proxy");
        assert_eq!(result["route"]["final"], "proxy");
    }

    #[test]
    fn config_rejects_invalid_sessions_and_settings() {
        let mut invalid = session();
        invalid["protocol"] = json!("hysteria2");
        assert!(build_tun_config_ext(&invalid, false, "none", "cloudflare").is_err());
        invalid = session();
        invalid["uuid"] = json!("broken");
        assert!(build_tun_config_ext(&invalid, false, "none", "cloudflare").is_err());
        invalid = session();
        invalid["public_key"] = json!("broken");
        assert!(build_tun_config_ext(&invalid, false, "none", "cloudflare").is_err());
        invalid = session();
        invalid["server"] = json!("host/path");
        assert!(build_tun_config_ext(&invalid, false, "none", "cloudflare").is_err());
        assert!(build_tun_config_ext(&session(), false, "no-such-filter", "cloudflare").is_err());
        assert!(build_tun_config_ext(&session(), false, "none", "malicious").is_err());
    }
}
