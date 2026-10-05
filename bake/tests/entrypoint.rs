// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use std::process::Command;

#[test]
fn bake_binary_starts_and_prints_task_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_bake-releases-project"))
        .arg("--help")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("releases:update"));
}
