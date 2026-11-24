use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    process::{Command, ExitCode},
    time::Instant,
};

fn main() -> ExitCode {
    let cap = std::env::args().nth(1).unwrap_or_else(|| "verify".into());
    let profile = std::fs::read("generation-report.json")
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .unwrap_or_default();
    let native = profile["targets"]
        .as_array()
        .is_none_or(|v| v.iter().any(|t| t == "native"));
    let wasm = profile["targets"]
        .as_array()
        .is_some_and(|v| v.iter().any(|t| t == "wasm"));
    let mut checks = Vec::new();
    match cap.as_str() {
        "doctor" => {
            checks.push(run("cargo", &["--version"], "host"));
            checks.push(run("rustc", &["-vV"], "host"));
        }
        "layers" => checks.push(layers("native")),
        "fast" | "verify" | "ci" => {
            checks.push(run("cargo", &["fmt", "--all", "--", "--check"], "host"));
            if native {
                checks.push(layers("native"));
                checks.push(run(
                    "cargo",
                    &["check", "--workspace", "--all-targets"],
                    "native",
                ));
                if cap != "fast" {
                    checks.push(run("cargo", &["test", "--workspace"], "native"));
                    checks.push(run(
                        "cargo",
                        &[
                            "clippy",
                            "--workspace",
                            "--all-targets",
                            "--",
                            "-D",
                            "warnings",
                        ],
                        "native",
                    ));
                }
            }
            if wasm {
                checks.push(wasm_check(&profile));
            }
        }
        "test" if native => checks.push(run("cargo", &["test", "--workspace"], "native")),
        "wasm" if wasm => checks.push(wasm_check(&profile)),
        "test" | "wasm" => checks.push(record(
            &cap,
            "profile",
            "not_applicable",
            0,
            "target absent",
        )),
        "docs" => checks.push(run("cargo", &["doc", "--workspace", "--no-deps"], "host")),
        _ => {
            eprintln!("cargo xtask <doctor|layers|fast|verify|ci|test|wasm|docs>");
            return ExitCode::FAILURE;
        }
    }
    let evidence = json!({"toolchain":host().unwrap_or_default(),"checks":checks});
    let dir = "target/starter-evidence";
    if let Err(error) = std::fs::create_dir_all(dir).and_then(|()| {
        std::fs::write(
            format!("{dir}/{cap}.json"),
            serde_json::to_vec_pretty(&evidence)?,
        )
    }) {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    if checks
        .iter()
        .any(|v| matches!(v["status"].as_str(), Some("blocked" | "failed")))
    {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
fn wasm_check(profile: &Value) -> Value {
    let mut args = vec!["check", "--target", "wasm32-unknown-unknown"];
    if let Some(package) = profile["wasm_package"].as_str() {
        args.extend(["-p", package]);
    } else {
        args.extend(["--workspace", "--exclude", "xtask"]);
    }
    run("cargo", &args, "wasm32-unknown-unknown")
}
fn record(command: &str, target: &str, status: &str, duration: u128, message: &str) -> Value {
    json!({"command":command,"target":target,"status":status,"duration_ms":duration,"message":message,"artifacts":[],"warnings":[]})
}
fn run(tool: &str, args: &[&str], target: &str) -> Value {
    let start = Instant::now();
    let cmd = format!("{tool} {}", args.join(" "));
    eprintln!("$ {cmd}");
    let (status, reason) = match Command::new(tool).args(args).status() {
        Ok(s) if s.success() => ("passed", String::new()),
        Ok(s) => ("failed", s.to_string()),
        Err(e) => ("blocked", e.to_string()),
    };
    record(&cmd, target, status, start.elapsed().as_millis(), &reason)
}
fn host() -> Option<String> {
    let out = Command::new("rustc").arg("-vV").output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}
fn layers(target: &str) -> Value {
    let start = Instant::now();
    let mut args = vec!["metadata", "--format-version", "1", "--all-features"];
    let info = host();
    if let Some(triple) = info
        .as_deref()
        .and_then(|text| text.lines().find_map(|line| line.strip_prefix("host: ")))
    {
        args.extend(["--filter-platform", triple]);
    }
    let cmd = format!("cargo {}", args.join(" "));
    let (status, reason) = match Command::new("cargo").args(args).output() {
        Ok(out) if out.status.success() => match serde_json::from_slice(&out.stdout)
            .map_err(|e| e.to_string())
            .and_then(|value| audit(&value))
        {
            Ok(()) => ("passed", String::new()),
            Err(e) => ("failed", e),
        },
        Ok(out) => ("failed", String::from_utf8_lossy(&out.stderr).into_owned()),
        Err(e) => ("blocked", e.to_string()),
    };
    eprintln!("{status} {reason}");
    record(&cmd, target, status, start.elapsed().as_millis(), &reason)
}
fn audit(graph: &Value) -> Result<(), String> {
    let packages = graph["packages"]
        .as_array()
        .ok_or("metadata has no packages")?;
    let nodes = graph["resolve"]["nodes"]
        .as_array()
        .ok_or("metadata has no resolution")?;
    let names: BTreeMap<_, _> = packages
        .iter()
        .filter_map(|p| Some((p["id"].as_str()?, p["name"].as_str()?)))
        .collect();
    let edges: BTreeMap<_, _> = nodes
        .iter()
        .filter_map(|n| Some((n["id"].as_str()?, n["deps"].as_array()?)))
        .collect();
    for name in ["wgpu", "egui", "eframe"] {
        let versions: BTreeSet<_> = packages
            .iter()
            .filter(|p| {
                p["name"] == name && p["id"].as_str().is_some_and(|id| edges.contains_key(id))
            })
            .filter_map(|p| p["version"].as_str())
            .collect();
        if versions.len() > 1 {
            return Err(format!("incompatible {name} versions: {versions:?}"));
        }
    }
    for p in packages {
        let name = p["name"].as_str().unwrap_or_default();
        if !matches!(
            name,
            "starter-core" | "starter-documents" | "starter-science" | "starter-evaluation"
        ) && !matches!(
            p["metadata"]["starter"]["layer"].as_str(),
            Some("domain" | "application")
        ) {
            continue;
        }
        let mut pending = vec![(p["id"].as_str().ok_or("package has no ID")?, vec![name])];
        let mut seen = BTreeSet::new();
        while let Some((parent, path)) = pending.pop() {
            if !seen.insert(parent) {
                continue;
            }
            for e in edges.get(parent).into_iter().flat_map(|d| d.iter()) {
                let id = e["pkg"].as_str().ok_or("dependency has no package")?;
                let name = names.get(id).copied().ok_or("dependency has no name")?;
                let mut route = path.clone();
                route.push(name);
                if matches!(
                    name,
                    "egui"
                        | "eframe"
                        | "wgpu"
                        | "rmcp"
                        | "axum"
                        | "leptos"
                        | "dioxus"
                        | "starter-desktop"
                ) {
                    return Err(format!("forbidden dependency: {}", route.join(" -> ")));
                }
                pending.push((id, route));
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_a_transitive_violation() {
        let d = json!({"packages":[{"id":"a","name":"starter-core"},{"id":"b","name":"bridge"},{"id":"c","name":"egui","version":"0.37"}],
            "resolve":{"nodes":[{"id":"a","deps":[{"pkg":"b"}]},{"id":"b","deps":[{"pkg":"c"}]},{"id":"c","deps":[]}]}});
        assert_eq!(
            audit(&d).unwrap_err(),
            "forbidden dependency: starter-core -> bridge -> egui"
        );
    }
}
