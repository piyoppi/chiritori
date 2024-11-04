use chiritori::{ChiritoriConfiguration, MarkerTagConfiguration, TimeLimitedConfiguration};
use clap::Parser;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
mod chiritori;
mod code;
mod element_parser;
mod parser;
mod tokenizer;

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
    #[arg(long, default_value = "<!--")]
    delimiter_start: String,

    /// The delimiter end
    #[arg(long, default_value = "-->")]
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
    marker_tag_name: String,

    /// Name of removal-marker to be removed
    #[arg(long, default_value = "vec![]")]
    marker_removal_names: Vec<String>,
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

    let content = Rc::new(content);
    let marker_removal_tags = HashSet::from_iter(args.marker_removal_names);

    let config = ChiritoriConfiguration {
        delimiter_start: args.delimiter_start,
        delimiter_end: args.delimiter_end,
        time_limited_configuration: TimeLimitedConfiguration {
            tag_name: args.time_limited_tag_name,
            time_offset: args.time_limited_time_offset,
            current: args
                .time_limited_current
                .parse::<chrono::DateTime<chrono::Local>>()
                .unwrap_or(chrono::Local::now()),
        },
        marker_tag_configuration: MarkerTagConfiguration {
            tag_name: args.marker_tag_name,
            marker_removal_tags,
        },
    };

    let cleaned = chiritori::clean(content, config);

    if let Some(output) = args.output {
        let mut f = File::create(output).expect("file not found");
        f.write_all(cleaned.as_bytes())
            .expect("something went wrong writing the file");
    } else {
        print!("{}", cleaned);
    }
}
