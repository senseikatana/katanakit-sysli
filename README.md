# katanakit-sysli — ksys

lazysystemd para el **ecosistema systemd**: units + timers + journal + boot + networkd/logind/udevd.
Sin sudo total: polkit pide auth solo por la acción que confirmás.

Funciona en cualquier distro con systemd (probado en BigLinux/Manjaro systemd 261;
mismo D-Bus en Arch, Deb, Fedora). Si un subsistema no está activo
(ej. networkd en desktop con NetworkManager), lo muestra en vez de romper.

## Requisitos

- Linux con systemd (PID 1 = systemd)
- Rust estable + `systemctl`, `journalctl`
- Opcional Fase 1: `gum`

## Instalación

```bash
npm i -g katanakit-sysli   # binario precompilado (Linux x64/arm64) + comando `ksys`
ksys
```

> npm es solo el instalador (patrón esbuild): el core sigue siendo Rust.
> En macOS/Windows el comando avisa que requiere Linux + systemd en vez de romper.

## Uso desde fuente

```bash
cargo run -- --kind service   # o timer
cargo build --release        # binario en target/release/ksys
./target/release/ksys
```

Fase 1 (validar UX sin TUI):

```bash
./scripts/dev/sys-menu.sh
```

## Teclas

`q` salir · `j/k` mover · `Tab` tabs · `1..7` tab directa ·
`s/t/r/e/d/m` start/stop/restart/enable/disable/mask (+`M` unmask, con confirm `y/n`) ·
`U` alternar scope system/user · `R` daemon-reload · `P` perfiles built-in ·
`/` filtrar · `f` follow journal · `Ctrl-C` salir

## Scope system/user

`U` alterna el manager: system (PID 1) o user (sesión). Las units de usuario
(`gcr-ssh-agent`, `at-spi`) solo existen en el session bus — sin esto son invisibles.
`ksys --scope user` arranca directo en user. El header muestra `[system]` o `[user]`.

## Perfiles built-in

Recetas versionadas (una confirmación por perfil, reporte por paso, polkit por acción):

| Perfil | Qué hace |
|---|---|
| `no-power` | stop+disable+mask `upower`, `power-profiles-daemon` |
| `no-modem` | stop+disable+mask `ModemManager`, `switcheroo-control`, `bolt` |
| `print-on-demand` | stop+disable `cups-browsed`+`cups.service`, enable+start `cups.socket` |
| `no-ssh-a11y` | stop+mask `gcr-ssh-agent.*`, `at-spi-dbus-bus` (scope user) |

En TUI: `P` abre el picker. En CLI: `ksys profile <nombre> --yes` (sin `--yes` se niega).

## CLI = funciones scripteables

Todo lo del TUI existe como subcomando (lo usa el menú gum como única fuente de verdad):

```bash
ksys list --state running --type service   # LISTAR SERVICIOS ACTIVOS
ksys list --user --state running           # los de tu sesión
ksys status upower                         # active/enabled sin parsear texto
ksys stop --user gcr-ssh-agent.socket
ksys mask bolt
ksys daemon-reload                         # REINICIAR SYSTEMD
ksys profiles                              # ver recetas
ksys profile print-on-demand --yes
```

## Seguridad

- Solo lectura por defecto. Sin daemon root, sin sudo total.
- Acciones destructivas (`stop/restart/disable`) piden confirmación.
- `systemctl` + polkit autentican por acción.

## Layout

```
src/main.rs      entry + loop eventos + CLI (subcomandos = funciones)
src/app.rs       estado, tabs, scope, filtro, confirm, follow, picker perfiles
src/systemd.rs   Scope System/User · D-Bus ListUnits + fallback · run_action (mask/unmask) · daemon_reload · is_active/is_enabled
src/profiles.rs  4 perfiles built-in + apply con reporte por paso (+ tests)
src/journal.rs   journalctl tail scope-aware (user usa journal --user)
src/ui.rs        split lista/detail/journal + tabs + popup perfiles
scripts/dev/     sys-menu.sh gum (usa ksys CLI, cae a systemctl)
scripts/sync-version.cjs  mantiene Cargo.toml == versiones npm
npm/             wrapper `katanakit-sysli` (bin `ksys`, sin código)
npm-linux-x64/   binario Linux x64 (lo pone el CI)
npm-linux-arm64/ binario Linux arm64 (lo pone el CI)
.github/workflows/npm-platform.yml  build matrix + publish con provenance
```

## Publicar una versión

```bash
# 1. Sube versión en Cargo.toml, sincroniza npm:
node scripts/sync-version.cjs   # (o edita Cargo y corre el script)
# 2. Tag y push — el CI compila, empaqueta y publica los 3 paquetes:
git tag npm-v0.2.0 && git push origin npm-v0.2.0
```
