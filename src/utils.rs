use std::process::Output;

pub fn log_cmd_output(cmd: &str, output: Output) {
    if !output.stdout.is_empty() {
        log::info!("{cmd}: {}", String::from_utf8_lossy(&output.stdout));
    } else if !output.stderr.is_empty() {
        log::error!("{cmd}: {}", String::from_utf8_lossy(&output.stderr));
    }
}
