//! Neural API routing compliance validator.
//!
//! Replaces the archived `convergence_check.py` (jelly). Audits a primal for
//! proper Neural API integration:
//!
//! - `primal.announce` method in dispatch
//! - Announce payload includes required fields (atomic routing matrix)
//! - Capability registration with songBird (`ipc.register`)
//! - Signal tier declarations for composition routing
//! - Cost/latency hints for Neural API decision layer
//!
//! Wire format reference: `specs/NEURAL_API_ATOMIC_ROUTING_SPEC.md`

use anyhow::Result;
use std::path::Path;

/// Required announce payload fields per the atomic routing spec.
const REQUIRED_ANNOUNCE_FIELDS: &[&str] = &[
    "primal",
    "socket",
    "pid",
    "capabilities",
    "methods",
    "version",
];

/// Enhanced announce fields (Wave 42+ atomic routing).
const ENHANCED_ANNOUNCE_FIELDS: &[&str] = &[
    "signal_tiers",
    "cost_hints",
    "latency_estimates",
];

/// Methods a primal should advertise to be Neural API routable.
const ROUTING_METHODS: &[&str] = &[
    "primal.announce",
    "capabilities.list",
    "health.liveness",
    "health.readiness",
];

struct Finding {
    category: &'static str,
    message: String,
    severity: Severity,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Severity {
    Pass,
    Warning,
    Fail,
}

#[expect(clippy::unnecessary_wraps, reason = "signature required by dispatch")]
pub(super) fn validate(path: &Path, json: bool) -> Result<()> {
    let source_dir = super::find_source_dir(path);
    let Some(source_dir) = source_dir else {
        if json {
            println!(r#"{{"error":"no source directory found"}}"#);
        } else {
            crate::error("No source directory found");
        }
        return Ok(());
    };

    let files = super::collect_rs_files(&source_dir);
    let findings = audit_neural_api(path, &files);

    if json {
        print_json(&findings);
    } else {
        print_human(&findings);
    }

    Ok(())
}

/// Collected evidence from source scan.
#[expect(clippy::struct_excessive_bools, reason = "scan state accumulator")]
#[derive(Default)]
struct ScanEvidence {
    has_announce_dispatch: bool,
    has_announce_call: bool,
    has_capabilities_fn: bool,
    capabilities_non_empty: bool,
    has_signal_tiers: bool,
    has_cost_hints: bool,
    has_latency_estimates: bool,
    has_ipc_register: bool,
    has_dispatch_module: bool,
    dispatch_methods: Vec<String>,
    announce_fields_present: Vec<&'static str>,
}

fn audit_neural_api(base_path: &Path, files: &[std::path::PathBuf]) -> Vec<Finding> {
    let evidence = scan_files(base_path, files);
    evaluate_evidence(&evidence)
}

fn scan_files(base_path: &Path, files: &[std::path::PathBuf]) -> ScanEvidence {
    let mut ev = ScanEvidence::default();

    for file in files {
        let rel = file.strip_prefix(base_path).unwrap_or(file);
        let rel_str = rel.to_string_lossy();

        if rel_str.contains("/tests/") || rel_str.ends_with("_test.rs") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(file) else {
            continue;
        };

        if rel_str.contains("dispatch") {
            scan_dispatch(&content, &mut ev);
        }

        if rel_str.contains("announce") {
            scan_announce(&content, &mut ev);
        }
    }

    ev
}

fn scan_dispatch(content: &str, ev: &mut ScanEvidence) {
    ev.has_dispatch_module = true;
    if content.contains("\"primal.announce\"") {
        ev.has_announce_dispatch = true;
    }
    for method in ROUTING_METHODS {
        if content.contains(&format!("\"{method}\"")) {
            ev.dispatch_methods.push((*method).to_owned());
        }
    }
}

fn scan_announce(content: &str, ev: &mut ScanEvidence) {
    if content.contains("primal.announce") {
        ev.has_announce_call = true;
    }
    for field in REQUIRED_ANNOUNCE_FIELDS {
        if content.contains(&format!("\"{field}\"")) {
            ev.announce_fields_present.push(field);
        }
    }
    for field in ENHANCED_ANNOUNCE_FIELDS {
        if content.contains(&format!("\"{field}\""))
            || content.contains(&format!("\"{field}\":"))
        {
            match *field {
                "signal_tiers" => ev.has_signal_tiers = true,
                "cost_hints" => ev.has_cost_hints = true,
                "latency_estimates" => ev.has_latency_estimates = true,
                _ => {}
            }
        }
    }
    if content.contains("fn capabilities()") {
        ev.has_capabilities_fn = true;
        if !content.contains("&[]") {
            ev.capabilities_non_empty = true;
        }
    }
    if content.contains("ipc.register") {
        ev.has_ipc_register = true;
    }
}

fn evaluate_evidence(ev: &ScanEvidence) -> Vec<Finding> {
    let mut findings = Vec::new();

    evaluate_dispatch(ev, &mut findings);
    evaluate_announce(ev, &mut findings);
    evaluate_wire_format(ev, &mut findings);
    evaluate_songbird(ev, &mut findings);
    evaluate_capabilities(ev, &mut findings);
    evaluate_routing_hints(ev, &mut findings);
    evaluate_methods(ev, &mut findings);

    findings
}

fn evaluate_dispatch(ev: &ScanEvidence, findings: &mut Vec<Finding>) {
    if !ev.has_dispatch_module {
        findings.push(Finding {
            category: "dispatch",
            message: "no dispatch module found".to_owned(),
            severity: Severity::Fail,
        });
    } else if !ev.has_announce_dispatch {
        findings.push(Finding {
            category: "dispatch",
            message: "primal.announce not in dispatch METHODS".to_owned(),
            severity: Severity::Fail,
        });
    } else {
        findings.push(Finding {
            category: "dispatch",
            message: "primal.announce in dispatch".to_owned(),
            severity: Severity::Pass,
        });
    }
}

fn evaluate_announce(ev: &ScanEvidence, findings: &mut Vec<Finding>) {
    if ev.has_announce_call {
        findings.push(Finding {
            category: "announce",
            message: "primal.announce call present".to_owned(),
            severity: Severity::Pass,
        });
    } else {
        findings.push(Finding {
            category: "announce",
            message: "no primal.announce call found (primal won't self-register)".to_owned(),
            severity: Severity::Fail,
        });
    }
}

fn evaluate_wire_format(ev: &ScanEvidence, findings: &mut Vec<Finding>) {
    let missing_fields: Vec<&&str> = REQUIRED_ANNOUNCE_FIELDS
        .iter()
        .filter(|f| !ev.announce_fields_present.contains(f))
        .collect();

    if missing_fields.is_empty() {
        findings.push(Finding {
            category: "wire_format",
            message: "all required announce fields present".to_owned(),
            severity: Severity::Pass,
        });
    } else if ev.has_announce_call {
        findings.push(Finding {
            category: "wire_format",
            message: format!(
                "missing announce fields: {}",
                missing_fields
                    .iter()
                    .map(|f| format!("`{f}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            severity: Severity::Warning,
        });
    }
}

fn evaluate_songbird(ev: &ScanEvidence, findings: &mut Vec<Finding>) {
    if ev.has_ipc_register {
        findings.push(Finding {
            category: "songbird",
            message: "ipc.register with songBird present".to_owned(),
            severity: Severity::Pass,
        });
    } else {
        findings.push(Finding {
            category: "songbird",
            message: "no ipc.register — primal not discoverable via songBird".to_owned(),
            severity: Severity::Warning,
        });
    }
}

fn evaluate_capabilities(ev: &ScanEvidence, findings: &mut Vec<Finding>) {
    if ev.has_capabilities_fn {
        if ev.capabilities_non_empty {
            findings.push(Finding {
                category: "capabilities",
                message: "capabilities() returns domain list".to_owned(),
                severity: Severity::Pass,
            });
        } else {
            findings.push(Finding {
                category: "capabilities",
                message: "capabilities() returns empty — Neural API cannot route to this primal"
                    .to_owned(),
                severity: Severity::Warning,
            });
        }
    } else {
        findings.push(Finding {
            category: "capabilities",
            message: "no capabilities() function found".to_owned(),
            severity: Severity::Fail,
        });
    }
}

fn evaluate_routing_hints(ev: &ScanEvidence, findings: &mut Vec<Finding>) {
    if !ev.has_announce_call {
        return;
    }

    if ev.has_signal_tiers {
        findings.push(Finding {
            category: "routing",
            message: "signal_tiers declared".to_owned(),
            severity: Severity::Pass,
        });
    } else {
        findings.push(Finding {
            category: "routing",
            message: "no signal_tiers — Neural API cannot assign composition tier".to_owned(),
            severity: Severity::Warning,
        });
    }

    if ev.has_cost_hints && ev.has_latency_estimates {
        findings.push(Finding {
            category: "routing",
            message: "cost + latency hints present (full atomic routing)".to_owned(),
            severity: Severity::Pass,
        });
    } else {
        let mut missing = Vec::new();
        if !ev.has_cost_hints {
            missing.push("cost_hints");
        }
        if !ev.has_latency_estimates {
            missing.push("latency_estimates");
        }
        findings.push(Finding {
            category: "routing",
            message: format!(
                "missing routing hints: {} (Neural API defaults to equal-weight)",
                missing.join(", ")
            ),
            severity: Severity::Warning,
        });
    }
}

fn evaluate_methods(ev: &ScanEvidence, findings: &mut Vec<Finding>) {
    if !ev.has_dispatch_module {
        return;
    }

    let missing_methods: Vec<&&str> = ROUTING_METHODS
        .iter()
        .filter(|m| !ev.dispatch_methods.contains(&(**m).to_owned()))
        .collect();

    if missing_methods.is_empty() {
        findings.push(Finding {
            category: "methods",
            message: "all routing methods advertised".to_owned(),
            severity: Severity::Pass,
        });
    } else {
        findings.push(Finding {
            category: "methods",
            message: format!(
                "missing routing methods: {}",
                missing_methods
                    .iter()
                    .map(|m| format!("`{m}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            severity: Severity::Warning,
        });
    }
}

fn compliance_level(findings: &[Finding]) -> &'static str {
    let fails = findings.iter().filter(|f| f.severity == Severity::Fail).count();
    let warns = findings.iter().filter(|f| f.severity == Severity::Warning).count();
    let passes = findings.iter().filter(|f| f.severity == Severity::Pass).count();

    if fails == 0 && warns == 0 {
        "FULL"
    } else if fails == 0 && passes > 0 {
        "ROUTABLE"
    } else if findings.iter().any(|f| f.category == "announce" && f.severity == Severity::Pass) {
        "PARTIAL"
    } else {
        "NONE"
    }
}

fn print_json(findings: &[Finding]) {
    let level = compliance_level(findings);
    let passes: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Pass).collect();
    let warns: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Warning).collect();
    let fails: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Fail).collect();

    println!("{{");
    println!("  \"compliance\": \"{level}\",");
    println!("  \"pass\": [");
    for (i, f) in passes.iter().enumerate() {
        let comma = if i + 1 < passes.len() { "," } else { "" };
        println!(
            "    {{\"category\": \"{}\", \"message\": \"{}\"}}{}",
            f.category, f.message, comma
        );
    }
    println!("  ],");
    println!("  \"warnings\": [");
    for (i, f) in warns.iter().enumerate() {
        let comma = if i + 1 < warns.len() { "," } else { "" };
        println!(
            "    {{\"category\": \"{}\", \"message\": \"{}\"}}{}",
            f.category, f.message, comma
        );
    }
    println!("  ],");
    println!("  \"failures\": [");
    for (i, f) in fails.iter().enumerate() {
        let comma = if i + 1 < fails.len() { "," } else { "" };
        println!(
            "    {{\"category\": \"{}\", \"message\": \"{}\"}}{}",
            f.category, f.message, comma
        );
    }
    println!("  ]");
    println!("}}");
}

fn print_human(findings: &[Finding]) {
    let level = compliance_level(findings);

    println!();
    crate::info(&format!("Neural API compliance: {level}"));
    println!();

    for f in findings {
        match f.severity {
            Severity::Pass => crate::success(&format!("  [{}] {}", f.category, f.message)),
            Severity::Warning => {
                crate::warning(&format!("  [{}] {}", f.category, f.message));
            }
            Severity::Fail => crate::error(&format!("  [{}] {}", f.category, f.message)),
        }
    }

    println!();
    match level {
        "FULL" => crate::success("Fully routable via Neural API atomic matrix"),
        "ROUTABLE" => crate::success("Routable (some hints missing — defaults apply)"),
        "PARTIAL" => {
            crate::warning("Partially compliant — announce works but gaps remain");
        }
        _ => crate::error("Not Neural API compliant — primal cannot be routed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scaffold_primal(dir: &Path) {
        let core_src = dir.join("crates/test-core/src");
        let server_src = dir.join("crates/test-server/src");
        fs::create_dir_all(&core_src).unwrap();
        fs::create_dir_all(&server_src).unwrap();

        fs::write(
            server_src.join("dispatch.rs"),
            r#"
pub const METHODS: &[&str] = &[
    "primal.announce",
    "capabilities.list",
    "health.liveness",
    "health.readiness",
];
"#,
        )
        .unwrap();

        fs::write(
            server_src.join("announce.rs"),
            r#"
pub async fn announce_to_biomeos() {
    let payload = serde_json::json!({
        "method": "primal.announce",
        "params": {
            "primal": "test",
            "socket": "/tmp/test.sock",
            "pid": 1234,
            "capabilities": capabilities(),
            "methods": ["primal.announce"],
            "signal_tiers": signal_tiers(),
            "cost_hints": cost_hints(),
            "latency_estimates": latency_estimates(),
            "version": "0.1.0",
        },
    });
    // ipc.register
}

pub fn capabilities() -> &'static [&'static str] {
    &["crypto"]
}

fn signal_tiers() -> &'static [&'static str] { &["tower"] }
fn cost_hints() -> serde_json::Value { serde_json::json!({"crypto": 5}) }
fn latency_estimates() -> serde_json::Value { serde_json::json!({"crypto": 2}) }
"#,
        )
        .unwrap();
    }

    #[test]
    fn full_compliance_detected() {
        let tmp = tempfile::tempdir().unwrap();
        scaffold_primal(tmp.path());

        let files = super::super::collect_rs_files(&tmp.path().join("crates"));
        let findings = audit_neural_api(tmp.path(), &files);
        let level = compliance_level(&findings);

        assert_eq!(level, "FULL");
    }

    #[test]
    fn missing_announce_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("crates/test-core/src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), "pub fn hello() {}").unwrap();

        let files = super::super::collect_rs_files(&tmp.path().join("crates"));
        let findings = audit_neural_api(tmp.path(), &files);
        let level = compliance_level(&findings);

        assert_eq!(level, "NONE");
    }

    #[test]
    fn empty_capabilities_is_routable() {
        let tmp = tempfile::tempdir().unwrap();
        let server_src = tmp.path().join("crates/test-server/src");
        fs::create_dir_all(&server_src).unwrap();

        fs::write(
            server_src.join("dispatch.rs"),
            r#"pub const METHODS: &[&str] = &["primal.announce", "capabilities.list", "health.liveness", "health.readiness"];"#,
        )
        .unwrap();

        fs::write(
            server_src.join("announce.rs"),
            r#"
pub async fn f() {
    let _ = serde_json::json!({
        "method": "primal.announce",
        "params": {
            "primal": "x",
            "socket": "/s",
            "pid": 1,
            "capabilities": capabilities(),
            "methods": [],
            "signal_tiers": [],
            "cost_hints": {},
            "latency_estimates": {},
            "version": "0.1.0",
        }
    });
    // ipc.register
}
pub fn capabilities() -> &'static [&'static str] { &[] }
"#,
        )
        .unwrap();

        let files = super::super::collect_rs_files(&tmp.path().join("crates"));
        let findings = audit_neural_api(tmp.path(), &files);
        let level = compliance_level(&findings);

        assert_eq!(level, "ROUTABLE");
    }

    #[test]
    fn compliance_levels_are_ordered() {
        let none_findings = vec![Finding {
            category: "dispatch",
            message: "fail".to_owned(),
            severity: Severity::Fail,
        }];
        assert_eq!(compliance_level(&none_findings), "NONE");

        let full_findings = vec![Finding {
            category: "dispatch",
            message: "pass".to_owned(),
            severity: Severity::Pass,
        }];
        assert_eq!(compliance_level(&full_findings), "FULL");
    }
}
