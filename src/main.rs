use clap::Parser;
use std::io::Error;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use std::{env, fs};
use which::which;

#[derive(Parser, Debug)]
#[clap(
    name = "CredDock",
    version = "0.1.0",
    about = "A command line tool to run docker commands with Google Cloud credentials."
)]
struct Args {
    #[clap(
        long,
        default_value_t = get_default_credentials_path().to_string_lossy().to_string(),
        value_hint = clap::ValueHint::DirPath,
        help = "The path to your Google Cloud credentials. Defaults to the default user credentials."
    )]
    adc: String,

    #[clap(
        long,
        default_value = "/tmp/keys/creds.json",
        help = "Path to the credentials file inside the docker container."
    )]
    adc_docker: String,

    #[clap(long, help = "Google Cloud project name.")]
    project: String,

    #[clap(
        short,
        long,
        help = "Path to the directory where the Docker image should be built."
    )]
    context: String,

    #[clap(
        long,
        num_args = 1..,
        default_value = "",
        help = "Command line arguments to pass to the docker command. Separate multiple arguments with commas."
    )]
    args: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if which("docker").is_err() {
        panic!("docker not found");
    }
    let image_hash = match build_docker_image(&args) {
        Ok(image_hash) => image_hash,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    match run_docker_image(&args, &image_hash) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}

fn get_default_credentials_path() -> PathBuf {
    match env::consts::FAMILY {
        "unix" => {
            let mut credentials_path = PathBuf::from(env::var("HOME").ok().unwrap());
            credentials_path.push(".config");
            credentials_path.push("gcloud");
            credentials_path.push("application_default_credentials.json");
            credentials_path
        }
        "windows" => {
            let mut credentials_path = PathBuf::from(env::var("APPDATA").ok().unwrap());
            credentials_path.push("gcloud");
            credentials_path.push("application_default_credentials.json");
            credentials_path
        }
        _ => panic!("Unsupported OS"),
    }
}

fn build_docker_image(args: &Args) -> Result<String, Error> {
    let output = Command::new("docker")
        .arg("build")
        .arg("-q")
        .arg(&args.context)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        Ok(stdout.trim().to_string())
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        eprintln!("{}", stderr);
        Err(Error::new(std::io::ErrorKind::Other, "docker build failed"))
    }
}

fn run_docker_image(args: &Args, image_hash: &String) -> Result<(), Error> {
    let local_creds = fs::canonicalize(&args.adc)?;
    println!("local_creds: {:?}", local_creds);
    let mut command = Command::new("docker");
    command
        .args(["run", "--rm"])
        .args([
            "-e",
            &format!("GOOGLE_APPLICATION_CREDENTIALS={}", args.adc_docker),
        ])
        .args(["-e", &format!("GOOGLE_CLOUD_PROJECT={}", args.project)])
        .args([
            "-v",
            &format!(
                "{}:{}:ro",
                dbg!(local_creds.to_string_lossy()),
                args.adc_docker
            ),
        ])
        .arg(image_hash);

    if !args
        .args
        .iter()
        .filter(|arg| !arg.is_empty())
        .collect::<Vec<_>>()
        .is_empty()
    {
        command.args(&args.args);
    }
    let mut child = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = child.wait()?;

    if !status.success() {
        Err(Error::new(std::io::ErrorKind::Other, "docker run failed"))
    } else {
        Ok(())
    }
}
