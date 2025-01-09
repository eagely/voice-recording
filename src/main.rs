mod error;
mod recorder;
mod transcription_client;

use recorder::Recorder;
use std::fs::{remove_file, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;

const CONTROL_PIPE: &str = "/tmp/recorder_control";
const OUTPUT_PIPE: &str = "/tmp/recorder_output";

fn main() {
    remove_file(CONTROL_PIPE).ok();
    remove_file(OUTPUT_PIPE).ok();

    nix::unistd::mkfifo(CONTROL_PIPE, nix::sys::stat::Mode::S_IRWXU)
        .expect("Failed to create control pipe");
    nix::unistd::mkfifo(OUTPUT_PIPE, nix::sys::stat::Mode::S_IRWXU)
        .expect("Failed to create output pipe");

    let control = OpenOptions::new()
        .read(true)
        .custom_flags(nix::fcntl::OFlag::O_NONBLOCK.bits())
        .open(CONTROL_PIPE)
        .expect("Failed to open control pipe");

    let mut output = OpenOptions::new()
        .write(true)
        .open(OUTPUT_PIPE)
        .expect("Failed to open output pipe");

    let mut recorder = Recorder::new().expect("Failed to create recorder");
    let mut reader = BufReader::new(control);

    loop {
        let mut command = String::new();
        if reader.read_line(&mut command).unwrap_or(0) > 0 {
            match command.trim() {
                "start" => {
                    recorder.start().expect("Failed to start recording");
                    output.write_all(b"recording_started\n").unwrap();
                    output.flush().unwrap();
                }
                "stop" => {
                    let result = recorder.stop().expect("Failed to stop recording");
                    output.write_all(result.as_bytes()).unwrap();
                    output.write_all(b"\n").unwrap();
                    output.flush().unwrap();
                    recorder = Recorder::new().expect("Failed to create new recorder");
                }
                _ => {
                    output.write_all(b"unknown_command\n").unwrap();
                    output.flush().unwrap();
                }
            }
        }
    }
}
