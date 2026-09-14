use crate::{journal, systemd};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Units,
    Timers,
    Journal,
    Boot,
    Network,
    Sessions,
    Devices,
}

impl Tab {
    pub fn all() -> Vec<Tab> {
        vec![
            Tab::Units,
            Tab::Timers,
            Tab::Journal,
            Tab::Boot,
            Tab::Network,
            Tab::Sessions,
            Tab::Devices,
        ]
    }
    pub fn title(&self) -> &'static str {
        match self {
            Tab::Units => "1 Units",
            Tab::Timers => "2 Timers",
            Tab::Journal => "3 Journal",
            Tab::Boot => "4 Boot",
            Tab::Network => "5 Network",
            Tab::Sessions => "6 Sessions",
            Tab::Devices => "7 Devices",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub action: String,
    pub unit: String,
}

pub struct App {
    pub active_tab: Tab,
    pub units: Vec<systemd::UnitInfo>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub filter: String,
    pub filtering: bool,
    pub status: String,
    pub detail: String,
    pub journal_lines: Vec<String>,
    pub confirm: Option<PendingAction>,
    pub follow: bool,
    last_poll: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            active_tab: Tab::Units,
            units: vec![],
            filtered: vec![],
            selected: 0,
            filter: String::new(),
            filtering: false,
            status: "q salir · j/k mover · tab tabs · s/t/r/e/d accion · / filtrar · f follow"
                .to_string(),
            detail: String::new(),
            journal_lines: vec!["selecciona una unit...".to_string()],
            confirm: None,
            follow: false,
            last_poll: Instant::now(),
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
        self.selected = 0;
        self.rebuild_filter();
    }

    pub fn next_tab(&mut self) {
        let all = Tab::all();
        let i = all.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.set_tab(all[(i + 1) % all.len()]);
    }

    pub fn prev_tab(&mut self) {
        let all = Tab::all();
        let i = all.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.set_tab(all[(i + all.len() - 1) % all.len()]);
    }

    pub async fn refresh_units(&mut self) {
        let kind = match self.active_tab {
            Tab::Timers => "timer",
            Tab::Journal => "service",
            _ => "service",
        };
        // Intento D-Bus primero, fallback a systemctl si no hay bus (containers, etc.)
        match systemd::list_units(kind).await {
            Ok(units) => {
                self.status = format!("{} units via D-Bus ({} totales)", units.len(), kind);
                self.units = units;
            }
            Err(e) => {
                self.status = format!("D-Bus fallo ({e}), fallback systemctl");
                self.units = systemd::list_units_fallback(kind).await.unwrap_or_default();
            }
        }
        self.rebuild_filter();
        self.load_detail().await;
    }

    fn rebuild_filter(&mut self) {
        let q = self.filter.to_lowercase();
        self.filtered = self
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| q.is_empty() || u.name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn selected_unit(&self) -> Option<&systemd::UnitInfo> {
        self.filtered
            .get(self.selected)
            .and_then(|i| self.units.get(*i))
    }

    pub fn select_next(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + 1) % self.filtered.len();
            let app_ptr = self as *mut Self;
            // fire-and-forget visual: cargamos en el proximo tick via polling simple
            // Para no complicar lifetimes, usamos tokio spawn desde el caller (poll).
            unsafe { (*app_ptr).detail = "cargando...".to_string() };
        }
    }

    pub fn select_prev(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + self.filtered.len() - 1) % self.filtered.len();
        }
    }

    /// Llamado despues de mover seleccion o cambiar tab: recarga journal + detail.
    pub async fn load_detail(&mut self) {
        if let Some(u) = self.selected_unit().cloned() {
            self.detail = systemd::show_unit(&u.name).await.unwrap_or_default();
            self.journal_lines = journal::tail_unit(&u.name, 80).await.unwrap_or_default();
            if self.journal_lines.is_empty() {
                self.journal_lines = vec!["(sin logs o sin permiso journal)".to_string()];
            }
        }
        // Tabs no-units: mostrar salida de su comando ecosistema
        match self.active_tab {
            Tab::Timers => {
                if self.units.is_empty()
                    || !matches!(self.units.first().map(|u| u.name.as_str()), Some(n) if n.ends_with(".timer"))
                {
                    // ya cargado via refresh
                }
            }
            Tab::Boot => {
                self.journal_lines =
                    journal::run_cmd("journalctl", &["-b", "--no-pager", "-n", "100"])
                        .await
                        .unwrap_or_default();
                self.detail = journal::run_cmd_string("bootctl", &["status"])
                    .await
                    .unwrap_or_else(|_| "bootctl no disponible (no systemd-boot?)".to_string());
            }
            Tab::Journal => {
                self.journal_lines =
                    journal::run_cmd("journalctl", &["--no-pager", "-n", "100", "-p", "info"])
                        .await
                        .unwrap_or_default();
            }
            Tab::Network => {
                self.detail = journal::run_cmd_string("networkctl", &["status"])
                    .await
                    .unwrap_or_else(|_| "networkd no activo".to_string());
                self.journal_lines = journal::run_cmd(
                    "journalctl",
                    &["-u", "systemd-networkd", "-n", "60", "--no-pager"],
                )
                .await
                .unwrap_or_default();
            }
            Tab::Sessions => {
                self.detail =
                    journal::run_cmd_string("loginctl", &["list-sessions", "--no-legend"])
                        .await
                        .unwrap_or_default();
                self.journal_lines = journal::run_cmd(
                    "journalctl",
                    &["-u", "systemd-logind", "-n", "60", "--no-pager"],
                )
                .await
                .unwrap_or_default();
            }
            Tab::Devices => {
                self.detail = journal::run_cmd_string("udevadm", &["info", "--export-db"])
                    .await
                    .unwrap_or_default();
                self.journal_lines = journal::run_cmd(
                    "journalctl",
                    &["-u", "systemd-udevd", "-n", "60", "--no-pager"],
                )
                .await
                .unwrap_or_default();
            }
            _ => {}
        }
    }

    pub fn toggle_filter_input(&mut self) {
        self.filtering = !self.filtering;
        if self.filtering {
            self.status = "filtro: escribe + Enter".to_string();
        }
    }

    pub fn push_filter(&mut self, c: char) {
        self.filter.push(c);
        self.rebuild_filter();
    }

    pub fn pop_filter(&mut self) {
        self.filter.pop();
        self.rebuild_filter();
    }

    pub async fn apply_filter_and_close(&mut self) {
        self.filtering = false;
        self.selected = 0;
        self.rebuild_filter();
        self.load_detail().await;
    }

    pub async fn request_action(&mut self, action: &str) {
        // Solo Units/Timers tienen accion directa
        let Some(u) = self.selected_unit().cloned() else {
            self.status = "nada seleccionado".to_string();
            return;
        };
        if !matches!(self.active_tab, Tab::Units | Tab::Timers) {
            self.status = "accion solo en Units/Timers".to_string();
            return;
        }
        if matches!(action, "stop" | "restart" | "disable") {
            self.confirm = Some(PendingAction {
                action: action.to_string(),
                unit: u.name.clone(),
            });
            self.status = format!("confirmar {} {} ?  (y/n)", action, u.name);
        } else {
            self.confirm = Some(PendingAction {
                action: action.to_string(),
                unit: u.name.clone(),
            });
            // start/enable tambien piden confirm para enseñar polkit
            self.status = format!("confirmar {} {} ?  (y/n)", action, u.name);
        }
    }

    pub async fn execute_pending(&mut self, p: PendingAction) {
        self.status = format!(
            "ejecutando {} {}... (polkit puede pedir auth)",
            p.action, p.unit
        );
        match systemd::run_action(&p.action, &p.unit).await {
            Ok(out) => {
                self.status = format!(
                    "ok {} {}: {}",
                    p.action,
                    p.unit,
                    out.lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(80)
                        .collect::<String>()
                );
            }
            Err(e) => {
                self.status = format!("error: {e:#}");
            }
        }
        self.refresh_units().await;
    }

    pub async fn toggle_follow(&mut self) {
        self.follow = !self.follow;
        self.status = if self.follow {
            "follow ON (f para salir)".to_string()
        } else {
            "follow OFF".to_string()
        };
        self.last_poll = Instant::now();
    }

    pub async fn maybe_poll_follow(&mut self) {
        // Recarga liviana: si cambiaste seleccion con j/k, refresca detail (debounce simple)
        // y si follow esta ON, re-taulea cada 2s.
        if self.detail == "cargando..." {
            self.load_detail().await;
            return;
        }
        if self.follow && self.last_poll.elapsed().as_secs() >= 2 {
            self.last_poll = Instant::now();
            self.load_detail().await;
        }
    }
}
