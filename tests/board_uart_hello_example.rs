use std::fs;
use std::process::Command;

#[test]
fn board_uart_hello_example_accepts_custom_board_path() {
    let board_path = std::env::temp_dir().join(format!(
        "sw-rv32i-board-uart-hello-custom-{}.toml",
        std::process::id()
    ));
    fs::write(
        &board_path,
        r#"
id = "custom-uart-board"
family = "custom"
arch = "rv32e"

[memory]
ram_base = "0x00000000"
ram_size = "0x00001000"

[gpio]
pins = [1]

[aliases]
led = 1

[mmio.uart0]
kind = "generic-uart"
base = "0x40013800"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "--example", "board_uart_hello", "--"])
        .arg(&board_path)
        .output()
        .unwrap();
    fs::remove_file(&board_path).unwrap();

    assert!(
        output.status.success(),
        "board_uart_hello failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("== Board: custom-uart-board =="));
    assert!(stdout.contains("UART Output:\nhello\n"));
}
