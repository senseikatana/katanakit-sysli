use anyhow::{Context, Result};
use tokio::process::Command;

/// Info minima de una unit systemd (lo que devuelve ListUnits por D-Bus).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UnitInfo {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
}

type DbusUnitRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    u32,
    String,
    String,
);

/// Lista units por D-Bus: org.freedesktop.systemd1.Manager.ListUnits
/// Funciona en BigLinux/Arch/Deb/Fedora igual: mismo bus, misma interfaz.
pub async fn list_units(kind: &str) -> Result<Vec<UnitInfo>> {
    let conn = zbus::Connection::system()
        .await
        .context("sin system bus (¿systemd corriendo?)")?;
    let proxy = zbus::Proxy::new(
        &conn,
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    )
    .await?;
    let rows: Vec<DbusUnitRow> = proxy.call("ListUnits", &()).await?;
    let mut out: Vec<UnitInfo> = rows
        .into_iter()
        .filter(|r| kind.is_empty() || r.0.ends_with(&format!(".{kind}")) || kind == "all")
        .map(|r| UnitInfo {
            name: r.0,
            description: r.1,
            load_state: r.2,
            active_state: r.3,
            sub_state: r.4,
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Fallback cuando no hay D-Bus (containers sin systemd): parsea systemctl.
pub async fn list_units_fallback(kind: &str) -> Result<Vec<UnitInfo>> {
    let arg = format!("--type={kind}");
    let out = Command::new("systemctl")
        .args(["list-units", &arg, "--all", "--no-legend", "--no-pager"])
        .output()
        .await?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut v = vec![];
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(name), Some(_load), Some(active), Some(sub)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        v.push(UnitInfo {
            name: name.to_string(),
            description: line.to_string(),
            load_state: String::new(),
            active_state: active.to_string(),
            sub_state: sub.to_string(),
        });
    }
    Ok(v)
}

pub async fn show_unit(unit: &str) -> Result<String> {
    let out = Command::new("systemctl")
        .args(["show", unit, "--no-pager"])
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| {
            l.starts_with("Description=")
                || l.starts_with("ActiveState=")
                || l.starts_with("SubState=")
                || l.starts_with("ExecMainStartTimestamp=")
                || l.starts_with("FragmentPath=")
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Ejecuta accion con systemctl (polkit pide auth solo por esa accion).
/// A proposito NO usamos sudo: dejamos que polkit decida. Asi evitamos sudo total.
pub async fn run_action(action: &str, unit: &str) -> Result<String> {
    let sys_action = match action {
        "start" => "start",
        "stop" => "stop",
        "restart" => "restart",
        "enable" => "enable",
        "disable" => "disable",
        _ => anyhow::bail!("accion desconocida: {action}"),
    };
    let mut cmd = Command::new("systemctl");
    // enable/disable llevan --now? No: separado para no sorprender. Solo la accion pura.
    cmd.arg(sys_action).arg(unit);
    let out = cmd.output().await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string()
            + String::from_utf8_lossy(&out.stderr).as_ref())
    } else {
        anyhow::bail!(
            "{}",
            String::from_utf8_lossy(&out.stderr)
                .lines()
                .next()
                .unwrap_or("systemctl fallo")
        );
    }
}
