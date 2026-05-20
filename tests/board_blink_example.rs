use std::fs;
use std::process::Command;

#[test]
fn board_blink_example_accepts_custom_board_path() {
    let board_path = std::env::temp_dir().join(format!(
        "sw-rv32i-board-blink-custom-{}.toml",
        std::process::id()
    ));
    fs::write(
        &board_path,
        r#"
id = "custom-gpio8-board"
family = "custom"
arch = "rv32i"

[memory]
ram_base = "0x00000000"
ram_size = "0x00001000"

[gpio]
pins = [8]

[aliases]
led = 8

[mmio.gpio]
kind = "generic-gpio"
base = "0x60004000"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "--example", "board_blink", "--"])
        .arg(&board_path)
        .output()
        .unwrap();
    fs::remove_file(&board_path).unwrap();

    assert!(
        output.status.success(),
        "board_blink failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("== Board: custom-gpio8-board =="));
    assert!(stdout.contains("0: gpio8 high"));
    assert!(stdout.contains("1: gpio8 low"));
}
