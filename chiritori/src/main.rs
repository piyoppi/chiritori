extern crate lib_chiritori;
use lib_chiritori::chiritori::{ChiritoriConfiguration, MarkerTagConfiguration, TimeLimitedConfiguration};
use clap::Parser;
use std::collections::HashSet;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::rc::Rc;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The filename to read
    #[arg(short, long)]
    filename: Option<String>,

    /// The filename to write
    #[arg(short, long)]
    output: Option<String>,

    /// The delimiter start
    #[arg(long, default_value = "<!-- <")]
    delimiter_start: String,

    /// The delimiter end
    #[arg(long, default_value = "> -->")]
    delimiter_end: String,

    /// The tag name for time-limited content
    #[arg(long, default_value = "time-limited")]
    time_limited_tag_name: String,

    /// The time offset for time-limited content
    #[arg(long, default_value = "+00:00")]
    time_limited_time_offset: String,

    /// The current time for time-limited content
    #[arg(long, default_value = "")]
    time_limited_current: String,

    /// The tag name for removal-marker
    #[arg(long, default_value = "removal-marker")]
    removal_marker_tag_name: String,

    /// Name of removal-marker to be removed
    #[arg(long, default_value = "vec![]")]
    removal_marker_target_name: Vec<String>,

    /// Config file specifying the name of the removal-marker to be removed.
    /// The content of the config file is indicated by the name of the removal target, separated by a newline.
    #[arg(long)]
    removal_marker_target_config: Option<String>,

    /// List source code to be removed
    #[arg(short, long)]
    list: bool,

    /// List source code to be removed or pending
    #[arg(long, long)]
    list_all: bool,
}

fn main() {
    let args = Args::parse();

    let mut content = String::new();
    if args.filename.is_none() {
        if atty::isnt(atty::Stream::Stdin) {
            std::io::stdin()
                .read_to_string(&mut content)
                .expect("something went wrong reading the file");
        } else {
            println!("No input file or stdin. More information: --help");
            std::process::exit(1);
        }
    } else {
        let mut f = File::open(args.filename.unwrap()).expect("file not found");
        f.read_to_string(&mut content)
            .expect("something went wrong reading the file");
    }

    let removal_marker_target_names_from_file =
        if let Some(removal_marker_target_config) = args.removal_marker_target_config {
            load_removal_marker_target_names(removal_marker_target_config)
        } else {
            vec![]
        };

    let marker_removal_tags: HashSet<_> = removal_marker_target_names_from_file
        .into_iter()
        .chain(args.removal_marker_target_name)
        .collect();

    let config = ChiritoriConfiguration {
        time_limited_configuration: TimeLimitedConfiguration {
            tag_name: args.time_limited_tag_name,
            time_offset: args.time_limited_time_offset,
            current: args
                .time_limited_current
                .parse::<chrono::DateTime<chrono::Local>>()
                .unwrap_or(chrono::Local::now()),
        },
        marker_tag_configuration: MarkerTagConfiguration {
            tag_name: args.removal_marker_tag_name,
            marker_removal_tags,
        },
    };

    let content = Rc::new(content);

    let output = if args.list {
        lib_chiritori::chiritori::list(content, (args.delimiter_start, args.delimiter_end), config)
    } else if args.list_all {
        lib_chiritori::chiritori::list_all(content, (args.delimiter_start, args.delimiter_end), config)
    } else {
        lib_chiritori::chiritori::clean(content, (args.delimiter_start, args.delimiter_end), config)
    };

    if let Some(filename) = args.output {
        let mut f = File::create(filename).expect("file not found");
        f.write_all(output.as_bytes())
            .expect("something went wrong writing the file");
    } else {
        print!("{}", output);
    }
}

fn load_removal_marker_target_names(filename: String) -> Vec<String> {
    let f = File::open(filename).expect("file not found");
    let reader = BufReader::new(f);

    reader.lines().map_while(Result::ok).collect::<Vec<_>>()
}
