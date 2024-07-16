use clap::{Arg, Command};

pub fn new_server_cmd() -> Command {
    clap::Command::new("server")
        .subcommand(server_add())
        .subcommand(server_set())
        .subcommand(server_remove())
        .subcommand(server_list())
        .subcommand(server_current())
}

fn server_add() -> Command {
    clap::Command::new("add")
        .about("add server")
        .args(&[Arg::new("name")
            .value_name("name")
            .required(true)
            .index(1)
            .help("server name")])
        .args(&[Arg::new("url")
            .value_name("url")
            .required(true)
            .index(2)
            .help("server url")])
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
