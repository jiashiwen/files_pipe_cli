use clap::{value_parser, Arg, Command};

pub fn new_gen_file_cmd() -> Command {
    clap::Command::new("gen_file")
        .about("gen_file")
        .args(&[Arg::new("file_size")
            .value_name("file_size")
            .required(true)
            .index(1)
            .help("specific generated file size,example 100K,1M,2G")])
        .args(&[Arg::new("chunk_size")
            .value_name("chunk_size")
            .required(true)
            // .value_parser(value_parser!(usize))
            .index(2)
            .help("specific generated file write chunk size")])
        .args(&[Arg::new("file_name")
            .value_name("file_name")
            .required(true)
            .index(3)
            .help("specific generated file path")])
}

pub fn new_gen_files_cmd() -> Command {
    clap::Command::new("gen_files")
        .about("gen_files")
        .args(&[Arg::new("dir")
            .value_name("dir")
            .required(true)
            .index(1)
            .help("specific generated files folder")])
        .args(&[Arg::new("file_prefix_len")
            .value_name("file_prefix_len")
            .required(true)
            .value_parser(value_parser!(usize))
            .index(2)
            .help("specific generated file prefix length")])
        .args(&[Arg::new("file_size")
            .value_name("file_size")
            .required(true)
            .index(3)
            .help("specific generated file size")])
        .args(&[Arg::new("chunk_size")
            .value_name("chunk_size")
            .required(true)
            .index(4)
            .help("specific generated file write chunk size")])
        .args(&[Arg::new("file_quantity")
            .value_name("file_quantity")
            .required(true)
            .value_parser(value_parser!(usize))
            .index(5)
            .help("specific how may files to generated")])
}
