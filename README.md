# ByMeVPN for Linux

Native Linux desktop port of [ByMeVPN for Windows](https://github.com/Brof14/ByMeVPN-app-windows).

The **HTML/CSS/JS UI is carried across from Windows**, with the same screens and backend API contract. The desktop shell is **Tauri 2 / Rust**; the Linux tunnel engine is **sing-box**, using TUN and VLESS + REALITY.

> **Experimental preview, not a security-certified VPN.** Linux tunnelling, DNS/IPv6 leaks, rollback on network failure, and all billing/account flows require acceptance tests on real Linux machines. This port intentionally does not claim an effective kill switch.

## Build

On Ubuntu 24.04 / Debian-compatible Linux:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libwebkit2gtk-4.1-dev libdbus-1-dev
```

Install the current stable Rust toolchain, Node.js 22+, then:

```bash
node --check ui/app.js
node --test tests/frontend-contract.test.mjs
cargo check --manifest-path src-tauri/Cargo.toml
npx --yes @tauri-apps/cli@2 build
```

The build produces `.deb` and `.AppImage` under `src-tauri/target/release/bundle/`.

## VPN runtime prerequisite

For VPN connections, install **sing-box 1.14+** from the [official distribution](https://sing-box.sagernet.org/installation/package-manager/). The application only executes `/usr/bin/sing-box` or `/usr/local/bin/sing-box`; it will not download or silently execute arbitrary user-provided binaries.

Linux TUN/routing needs elevated network capabilities; the GUI must **not** run as root. A privileged administrator can provision the system-owned engine explicitly:

```bash
sudo apt-get install -y libcap2-bin
sudo setcap cap_net_admin,cap_net_raw+ep /usr/bin/sing-box
getcap /usr/bin/sing-box
```

This grants network capabilities to any permitted caller of that engine; it is a system-level security decision, **not a sandbox**. Keep the engine root-owned, update it promptly, and reapply capabilities after a package update if needed. For production, use a dedicated root-owned helper with narrowly scoped policy authorization and crash-safe routing rollback instead.

The desktop uses Secret Service (GNOME Keyring / KWallet backend) to protect session tokens. It needs an active D-Bus secrets provider.

The application currently supports VLESS TCP + REALITY nodes only. Hysteria2 and other unsupported protocol nodes remain disabled.

## Features and limitations

Login, registration, Telegram session, subscriptions, trial, plans, payments, account overview, server selector, device UI, settings, and RU/EN UI follow the Windows client. API operations depend on the actual server implementation; unsupported remote operations remain visibly unavailable rather than simulated.

On Linux, auto-start writes a desktop entry under `~/.config/autostart/` and external URLs open with `xdg-open`. Do not conflate successful compilation with a live network test. See `docs/LINUX_RELEASE_GATE.md`.

## References

- [Windows application (design source)](https://github.com/Brof14/ByMeVPN-app-windows)
- [sing-box TUN](https://sing-box.sagernet.org/configuration/inbound/tun/)
- [sing-box VLESS](https://sing-box.sagernet.org/configuration/outbound/vless/)
