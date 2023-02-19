use std::{
    fs::{self, File},
    process::{self, Stdio},
};

use clap::Parser;
use daemonize::Daemonize;

use futures::{
    channel::mpsc::{channel, Receiver},
    future::try_join_all,
    SinkExt, StreamExt,
};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use notify::{self};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Debug, Clone, clap::ValueEnum)]
enum CliCommand {
    Start,
    Stop,
    Restart,
    Debug,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    command: CliCommand,
}

use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct InCronJob {
    label: String,
    watch: String,
    events: Vec<String>,
    command: String,
}

#[derive(Deserialize, Clone)]
struct InCronConfig {
    logfile: String,
    pidfile: String,
    jobs: Vec<InCronJob>,
}

fn run(config: InCronConfig) {
    futures::executor::block_on(async {
        let v: Vec<_> = config.jobs.iter().map(|job| async_watch(job)).collect();

        try_join_all(v).await.unwrap();
    });
}

fn start(config: InCronConfig) {
    let log_file = File::create(config.logfile.clone()).unwrap();

    let stdout = log_file.try_clone().unwrap();
    let stderr = log_file.try_clone().unwrap();

    let daemonize = Daemonize::new()
        .pid_file(config.pidfile.clone())
        .umask(0o777)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            println!("Success, daemonized");
            run(config);
            println!("Finished");
        }
        Err(e) => eprintln!("Error, {}", e),
    }
}
fn stop(config: InCronConfig) {
    let contents = fs::read_to_string(&config.pidfile)
        .expect(&format!("Cannot read config file at {}", config.pidfile));

    let pid: i32 = contents.parse().unwrap();
    signal::kill(Pid::from_raw(pid), Signal::SIGTERM).unwrap();

    println!("Stopped successfully")
}
fn debug(config: InCronConfig) {
    run(config)
}

fn main() {
    let args = Cli::parse();

    let home = home::home_dir().unwrap();

    let config_path = format!("{}/.config/incronrs/config.json", home.to_str().unwrap());

    let contents = fs::read_to_string(&config_path)
        .expect(&format!("Cannot read config file at {}", &config_path));

    let config = serde_json::from_str::<InCronConfig>(&contents).expect(&format!(
        "Configuration file {} format mismatch. Check file and run again",
        config_path
    ));

    match args.command {
        CliCommand::Start => start(config),
        CliCommand::Stop => stop(config),
        CliCommand::Restart => {
            stop(config.clone());
            start(config);
        }
        CliCommand::Debug => debug(config),
    };
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch(job: &InCronJob) -> notify::Result<()> {
    println!(
        "Registered Job: {} with events: {:?}",
        job.label, job.events
    );

    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(job.watch.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                println!("receiving event: {:?}", event);

                if job.events.contains(&format!("{:?}", event.kind)) {
                    println!("processing event: {:?}", event);

                    let command = job
                        .command
                        .clone()
                        .replace("$watched", &job.watch)
                        .replace("$filename", event.paths[0].to_str().unwrap())
                        .replace("$event", &format!("{:?}", &event.kind));

                    println!("running command: {}", command);

                    let mut child = process::Command::new("bash")
                        .arg("-c")
                        .arg(command)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()
                        .expect("Process failed to start");

                    child.wait().unwrap();
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
