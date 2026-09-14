# CHANGELOG — katanakit-sysli

## [Unreleased]

Checklist de la iteración systemd-control (scope + mask + perfiles + CLI):

- [x] `Scope::System/User` en `systemd.rs` (system bus + session bus, `systemctl --user`)
- [x] Acciones `mask/unmask` en TUI (`m`/`M`) y CLI, con confirm
- [x] `daemon-reload` en TUI (`R`) y CLI (`ksys daemon-reload [--user]`)
- [x] Filtro `--state running` en `ksys list` (LISTAR SERVICIOS ACTIVOS, ambos scopes)
- [x] `ksys status <unit>` (active/enabled sin parsear texto)
- [x] `src/profiles.rs` built-in: `no-power`, `no-modem`, `print-on-demand`, `no-ssh-a11y`
- [x] Picker de perfiles en TUI (`P`) + `ksys profile <nombre> --yes` (sin `--yes` se niega)
- [x] `sys-menu.sh` con scope, mask, daemon-reload, running y perfiles vía CLI
- [x] Tests invariantes de perfiles (4/4) + pruebas reales de lectura en BigLinux
- [x] `AGENTS.md` creado; README con CLI, perfiles y scope
- [ ] Publicar `npm-v0.1.0` (requiere secret `NPM_TOKEN` + tag)

## [0.1.0] — base

- [x] TUI Ratatui: 7 tabs ecosistema, split lista/detail/journal, `s/t/r/e/d`, `/` filtro, `f` follow
- [x] Lectura por D-Bus `ListUnits` con fallback `systemctl`
- [x] Wrapper npm `katanakit-sysli` + paquetes `@senseikatana/ksys-linux-*`
- [x] Workflow `npm-platform.yml` (build matrix + publish con provenance)
