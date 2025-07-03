use anyhow::{anyhow, Context, Result};
use clap::ArgEnum;
use nix::{
    libc,
    sys::signal::{kill, killpg, Signal},
    unistd::Pid,
};
use process_wrap::std::{ProcessGroup, StdCommandWrap};
use shlex::Shlex;
use signal_hook::iterator::Signals;
use std::{
    io::Write,
    process::{exit, Command, Stdio},
    sync::{
        atomic::{AtomicI32, Ordering},
        Once,
    },
    thread,
};

#[derive(Debug, Clone, ArgEnum)]
pub enum LauncherType {
    Fuzzel,
    Rofi,
    Dmenu,
    Walker,
    Custom,
}

#[derive(Debug, Clone)]
pub enum LauncherCommand {
    Fuzzel {
        icon_type: String,
        placeholder: Option<String>,
    },
    Rofi {
        icon_type: String,
        placeholder: Option<String>,
    },
    Dmenu {
        prompt: Option<String>,
    },
    Walker {
        placeholder: Option<String>,
    },
    Custom {
        command: String,
        args: Vec<(String, String)>,
    },
}

static CURRENT_LAUNCHER_PID: AtomicI32 = AtomicI32::new(-1);
static SIGNAL_HANDLER_INIT: Once = Once::new();

pub struct Launcher;

impl Launcher {
    pub fn run(cmd: LauncherCommand, input: Option<&str>) -> Result<Option<String>> {
        let command = match cmd {
            LauncherCommand::Fuzzel {
                icon_type,
                placeholder,
            } => {
                let mut cmd = Command::new("fuzzel");
                cmd.arg("-d");
                if icon_type == "font" {
                    cmd.arg("-I");
                }
                if let Some(placeholder_text) = placeholder {
                    cmd.arg("--placeholder").arg(placeholder_text);
                }
                cmd
            }
            LauncherCommand::Rofi {
                icon_type,
                placeholder,
            } => {
                let mut cmd = Command::new("rofi");
                cmd.arg("-m").arg("-1").arg("-dmenu");
                if icon_type == "xdg" {
                    cmd.arg("-show-icons");
                }
                if let Some(placeholder_text) = placeholder {
                    cmd.arg("-theme-str")
                        .arg(format!("entry {{ placeholder: \"{placeholder_text}\"; }}"));
                }
                cmd
            }
            LauncherCommand::Dmenu { prompt } => {
                let mut cmd = Command::new("dmenu");
                if let Some(prompt_text) = prompt {
                    cmd.arg("-p").arg(format!("{prompt_text}: "));
                }
                cmd
            }
            LauncherCommand::Walker { placeholder } => {
                let mut cmd = Command::new("walker");
                cmd.arg("-d").arg("-k");
                if let Some(placeholder_text) = placeholder {
                    cmd.arg("-p").arg(placeholder_text);
                }
                cmd
            }
            LauncherCommand::Custom { command, args } => {
                let mut cmd_str = command;

                for (key, value) in args {
                    cmd_str = cmd_str.replace(&format!("{{{key}}}"), &value);
                }

                cmd_str = cmd_str.replace("{placeholder}", "");
                cmd_str = cmd_str.replace("{prompt}", "");

                let parts: Vec<String> = Shlex::new(&cmd_str).collect();
                let (cmd_program, args) = parts
                    .split_first()
                    .ok_or_else(|| anyhow!("Failed to parse custom launcher command"))?;

                let mut cmd = Command::new(cmd_program);
                cmd.args(args);
                cmd
            }
        };

        Self::run_command(command, input)
    }

    fn run_command(mut command: Command, input: Option<&str>) -> Result<Option<String>> {
        command.stdin(Stdio::piped()).stdout(Stdio::piped());

        let mut command_wrap = StdCommandWrap::from(command);
        command_wrap.wrap(ProcessGroup::leader());

        let mut child = command_wrap
            .spawn()
            .context("Failed to spawn launcher command")?;

        let pid = child.id() as i32;

        SIGNAL_HANDLER_INIT.call_once(|| {
            thread::spawn(|| {
                let mut signals = Signals::new([libc::SIGTERM, libc::SIGINT]).unwrap();
                if let Some(_signal) = signals.forever().next() {
                    let current_pid = CURRENT_LAUNCHER_PID.load(Ordering::Relaxed);
                    if current_pid > 0 && kill(Pid::from_raw(current_pid), None).is_ok() {
                        let _ = killpg(Pid::from_raw(current_pid), Signal::SIGTERM);
                    }
                    exit(0);
                }
            });
        });

        CURRENT_LAUNCHER_PID.store(pid, Ordering::Relaxed);

        if let Some(input_data) = input {
            if let Some(stdin) = child.stdin().as_mut() {
                stdin.write_all(input_data.as_bytes())?;
            }
        }

        let output = child.wait_with_output()?;
        let trimmed_output = String::from_utf8_lossy(&output.stdout).trim().to_string();

        CURRENT_LAUNCHER_PID.store(-1, Ordering::Relaxed);

        if trimmed_output.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed_output))
        }
    }

    pub fn create_command(
        launcher_type: &LauncherType,
        command_str: &Option<String>,
        icon_type: &str,
        prompt: Option<&str>,
        placeholder: Option<&str>,
    ) -> Result<LauncherCommand> {
        let placeholder_text = placeholder.filter(|p| !p.is_empty()).map(|p| p.to_string());
        let prompt_text = prompt.filter(|p| !p.is_empty()).map(|p| p.to_string());

        match launcher_type {
            LauncherType::Fuzzel => Ok(LauncherCommand::Fuzzel {
                icon_type: icon_type.to_string(),
                placeholder: placeholder_text,
            }),
            LauncherType::Rofi => Ok(LauncherCommand::Rofi {
                icon_type: icon_type.to_string(),
                placeholder: placeholder_text,
            }),
            LauncherType::Dmenu => Ok(LauncherCommand::Dmenu {
                prompt: prompt_text,
            }),
            LauncherType::Walker => Ok(LauncherCommand::Walker {
                placeholder: placeholder_text,
            }),
            LauncherType::Custom => {
                if let Some(cmd) = command_str {
                    let mut args = Vec::new();

                    if let Some(p) = prompt_text {
                        args.push(("prompt".to_string(), p));
                    }

                    if let Some(p) = placeholder_text {
                        args.push(("placeholder".to_string(), p));
                    }

                    Ok(LauncherCommand::Custom {
                        command: cmd.clone(),
                        args,
                    })
                } else {
                    Err(anyhow!("No custom launcher command provided"))
                }
            }
        }
    }
}
