#![cfg(feature = "auto-ui-tests")]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};

#[derive(Debug)]
struct WindowGeometry {
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct CropMetric {
    stddev: f64,
    colors: f64,
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn auto_ui_available() -> bool {
    env::var_os("DISPLAY").is_some()
        && ["xdotool", "wmctrl", "import", "convert", "identify"]
            .iter()
            .all(|tool| command_exists(tool))
}

fn run_command(program: &str, args: &[&str]) -> Result<Output> {
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("failed to run {program}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        bail!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

fn command_stdout(program: &str, args: &[&str]) -> Result<String> {
    let output = run_command(program, args)?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn get_active_window_id() -> Result<Option<String>> {
    let output = Command::new("xdotool")
        .args(["getactivewindow"])
        .output()
        .with_context(|| "failed to query active window".to_string())?;
    if output.status.success() {
        let id = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if id.is_empty() {
            Ok(None)
        } else {
            Ok(Some(id))
        }
    } else {
        Ok(None)
    }
}

fn find_window_id_for_pid(pid: i32, timeout: Duration) -> Result<Option<String>> {
    find_window_id_with_args(&["search", "--all", "--pid"], &pid.to_string(), timeout)
}

fn find_window_id(title: &str, timeout: Duration) -> Result<Option<String>> {
    find_window_id_with_args(&["search", "--name"], title, timeout)
}

fn find_window_id_with_args(
    prefix_args: &[&str],
    needle: &str,
    timeout: Duration,
) -> Result<Option<String>> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let output = Command::new("xdotool")
            .args(prefix_args)
            .arg(needle)
            .output()
            .with_context(|| "failed to search for X11 window".to_string())?;
        if output.status.success() {
            let id = String::from_utf8_lossy(&output.stdout)
                .lines()
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_owned());
            if id.is_some() {
                return Ok(id);
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    Ok(None)
}

fn prepare_window_for_capture(
    window_id: &str,
    width: u32,
    height: u32,
    previous_active_window: Option<&str>,
) -> Result<WindowGeometry> {
    let _ = previous_active_window;
    run_command(
        "wmctrl",
        &[
            "-i",
            "-r",
            window_id,
            "-b",
            "remove,maximized_vert,maximized_horz",
        ],
    )?;
    run_command("xdotool", &["windowmove", window_id, "0", "0"])?;
    run_command(
        "xdotool",
        &[
            "windowsize",
            "--sync",
            window_id,
            &width.to_string(),
            &height.to_string(),
        ],
    )?;
    run_command("xdotool", &["windowactivate", "--sync", window_id])?;
    let geometry = command_stdout("xdotool", &["getwindowgeometry", "--shell", window_id])?;
    parse_window_geometry(&geometry)
}

fn parse_window_geometry(geometry: &str) -> Result<WindowGeometry> {
    let mut width = None;
    let mut height = None;
    for line in geometry.lines() {
        if let Some(value) = line.strip_prefix("WIDTH=") {
            width = Some(value.parse::<u32>().context("invalid WIDTH from xdotool")?);
        }
        if let Some(value) = line.strip_prefix("HEIGHT=") {
            height = Some(
                value
                    .parse::<u32>()
                    .context("invalid HEIGHT from xdotool")?,
            );
        }
    }

    Ok(WindowGeometry {
        width: width.ok_or_else(|| anyhow!("xdotool geometry missing WIDTH"))?,
        height: height.ok_or_else(|| anyhow!("xdotool geometry missing HEIGHT"))?,
    })
}

fn capture_window_screenshot(window_id: &str, screenshot_path: &Path) -> Result<()> {
    let target = screenshot_path
        .to_str()
        .ok_or_else(|| anyhow!("screenshot path is not valid UTF-8"))?;
    run_command("import", &["-window", window_id, target])?;
    Ok(())
}

fn image_size(path: &Path) -> Result<(u32, u32)> {
    let target = path
        .to_str()
        .ok_or_else(|| anyhow!("image path is not valid UTF-8"))?;
    let output = command_stdout("identify", &["-format", "%w %h", target])?;
    let mut parts = output.split_whitespace();
    let width = parts
        .next()
        .ok_or_else(|| anyhow!("identify missing width"))?
        .parse::<u32>()
        .context("invalid width from identify")?;
    let height = parts
        .next()
        .ok_or_else(|| anyhow!("identify missing height"))?
        .parse::<u32>()
        .context("invalid height from identify")?;
    Ok((width, height))
}

fn crop_metric(path: &Path, x: u32, y: u32, width: u32, height: u32) -> Result<CropMetric> {
    let target = path
        .to_str()
        .ok_or_else(|| anyhow!("image path is not valid UTF-8"))?;
    let cropped = temp_png_path("tmc-egui-crop");
    let cropped_target = cropped
        .to_str()
        .ok_or_else(|| anyhow!("crop path is not valid UTF-8"))?;
    remove_file_if_exists(&cropped);

    run_command(
        "convert",
        &[
            target,
            "-crop",
            &format!("{width}x{height}+{x}+{y}"),
            "+repage",
            cropped_target,
        ],
    )?;

    let output = command_stdout(
        "identify",
        &[
            "-format",
            "%[standard-deviation] %[colors] %[fx:quantumrange]",
            cropped_target,
        ],
    )?;
    remove_file_if_exists(&cropped);

    let mut parts = output.split_whitespace();
    let stddev = parts
        .next()
        .ok_or_else(|| anyhow!("identify missing standard deviation"))?
        .parse::<f64>()
        .context("invalid standard deviation from identify")?;
    let colors = parts
        .next()
        .ok_or_else(|| anyhow!("identify missing color count"))?
        .parse::<f64>()
        .context("invalid color count from identify")?;
    let quantum_range = parts
        .next()
        .ok_or_else(|| anyhow!("identify missing quantum range"))?
        .parse::<f64>()
        .context("invalid quantum range from identify")?;

    Ok(CropMetric {
        stddev: stddev / quantum_range.max(1.0),
        colors,
    })
}

fn heuristic_text_visible(metric: &CropMetric) -> bool {
    metric.stddev > 0.0 && metric.colors > 1.0
}

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn id(&self) -> i32 {
        self.child.id() as i32
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn unique_name(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos}")
}

fn temp_png_path(prefix: &str) -> PathBuf {
    env::temp_dir().join(format!("{}-{}.png", prefix, unique_name("capture")))
}

fn remove_file_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn capture_harness_summary(
    title_prefix: &str,
    markdown: &str,
    width: u32,
    height: u32,
) -> Result<String> {
    let previous_active_window = get_active_window_id()?;
    let title = unique_name(title_prefix);
    let screenshot_path = temp_png_path(&title);
    remove_file_if_exists(&screenshot_path);

    let child = Command::new(env!("CARGO_BIN_EXE_tmc_egui_auto_ui"))
        .env("TMC_EGUI_WINDOW_TITLE", &title)
        .env("TMC_EGUI_MARKDOWN", markdown)
        .env("TMC_EGUI_WINDOW_WIDTH", width.to_string())
        .env("TMC_EGUI_WINDOW_HEIGHT", height.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| "failed to start egui auto-ui harness".to_string())?;
    let child = ChildGuard { child };

    let window_id = find_window_id_for_pid(child.id(), Duration::from_secs(10))?
        .or_else(|| {
            find_window_id(&title, Duration::from_secs(2))
                .ok()
                .flatten()
        })
        .ok_or_else(|| anyhow!("failed to locate window for {title}"))?;

    let geometry =
        prepare_window_for_capture(&window_id, width, height, previous_active_window.as_deref())?;
    thread::sleep(Duration::from_millis(1200));

    capture_window_screenshot(&window_id, &screenshot_path)?;
    let (image_width, image_height) = image_size(&screenshot_path)?;
    let metric = crop_metric(
        &screenshot_path,
        image_width / 6,
        image_height / 6,
        (image_width * 2 / 3).max(1),
        (image_height * 2 / 3).max(1),
    )?;

    assert!(image_width > 0);
    assert!(image_height > 0);
    assert!(heuristic_text_visible(&metric));

    let summary = format!(
        "window_width={window_width}\nwindow_height={window_height}\nimage_width={image_width}\nimage_height={image_height}\nfocus_stddev={stddev:.4}\nfocus_colors={colors:.1}\n",
        window_width = geometry.width,
        window_height = geometry.height,
        stddev = metric.stddev,
        colors = metric.colors,
    );

    remove_file_if_exists(&screenshot_path);
    drop(child);

    Ok(summary)
}

#[test]
fn auto_ui_captures_egui_harness_window() -> Result<()> {
    if !auto_ui_available() {
        eprintln!("Skipping auto-ui egui test because X11 capture tools are unavailable.");
        return Ok(());
    }

    let markdown = "# Auto UI Screenshot\n\nParagraph with **bold** text.\n\n| Name | Value |\n| --- | --- |\n| Parser | ready |\n| Renderer | visible |\n\n```json\n{\"capture\":true,\"mode\":\"x11\"}\n```\n\n> blockquote";
    let summary = capture_harness_summary("tmc-egui-auto-ui", markdown, 560, 760)?;

    insta::assert_snapshot!("egui_auto_ui_capture", summary);

    Ok(())
}

#[test]
fn auto_ui_captures_json_rendering() -> Result<()> {
    if !auto_ui_available() {
        eprintln!("Skipping auto-ui egui json test because X11 capture tools are unavailable.");
        return Ok(());
    }

    let markdown = "# JSON Render\n\n```json\n{\n  \"name\": \"tmc\",\n  \"features\": [\"egui\", \"auto-ui\"],\n  \"nested\": {\n    \"enabled\": true,\n    \"count\": 3,\n    \"items\": [\n      {\"id\": 1, \"label\": \"alpha\"},\n      {\"id\": 2, \"label\": \"beta\"}\n    ]\n  }\n}\n```";
    let summary = capture_harness_summary("tmc-egui-auto-ui-json", markdown, 640, 760)?;

    insta::assert_snapshot!("egui_auto_ui_json_capture", summary);

    Ok(())
}
