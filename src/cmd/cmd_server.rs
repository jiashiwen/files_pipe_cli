use clap::{Arg, Command};

pub fn new_server_cmd() -> Command {
    clap::Command::new("server")
        .subcommand(server_save())
        .subcommand(server_set())
        .subcommand(server_remove())
        .subcommand(server_list())
        .subcommand(server_current())
}

fn server_save() -> Command {
    clap::Command::new("save")
        .about("save server")
        .args(&[Arg::new("server_string")
            .value_name("server_string")
            .required(true)
            .index(1)
            .help("execute task description yaml file")])
}

fn server_set() -> Command {
    clap::Command::new("set")
        .about("set current server")
        .args(&[Arg::new("server_id")
            .value_name("server_id")
            .required(true)
            .index(1)
            .help("analyze source objects destributed")])
}

fn server_remove() -> Command {
    clap::Command::new("remove")
        .about("remove_server")
        .args(&[Arg::new("server_id")
            .value_name("server_id")
            .required(true)
            .index(1)
            .help("analyze source objects destributed")])
}

fn server_list() -> Command {
    clap::Command::new("list").about("list_all_servers")
}

fn server_current() -> Command {
    clap::Command::new("current").about("list current server")
}
