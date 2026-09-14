use anyhow::Result;
use tokio::process::Command;

pub async fn tail_unit(unit: &str, n: usize) -> Result<Vec<String>> {
    run_cmd(
        "journalctl",
        &["-u", unit, "-n", &n.to_string(), "--no-pager"],
    )
    .await
}

pub async fn run_cmd(bin: &str, args: &[&str]) -> Result<Vec<String>> {
    let out = Command::new(bin).args(args).output().await?;
    let text = String::from_utf8_lossy(&out.stdout).to_string()
        + String::from_utf8_lossy(&out.stderr).as_ref();
    Ok(text.lines().map(|l| l.to_string()).collect())
}

pub async fn run_cmd_string(bin: &str, args: &[&str]) -> Result<String> {
    Ok(run_cmd(bin, args).await?.join("\n"))
}
