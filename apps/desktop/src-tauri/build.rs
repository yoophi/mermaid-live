use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../package.json");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    println!("cargo:rerun-if-env-changed=COMMIT_SHA");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_TYPE");

    println!(
        "cargo:rustc-env=MERMAID_LIVE_PACKAGE_VERSION={}",
        package_json_version()
    );
    println!(
        "cargo:rustc-env=MERMAID_LIVE_BUILD_VERSION={}",
        build_version()
    );
    println!(
        "cargo:rustc-env=MERMAID_LIVE_GIT_COMMIT_HASH={}",
        build_commit_hash()
    );
    println!(
        "cargo:rustc-env=MERMAID_LIVE_GIT_COMMIT_TAG={}",
        build_commit_tag()
    );

    tauri_build::build();
}

fn package_json_version() -> String {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let package_json_path = manifest_dir.join("../package.json");

    let Ok(package_json) = fs::read_to_string(package_json_path) else {
        return env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".into());
    };

    serde_json::from_str::<serde_json::Value>(&package_json)
        .ok()
        .and_then(|value| {
            value
                .get("version")
                .and_then(|version| version.as_str())
                .filter(|version| !version.trim().is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".into()))
}

fn build_commit_hash() -> String {
    env::var("GITHUB_SHA")
        .or_else(|_| env::var("COMMIT_SHA"))
        .ok()
        .map(|hash| hash.trim().chars().take(7).collect::<String>())
        .filter(|hash| !hash.is_empty())
        .unwrap_or_else(git_commit_hash)
}

fn git_commit_hash() -> String {
    git_output(&["rev-parse", "--short=7", "HEAD"]).unwrap_or_else(|| "unknown".into())
}

fn build_version() -> String {
    let base_version = package_json_version();
    let commit_hash = build_commit_hash();

    if commit_hash == "unknown" {
        return format!("{base_version}-unknown");
    }

    let dirty = git_output(&["status", "--porcelain"])
        .map(|status| !status.is_empty())
        .unwrap_or(false);
    let release_tag = build_commit_tag()
        .split(", ")
        .any(|tag| tag == base_version);

    if release_tag && !dirty {
        base_version
    } else if dirty {
        format!("{base_version}-{commit_hash}-dirty")
    } else {
        format!("{base_version}-{commit_hash}")
    }
}

fn build_commit_tag() -> String {
    github_tag_ref()
        .or_else(|| git_output(&["tag", "--points-at", "HEAD"]))
        .unwrap_or_else(|| "unknown".into())
}

fn github_tag_ref() -> Option<String> {
    let ref_type = env::var("GITHUB_REF_TYPE").ok()?;
    if ref_type.trim() != "tag" {
        return None;
    }

    env::var("GITHUB_REF_NAME")
        .ok()
        .map(|tag| tag.trim().to_owned())
        .filter(|tag| !tag.is_empty())
}

fn git_output(args: &[&str]) -> Option<String> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let repository_root = manifest_dir.join("../../..");

    Command::new("git")
        .args(args)
        .current_dir(repository_root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| {
            value
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.is_empty())
}
