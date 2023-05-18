use std::collections::VecDeque;
use std::env;
use std::{thread, time};
use std::io::{Error, Read, Write};
use std::os::unix::net::{UnixStream, UnixListener};
use std::path::Path;
use std::process::Stdio;

use execute::{Execute, shell};

pub const TF_PREFIX: &str = "TF_";
pub const CHDIR: &str = "/tf";
pub const SOCKET_PATH: &str = "/dockerdoom.socket";

/// Terraform DOOM controller
/// 
/// Used communicate with the UNIX socket
pub struct TfDoom {
    /// Terraform commmon/base command expr
    base: String,
    /// UNIX listener
    stream: UnixListener
}

impl TfDoom {
    pub fn new<T: AsRef<Path>>(base: &str, socket_path: T) -> Self {
        let socket = socket_path.as_ref();
    
        // Bind to socket
        let stream = match UnixListener::bind(socket) {
            Err(_) => panic!("failed to bind socket"),
            Ok(stream) => stream,
        };

        Self {
            base: String::from(base),
            stream
        }
    }

    /// Returns every resource in the Terraform project
    pub fn tf_state_list(&self) -> Vec<String> {
        let cmd = format!(
            "{} state list",
            self.base
        );

        let mut command = shell(cmd);

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let output = command.execute_output().unwrap();

        match output.status.code() {
            Some(code) => {
                if code != 0 {
                    return Vec::new()
                }

                let output = String::from_utf8_lossy(&output.stdout);

                return output
                    .split('\n')
                    .filter(| value | value.len() > 0)
                    .map(| value | format!("{}\n", value))
                    .collect();
            },
            None => Vec::new()
        }
    }

    /// Destroy a Terraform resource
    pub fn tf_destroy(&self, name: &str) {
        let cmd = format!(
            "{} destroy -auto-approve -target {}",
            self.base,
            name
        );

        let mut command = shell(cmd);

        command
            .execute()
            .expect("Unable to run terraform destroy");
    }

    /// get the Terraform resources list 
    /// then send it back to the client
    fn doom_list(&self, stream: &mut UnixStream) {
        let resources = self.tf_state_list();

        for resource in resources {
            stream
                .write(resource.as_bytes())
                .expect(
                &format!(
                    "Cannot send client Terraform ressource: {}",
                    resource
                )
            );
        }

        stream
            .shutdown(std::net::Shutdown::Write)
            .expect("Could not shutdown writing on the stream");
    }

    /// Destroy a Terraform resource identified by `name`
    fn doom_kill(&self, name: &str) {
        self.tf_destroy(name);
    }

    /// Handle a client connection
    fn handle_client(&self, stream: &mut UnixStream) {
        let mut buffer = [0; 255];

        stream
            .read(&mut buffer)
            .expect("Cannot read the UNIX stream");

        let message = String::from_utf8_lossy(&buffer);

        let mut message_split: VecDeque<&str> = message
            .split("\n")
            .collect();

        let value = message_split
            .pop_front()
            .unwrap_or_default();

        let mut value_split: VecDeque<&str> = value
            .split(" ")
            .collect();

        let instruction = value_split
            .pop_front()
            .unwrap_or_default();

        match instruction {
            "list" => self.doom_list(stream),
            "kill" => {
                let name = value_split
                    .pop_front()
                    .unwrap_or_default();

                self.doom_kill(name);
            },
            _ => return
        }
    }

    /// Wait for data from the socket
    /// then process it
    pub fn send_resources(&mut self) {
        for stream in self.stream.incoming() {
            match stream {
                Ok(mut stream) => {
                    self.handle_client(&mut stream);
                },
                Err(err) => {
                    panic!("{}", err.to_string());
                }
            }
        }
    }
}

/// Returns the base command for terraform operations
fn base_command() -> String {
    // Terraform environment vars
    let tf_vars: Vec<String> = env::vars()
        .filter(| (k, _) | k.starts_with(TF_PREFIX))
        .map(| (k, v) | format!("{}={}", k, v))
        .collect();

    let chdir = format!("-chdir={}", CHDIR);
    let mut base = String::new();
    
    if tf_vars.len() > 0 {
        base.push_str(&tf_vars.join(" "));
        base.push(' ');
    }

    base.push_str("terraform ");
    base.push_str(&chdir);

    base
}

fn main() -> Result<(), Error> {
    let base = base_command();

    let mut tfdoom = TfDoom::new(&base, SOCKET_PATH);

    shell("/usr/bin/Xvfb :99 -ac -screen 0 640x480x24")
        .spawn()
        .expect("Unable to start the virtual X session");

    thread::sleep(time::Duration::from_secs(2));

    shell("x11vnc -geometry 640x480 -forever -usepw -create -display :99")
        .spawn()
        .expect("Unable to start the VNC server");

    shell("/usr/bin/env DISPLAY=:99 /usr/local/games/psdoom -warp -E1M1 -nomouse -iwad /doom1.wad")
        .spawn()
        .expect("Run DOOM");

    tfdoom.send_resources();

    Ok(())
}
