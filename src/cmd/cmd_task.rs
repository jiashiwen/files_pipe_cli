use clap::{Arg, Command};

pub fn new_task_cmd() -> Command {
    clap::Command::new("task")
        .subcommand(task_show())
        .subcommand(task_create())
        .subcommand(task_update())
        .subcommand(task_remove())
        .subcommand(task_clean())
        .subcommand(task_start())
        .subcommand(task_stop())
        .subcommand(task_checkpoint())
        .subcommand(task_status())
        .subcommand(task_analyze())
        .subcommand(task_list_all())
        .subcommand(task_all_living())
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

fn task_create() -> Command {
    clap::Command::new("create")
        .about("create task")
        .args(&[Arg::new("taskjson")
            .value_name("taskjson")
            .required(true)
            .index(1)
            .help("create task with json")])
}

fn task_update() -> Command {
    clap::Command::new("update")
        .about("create task")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("specify task id")])
        .args(&[Arg::new("taskjson")
            .value_name("taskjson")
            .required(true)
            .index(1)
            .help("new task json")])
}

fn task_remove() -> Command {
    clap::Command::new("remove")
        .about("remove task")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("remove task by task id")])
}

fn task_clean() -> Command {
    clap::Command::new("clean")
        .about("clean task")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("create task with json")])
}

fn task_start() -> Command {
    clap::Command::new("start")
        .about("start task")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("create task with json")])
}

fn task_stop() -> Command {
    clap::Command::new("stop")
        .about("stop task")
        .args(&[Arg::new("taskid")
            .value_name("taskid")
            .required(true)
            .index(1)
            .help("create task with json")])
}

fn task_analyze() -> Command {
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

fn task_all_living() -> Command {
    clap::Command::new("all_living").about("all living tasks")
}
