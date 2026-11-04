# ByMeVPN for Linux — release acceptance

Status: PREVIEW, not production-ready.

## Implemented in source

- Windows desktop UI structure, styles, shield branding, screens, navigation and Rust IPC commands imported.
- Tauri 2 / Rust Linux host; .deb and AppImage targets.
- Linux Secret Service account credentials rather than Windows Credential Manager.
- XDG autostart desktop entry and allowlisted HTTPS links opened with xdg-open.
- sing-box 1.14+ VLESS TCP + REALITY outbound, Linux TUN, auto_route, auto_redirect, strict_route and DNS hijack.
- Routing loop prevention via route.auto_detect_interface.
- Conservative connection status: public IPv4 must differ from original.
- VPN config stored with owner-only 0600 permissions; engine process stopped and config deleted on disconnect.
- Node UI regression suite and GitHub Actions Ubuntu compilation/build pipeline.

## Critical unverified behaviors

1. Real VPN connection on an Ubuntu 24.04 desktop with legitimate ByMeVPN account, VPN node and backend deployed.
2. IPv6, DNS and WebRTC leak behavior across NetworkManager, systemd-resolved, diverse networks and route changes.
3. Crash-safe route cleanup. Engine termination is not a substitute for a kernel kill switch.
4. Process privilege architecture: current system-owned binary uses CAP_NET_ADMIN and CAP_NET_RAW. Harden with a separate audited helper and narrow polkit/IPC authorization.
5. Local direct routing: Linux domain suffix bypass is not full Windows geosite/geoip policy. Russian IP ranges and non-.ru domains may still use VPN.
6. Advert blocking: Linux blocks a conservative static domain set, not the Windows comprehensive geosite rule database.
7. Hysteria2 and other unsupported protocols are not yet implemented in the source Windows client.
8. Live subscription, trial, Telegram, payment, device management and recurring billing require backend acceptance. The existing Windows source intentionally disables unavailable server features.
9. GUI and networking behavior on Fedora, Arch, KDE, GNOME, X11 and Wayland, including suspend/resume.
10. Signed installers, updater, deep-link routing, accessibility and cross-distribution QA.

## Required tests before public release

- [ ] UI Node tests pass
- [ ] Cargo check and Cargo unit tests pass
- [ ] GitHub Actions produces both .deb and .AppImage artifacts
- [ ] No blank screens at standard and high DPI
- [ ] Complete login, registration, Telegram and trial test flows
- [ ] Real TUN interface, routing and VPN public IP verification
- [ ] IPv4/IPv6 and DNS leak tests on at least two networks
- [ ] Connection cancellation, OS sleep/resume, engine crash and cleanup
- [ ] Split tunnel and ad-block behavior matches stated capabilities
- [ ] Verified system-engine update and runtime permission lifecycle
- [ ] Independent security review and explicit production release gate

Passing automated tests or seeing a changed public IPv4 does not prove leak-free routing.
