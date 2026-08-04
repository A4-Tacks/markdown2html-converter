use std::{
    env, fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

const EXECUTABLE: &str = env!("CARGO_BIN_EXE_markdown2html-converter");

fn temp_dir(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("markdown2html-converter-{name}"));

    let _ = fs::remove_dir_all(path.as_path());
    fs::create_dir_all(path.as_path()).unwrap();

    path
}

#[test]
fn markdown_from_stdin_goes_to_stdout() {
    let mut child = Command::new(EXECUTABLE)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(b"# Hello world!\n").unwrap();

    let output = child.wait_with_output().unwrap();
    let html = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(html.contains("<title>Hello world!</title>"));
}

#[test]
fn the_html_file_is_written_next_to_the_markdown_file() {
    let dir = temp_dir("next-to");
    let markdown_path = dir.join("readme.md");

    fs::write(markdown_path.as_path(), "Just a paragraph.").unwrap();

    let status = Command::new(EXECUTABLE).arg(markdown_path.as_path()).status().unwrap();

    let html = fs::read_to_string(dir.join("readme.html")).unwrap();

    assert!(status.success());
    assert!(html.contains("<title>readme</title>"));
}

#[test]
fn an_existing_html_file_is_only_overwritten_with_force() {
    let dir = temp_dir("force");
    let markdown_path = dir.join("readme.md");
    let html_path = dir.join("readme.html");

    fs::write(markdown_path.as_path(), "Just a paragraph.").unwrap();
    fs::write(html_path.as_path(), "old").unwrap();

    let status = Command::new(EXECUTABLE)
        .arg(markdown_path.as_path())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    assert!(!status.success());
    assert_eq!("old", fs::read_to_string(html_path.as_path()).unwrap());

    let status = Command::new(EXECUTABLE).arg(markdown_path.as_path()).arg("-f").status().unwrap();

    assert!(status.success());
    assert!(fs::read_to_string(html_path.as_path()).unwrap().contains("<!DOCTYPE html>"));
}

#[test]
fn local_images_can_be_embedded() {
    let dir = temp_dir("embed-images");
    let markdown_path = dir.join("readme.md");

    // The bytes only have to be readable, not a valid PNG.
    fs::write(dir.join("pic.png"), b"an image").unwrap();
    fs::write(markdown_path.as_path(), "![a picture](pic.png)").unwrap();

    let status = Command::new(EXECUTABLE)
        .arg(markdown_path.as_path())
        .arg("-o")
        .arg(dir.join("out.html"))
        .arg("--embed-images")
        .status()
        .unwrap();

    let html = fs::read_to_string(dir.join("out.html")).unwrap();

    assert!(status.success());
    assert!(html.contains("src=\"data:image/png;base64,"));
}
