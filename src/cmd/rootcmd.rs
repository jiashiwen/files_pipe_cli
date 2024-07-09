use crate::cmd::cmd_gen_file::{new_gen_file_cmd, new_gen_files_cmd};
use crate::cmd::cmd_server::new_server_cmd;
use crate::cmd::cmd_task::new_task_cmd;
use crate::cmd::{
    new_command_tree_cmd, new_config_cmd, new_exit_cmd, new_parameters_cmd, new_template,
};
use crate::commons::CommandCompleter;
use crate::commons::{byte_size_str_to_usize, generate_file, generate_files, SubCmd};
use crate::configure::{generate_default_config, set_config_file_path};
use crate::configure::{get_config_file_path, get_current_config_yml, set_config};
use crate::interact;
use crate::interact::INTERACT_STATUS;
use crate::request::GLOBAL_CURRENT_SERVER;
use crate::resources::{
    get_server_url_str_from_cf, list_servers_from_cf, remove_server_from_cf, save_server_to_cf,
};
use clap::{Arg, ArgAction, ArgMatches, Command as Clap_Command};
use lazy_static::lazy_static;
use tabled::builder::Builder;

pub const APP_NAME: &'static str = "files_pipe_cli";

lazy_static! {
    static ref CLIAPP: Clap_Command = Clap_Command::new(APP_NAME)
        .version("1.0")
        .author("Shiwen Jia. <jiashiwen@gmail.com>")
        .about("file_pipe_cli")
        .arg_required_else_help(true)
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
        )
        .arg(
            Arg::new("interact")
                .short('i')
                .long("interact")
                .action(ArgAction::SetTrue)
                .help("run as interact mod")
        )
        .subcommand(new_server_cmd())
        .subcommand(new_task_cmd())
        .subcommand(new_template())
        .subcommand(new_parameters_cmd())
        .subcommand(new_config_cmd())
        .subcommand(new_gen_file_cmd())
        .subcommand(new_gen_files_cmd())
        .subcommand(new_exit_cmd())
        .subcommand(new_command_tree_cmd());
    static ref SUBCMDS: Vec<SubCmd> = subcommands();
}

pub fn run_app() {
    set_config("");
    let matches = CLIAPP.clone().get_matches();
    cmd_match(&matches);
}

pub fn run_from(args: Vec<String>) {
    match Clap_Command::try_get_matches_from(CLIAPP.to_owned(), args.clone()) {
        Ok(matches) => {
            cmd_match(&matches);
        }
        Err(err) => {
            err.print().expect("Error writing Error");
        }
    };
}

// 获取全部子命令，用于构建commandcompleter
pub fn all_subcommand(app: &Clap_Command, beginlevel: usize, input: &mut Vec<SubCmd>) {
    let nextlevel = beginlevel + 1;
    let mut subcmds = vec![];
    for iterm in app.get_subcommands() {
        subcmds.push(iterm.get_name().to_string());
        if iterm.has_subcommands() {
            all_subcommand(iterm, nextlevel, input);
        } else {
            if beginlevel == 0 {
                all_subcommand(iterm, nextlevel, input);
            }
        }
    }
    let subcommand = SubCmd {
        level: beginlevel,
        command_name: app.get_name().to_string(),
        subcommands: subcmds,
    };
    input.push(subcommand);
}

pub fn get_cmd_tree(cmd: &Clap_Command) -> termtree::Tree<String> {
    let mut tree = termtree::Tree::new(cmd.get_name().to_string());
    if cmd.has_subcommands() {
        let mut vec_t = vec![];
        for item in cmd.get_subcommands() {
            let t = get_cmd_tree(item);
            vec_t.push(t);
        }
        tree = tree.with_leaves(vec_t);
    }
    tree
}

pub fn get_command_completer() -> CommandCompleter {
    CommandCompleter::new(SUBCMDS.to_vec())
}

fn subcommands() -> Vec<SubCmd> {
    let mut subcmds = vec![];
    all_subcommand(&CLIAPP, 0, &mut subcmds);
    subcmds
}

fn cmd_match(matches: &ArgMatches) {
    if let Some(c) = matches.get_one::<String>("config") {
        set_config_file_path(c.to_string());
        set_config(&get_config_file_path());
    } else {
        set_config("");
    }

    if matches.get_flag("interact") {
        if !INTERACT_STATUS.load(std::sync::atomic::Ordering::SeqCst) {
            interact::run();
            return;
        }
    }

    if let Some(config) = matches.subcommand_matches("config") {
        if let Some(_show) = config.subcommand_matches("show") {
            let yml = get_current_config_yml();
            match yml {
                Ok(str) => {
                    println!("{}", str);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }

        if let Some(gen_config) = config.subcommand_matches("gendefault") {
            let mut file = String::from("");
            if let Some(path) = gen_config.get_one::<String>("filepath") {
                file.push_str(path);
            } else {
                file.push_str("config_default.yml")
            }
            if let Err(e) = generate_default_config(file.as_str()) {
                log::error!("{}", e);
                return;
            };
            println!("{} created!", file);
        }
    }

    if let Some(server) = matches.subcommand_matches("server") {
        if let Some(_) = server.subcommand_matches("current") {
            println!("{:?}", GLOBAL_CURRENT_SERVER.read());
        }
        if let Some(save) = server.subcommand_matches("save") {
            if let Some(server) = save.get_one::<String>("server_string") {
                println!("{:?}", save_server_to_cf(server));
            }
        }

        if let Some(set) = server.subcommand_matches("set") {
            if let Some(id) = set.get_one::<String>("server_id") {
                let server = match get_server_url_str_from_cf(id) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        return;
                    }
                };

                let mut current = match GLOBAL_CURRENT_SERVER.write() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        return;
                    }
                };
                *current = server;
                println!("current server {:?} setted!", current);
            }
        }

        if let Some(remove) = server.subcommand_matches("remove") {
            if let Some(id) = remove.get_one::<String>("server_id") {
                println!("{:?}", remove_server_from_cf(id));
            }
        }

        if let Some(_) = server.subcommand_matches("list") {
            let servers_list = match list_servers_from_cf() {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("{:?}", e);
                    return;
                }
            };

            let mut builder = Builder::default();
            for (id, url) in servers_list {
                let raw = vec![id, url];
                builder.push_record(raw);
            }

            let header = vec!["server id", "url"];
            builder.insert_record(0, header);
            let table = builder.build();
            println!("{}", table);
        }
    }

    if let Some(task) = matches.subcommand_matches("task") {
        if let Some(exec) = task.subcommand_matches("exec") {
            if let Some(id) = exec.get_one::<String>("taskid") {
                println!("task id:{}", id);
            }
        }

        if let Some(analyze) = task.subcommand_matches("analyze") {
            if let Some(id) = analyze.get_one::<String>("taskid") {
                println!("task id:{}", id);
            }
        }
    }

    if let Some(template) = matches.subcommand_matches("template") {
        if let Some(transfer) = template.subcommand_matches("transfer") {
            if let Some(oss2oss) = transfer.subcommand_matches("oss2oss") {
                println!("template transfer oss2oss");
            }
            if let Some(oss2local) = transfer.subcommand_matches("oss2local") {
                println!("template transfer oss2local");
            }
            if let Some(local2oss) = transfer.subcommand_matches("local2oss") {
                println!("template transfer local2oss");
            }

            if let Some(local2local) = transfer.subcommand_matches("local2local") {
                println!("template transfer local2local");
            }
        }

        if let Some(truncate_bucket) = template.subcommand_matches("truncate_bucket") {
            println!("template truncate");
        }

        if let Some(compare) = template.subcommand_matches("compare") {
            println!("template compare");
        }
    }

    if let Some(parameters) = matches.subcommand_matches("parameters") {
        if let Some(_) = parameters.subcommand_matches("provider") {
            println!("parameters");
            // println!("{:?}", OssProvider::JD);
            // println!("{:?}", OssProvider::JRSS);
            // println!("{:?}", OssProvider::ALI);
            // println!("{:?}", OssProvider::AWS);
            // println!("{:?}", OssProvider::HUAWEI);
            // println!("{:?}", OssProvider::COS);
            // println!("{:?}", OssProvider::MINIO);
        }

        if let Some(_) = parameters.subcommand_matches("task_type") {
            println!("task_type");
            // println!("{:?}", TaskType::Transfer);
            // println!("{:?}", TaskType::TruncateBucket);
        }
    }

    if let Some(gen_file) = matches.subcommand_matches("gen_file") {
        let file_size = match gen_file.get_one::<String>("file_size") {
            Some(s) => {
                let size = byte_size_str_to_usize(s);
                match size {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("{}", e);
                        return;
                    }
                }
            }
            None => {
                return;
            }
        };
        let chunk: usize = match gen_file.get_one::<String>("chunk_size") {
            // Some(s) => *s,
            // None => {
            //     return;
            // }
            Some(s) => {
                let size = byte_size_str_to_usize(s);
                match size {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("{}", e);
                        return;
                    }
                }
            }
            None => {
                return;
            }
        };

        let file = match gen_file.get_one::<String>("file_name") {
            Some(s) => s,
            None => {
                return;
            }
        };

        if let Err(e) = generate_file(file_size, chunk, file) {
            log::error!("{}", e);
        };
    }

    if let Some(gen_file) = matches.subcommand_matches("gen_files") {
        let dir = match gen_file.get_one::<String>("dir") {
            Some(s) => s,
            None => {
                return;
            }
        };
        let file_prefix_len: usize = match gen_file.get_one("file_prefix_len") {
            Some(s) => *s,
            None => {
                return;
            }
        };

        let file_size = match gen_file.get_one::<String>("file_size") {
            Some(s) => {
                let size = byte_size_str_to_usize(s);
                match size {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("{}", e);
                        return;
                    }
                }
            }
            None => {
                return;
            }
        };

        let chunk_size: usize = match gen_file.get_one::<String>("chunk_size") {
            Some(s) => {
                let size = byte_size_str_to_usize(s);
                match size {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("{}", e);
                        return;
                    }
                }
            }
            None => {
                return;
            }
        };

        let file_quantity: usize = match gen_file.get_one("file_quantity") {
            Some(s) => *s,
            None => {
                return;
            }
        };

        if let Err(e) = generate_files(
            dir.as_str(),
            file_prefix_len,
            file_size,
            chunk_size,
            file_quantity,
        ) {
            log::error!("{}", e);
        };
    }

    if let Some(_) = matches.subcommand_matches("tree") {
        let tree = get_cmd_tree(&CLIAPP);
        println!("{}", tree);
    }
}

#[cfg(test)]
mod test {
    use crate::cmd::rootcmd::{get_cmd_tree, CLIAPP};

    //cargo test cmd::rootcmd::test::test_get_command_tree -- --nocapture
    #[test]
    fn test_get_command_tree() {
        let tree = get_cmd_tree(&CLIAPP);
        println!("{}", tree);
    }
}