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

## Uso

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
`s/t/r/e/d` start/stop/restart/enable/disable (con confirm `y/n`) ·
`/` filtrar · `f` follow journal · `Ctrl-C` salir

## Seguridad

- Solo lectura por defecto. Sin daemon root, sin sudo total.
- Acciones destructivas (`stop/restart/disable`) piden confirmación.
- `systemctl` + polkit autentican por acción.

## Layout

```
src/main.rs      entry + loop eventos
src/app.rs       estado, tabs, filtro, confirm, follow
src/systemd.rs   D-Bus ListUnits (fallback systemctl) + run_action
src/journal.rs   journalctl tail + comandos ecosistema
src/ui.rs        split lista/detail/journal + tabs
scripts/dev/     Fase 1 gum (descartable)
```
