#!/usr/bin/env bash
# Fase 1 — validacion UX con gum (se tira, no se porta a Rust).
# Requiere: gum, systemctl, journalctl. Funciona en BigLinux/Arch/Deb/Fedora.
set -euo pipefail

pick_area() {
  gum choose --header "ksys Fase1 — ecosistema systemd" \
    "services" "timers" "logs" "boot" "network" "sessions" "salir"
}

pick_service() {
  systemctl list-units --type=service --all --no-legend --no-pager \
    | awk '{print $1}' | gum filter --placeholder "fuzzy: nginx..."
}

tail_logs() {
  local unit="$1"
  echo "== journalctl -f -u $unit (Ctrl+C para volver) =="
  journalctl -f -u "$unit" --no-pager -n 50 || true
}

service_action() {
  local unit="$1"
  local act
  act=$(gum choose --header "$unit" "status" "restart" "stop" "start" "enable" "disable" "logs" "volver")
  case "$act" in
    status) systemctl status "$unit" --no-pager || true ;;
    restart|stop|start)
      gum confirm "¿$act $unit? (polkit pedira auth solo por esta accion)" || return 0
      systemctl "$act" "$unit" ;;
    enable|disable)
      gum confirm "¿$act $unit?" || return 0
      systemctl "$act" "$unit" ;;
    logs) tail_logs "$unit" ;;
  esac
  gum confirm "¿volver al menu?" || exit 0
}

main() {
  while true; do
    area=$(pick_area)
    case "$area" in
      services)
        unit=$(pick_service); [ -n "$unit" ] && service_action "$unit" ;;
      timers) systemctl list-timers --no-pager || true; read -rp "Enter..." _ ;;
      logs) journalctl --no-pager -n 100 -p info || true; read -rp "Enter..." _ ;;
      boot) journalctl -b --no-pager -n 80 || true; bootctl status || true; read -rp "Enter..." _ ;;
      network) networkctl status || true; read -rp "Enter..." _ ;;
      sessions) loginctl list-sessions || true; read -rp "Enter..." _ ;;
      salir|*) exit 0 ;;
    esac
  done
}
main "$@"
