use std::collections::VecDeque;
use std::{env, fs};
use std::io::{Error, Read, Write};
use std::os::unix::net::{UnixStream, UnixListener};
use std::path::Path;
use std::process::Stdio;

use structopt::StructOpt;
use execute::{Execute, shell};
use uuid::Uuid;

pub const TF_PREFIX: &str = "TF_";
pub const DOOM_IMAGE: &str = "b0thr34l/dockerdoomd";
pub const SOCKET_PATH: &str = "/tmp/dockerdoomd.socket";
pub const PORT: usize = 5900;

#[derive(StructOpt, Debug)]
#[structopt(name = "tf-doom")]
/// Managing CLI arguments
struct Opt {
    /// Change project directory
    #[structopt(short, long)]
    chdir: Option<String>,
    /// VNC bound port
    #[structopt(short, long)]
    port: Option<usize>,
}

impl Opt {
    /// Return the interpreter
    pub fn chdir(&self) -> String {
        self.chdir
            .clone()
            .unwrap_or(String::from("."))
    }

    /// Return VNC port
    pub fn port(&self) -> usize {
        self.port
            .clone()
            .unwrap_or(PORT)
    }
}

/// Docker utilities
pub struct Docker;

impl Docker {
    pub fn command(args: &[&str]) -> Result<(), Error> {
        let cmd = format!(
            "docker {}",
            args.join(" ")
        );
    
        shell(cmd).spawn()?;

        Ok(())
    }
}

/// Terraform DOOM controller
/// 
/// Used communicate with the UNIX socket
pub struct TfDoom {
    /// Terraform commmon/base command expr
    base: String,
    /// UNIX listener
    stream: UnixListener,
    /// Socket path
    socket: String,
    /// Container name, used on the Drop impl
    container_name: String
}

impl TfDoom {
    pub fn new<T: AsRef<Path>>(base: &str, socket_path: T) -> Self {
        // Create the socket
        let socket = socket_path.as_ref();

        if socket.exists() {
            fs::remove_file(socket).unwrap();
        }
    
        // Bind to socket
        let stream = match UnixListener::bind(socket) {
            Err(_) => panic!("failed to bind socket"),
            Ok(stream) => stream,
        };
        let container_name = Uuid::new_v4().to_string();


        Self {
            base: String::from(base),
            stream,
            socket: socket.to_string_lossy().to_string(),
            container_name
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

    /// Run DOOM as Docker container
    pub fn run_doom(&self, port: usize) -> Result<(), Error> {
        Docker::command(&[
            "run",
            "--rm=true",
            &format!("--name=\"{}\"", self.container_name),
            &format!("-p {}:{}", port, 5900),
            &format!("-v {}:/dockerdoom.socket", self.socket),
            "doomvnc",
            "x11vnc",
            "-geometry 1280x960",
            "-forever",
            "-usepw",
            "-create"
        ])?;

        Ok(())
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
    /// 
    /// Here it is supposed to be the DOOM Docker container
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
                    break;
                }
            }
        }
    }
}

/// Returns the base command for terraform operations
fn base_command(args: &Opt) -> String {
    // Terraform environment vars
    let tf_vars: Vec<String> = env::vars()
        .filter(| (k, _) | k.starts_with(TF_PREFIX))
        .map(| (k, v) | format!("{}={}", k, v))
        .collect();

    let chdir = format!("-chdir={}", args.chdir());
    let mut base = String::new();
    
    if tf_vars.len() > 0 {
        base.push_str(&tf_vars.join(" "));
        base.push(' ');
    }

    base.push_str("terraform ");
    base.push_str(&chdir);

    base
}

fn main() -> Result<(), Error>{
    let args = Opt::from_args();
    let base = base_command(&args);

    let mut tfdoom = TfDoom::new(&base, SOCKET_PATH);

    tfdoom.run_doom(args.port())?;
    tfdoom.send_resources();

    Ok(())
}
