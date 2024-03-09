use clap::Parser;
use formatter::Formatter;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
mod element_parser;
mod formatter;
mod parser;
mod remover;
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

    #[arg(long, default_value = "time-limited")]
    time_limited_tag_name: String,

    #[arg(long, default_value = "+00:00")]
    time_limited_time_offset: String,

    #[arg(long, default_value = "")]
    time_limited_current: String,

    #[arg(long, default_value = "<!--")]
    delimiter_start: String,

    #[arg(long, default_value = "-->")]
    delimiter_end: String,
}

fn main() {
    let args = Args::parse();

    let mut content = String::new();

    if args.filename.is_none() {
        std::io::stdin()
            .read_to_string(&mut content)
            .expect("something went wrong reading the file");
    } else {
        let mut f = File::open(args.filename.unwrap()).expect("file not found");
        f.read_to_string(&mut content)
            .expect("something went wrong reading the file");
    }

    let mut builder_map: HashMap<
        &str,
        Box<dyn remover::remove_marker_builder::RemoveMarkerBuilder>,
    > = HashMap::new();
    builder_map.insert(
        args.time_limited_tag_name.as_str(),
        Box::new(remover::time_limited_remover::TimeLimitedRemover {
            current_time: args
                .time_limited_current
                .parse::<chrono::DateTime<chrono::Local>>()
                .unwrap_or(chrono::Local::now()),
            time_offset: args.time_limited_time_offset.to_string(),
        }),
    );

    let tokens = tokenizer::tokenize(
        &content,
        args.delimiter_start.as_str(),
        args.delimiter_end.as_str(),
    );

    let (removed, markers) = remover::remove(parser::parse(&tokens), &content, &builder_map);

    let removed_pos = remover::get_removed_pos(&markers);
    let formatter: Vec<Box<dyn Formatter>> = vec![
        Box::new(formatter::indent_remover::IndentRemover {}),
        Box::new(formatter::prev_line_break_remover::PrevLineBreakRemover {}),
    ];
    let cleaned = formatter::format(&removed, &removed_pos, &formatter);

    if let Some(output) = args.output {
        let mut f = File::create(output).expect("file not found");
        f.write_all(cleaned.as_bytes())
            .expect("something went wrong writing the file");
    } else {
        print!("{}", cleaned);
    }
}
