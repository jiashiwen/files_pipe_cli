use clap::{Arg, Command};

pub fn new_task_cmd() -> Command {
    clap::Command::new("task")
        .subcommand(task_exec())
        .subcommand(task_analyze_source())
        .subcommand(task_list_all())
        .subcommand(task_show())
        .subcommand(task_status())
        .subcommand(task_checkpoint())
}

fn task_exec() -> Command {
    clap::Command::new("exec")
        .about("execute task description yaml file")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("execute task description yaml file")])
}

fn task_analyze_source() -> Command {
    clap::Command::new("analyze")
        .about("analyze source objects destributed")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("analyze source objects destributed")])
}

fn task_list_all() -> Command {
    clap::Command::new("list_all").about("list_all")
}

fn task_show() -> Command {
    clap::Command::new("show")
        .about("show task description")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("analyze source objects destributed")])
}

fn task_status() -> Command {
    clap::Command::new("status")
        .about("show task status")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("analyze source objects destributed")])
}

fn task_checkpoint() -> Command {
    clap::Command::new("checkpoint")
        .about("show task checkpoint")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("analyze source objects destributed")])
}
