//! GPU integration tests for triangle rendering
//!
//! These tests verify that the triangle example actually renders correctly.
//! They require a GPU and display to run.

use std::process::{Command, Stdio};
use std::time::Duration;

/// Test that the triangle example runs without crashing
#[test]
#[ignore] // Ignored by default - run with `cargo test --ignored` or `cargo test -- --ignored`
fn test_triangle_runs_without_crash() {
    // Build the example first
    let build_status = Command::new("cargo")
        .args(["build", "--example", "triangle", "--release"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to build triangle example");

    assert!(build_status.success(), "Triangle example failed to build");

    // Run the example with a timeout
    // We can't easily test visual output in CI, but we can verify it doesn't crash
    let child = Command::new("timeout")
        .args([
            "3", // Run for 3 seconds
            "target/release/examples/triangle",
        ])
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn triangle example");

    let output = child
        .wait_with_output()
        .expect("Failed to wait for triangle example");

    // timeout returns 124 when it times out (which is expected)
    // any other non-zero exit code indicates a crash
    let exit_code = output.status.code().unwrap_or(1);

    // Convert output to string for analysis
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("=== Triangle Example Output ===");
    println!("Exit code: {exit_code}");
    println!("\n=== STDOUT ===\n{stdout}");
    println!("\n=== STDERR ===\n{stderr}");

    // Check for success indicators in output
    let initialized =
        stdout.contains("backend initialized") || stdout.contains("Vulkan backend initialized");
    let no_panic = !stderr.contains("panic")
        && !stderr.contains("SIGSEGV")
        && !stderr.contains("segmentation fault");

    // 124 means timeout (expected), 0 means clean exit (also good)
    assert!(
        exit_code == 124 || exit_code == 0,
        "Triangle example crashed with exit code {exit_code} (124=timeout expected, 0=clean exit)"
    );

    assert!(
        initialized,
        "Triangle example did not initialize backend properly"
    );

    assert!(no_panic, "Triangle example panicked or segfaulted");
}

/// Test that triangle example accepts test-duration flag
#[test]
#[ignore]
fn test_triangle_with_duration_flag() {
    // This test verifies that we can control the example run duration
    // Useful for automated testing

    let start = std::time::Instant::now();

    let output = Command::new("target/release/examples/triangle")
        .args(["--test-duration", "2"])
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run triangle example");

    let elapsed = start.elapsed();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("=== Triangle Example with Duration Flag ===");
    println!("Exit code: {:?}", output.status.code());
    println!("Elapsed: {elapsed:?}");
    println!("\n=== STDOUT ===\n{stdout}");
    println!("\n=== STDERR ===\n{stderr}");

    // Should exit cleanly within ~2-3 seconds
    assert!(
        elapsed < Duration::from_secs(5),
        "Triangle should exit within 5 seconds with --test-duration 2"
    );

    // Should exit cleanly (0) not crash
    assert!(
        output.status.success(),
        "Triangle should exit cleanly with test-duration flag"
    );
}

/// Smoke test: verify backend initialization messages appear
#[test]
#[ignore]
fn test_triangle_backend_initialization() {
    let output = Command::new("timeout")
        .args(["2", "target/release/examples/triangle"])
        .env("RUST_LOG", "debug")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run triangle example");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    println!("=== Backend Initialization Check ===");
    println!("{combined}");

    // Check for key initialization steps
    let checks = vec![
        ("Creating Vulkan backend", "Backend creation started"),
        ("Vulkan instance created", "Instance created"),
        ("Selected device", "Device selected"),
        ("Logical device created", "Logical device created"),
        ("Swapchain created", "Swapchain created"),
        ("Pipeline created", "Pipeline created"),
    ];

    let mut passed = 0;
    for (pattern, description) in checks {
        if combined.contains(pattern) {
            println!("✓ {description}");
            passed += 1;
        } else {
            println!("✗ {description} (missing: '{pattern}')");
        }
    }

    assert!(
        passed >= 4,
        "Expected at least 4 initialization steps, found {passed}"
    );
}

#[cfg(test)]
mod visual_tests {
    //! Visual validation tests
    //!
    //! These would capture frames and compare to reference images.
    //! Currently placeholder - full implementation would require:
    //! - Screenshot capture capability
    //! - Reference images
    //! - Image comparison logic

    // TODO: Implement visual validation with image crate
    // TODO: Add reference image comparison
    // TODO: Add tolerance for platform differences
}
