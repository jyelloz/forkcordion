use clap::Parser;
use clio::Input;
use std::io::Write;

use console::style;

use forkcordion::applesingle::{self, Fork};

#[derive(Parser, Debug)]
#[clap(name = "applesingle-info", author, version, about)]
struct InfoCommand {
    #[clap(value_parser, default_value = "-")]
    input: Input,
}

struct Handler;

impl applesingle::Handler for Handler {
    fn sink(&mut self, fork: applesingle::Fork) -> Option<Box<dyn Write>> {
        match fork {
            Fork::Rsrc => {
                Some(Box::new(std::io::stdout()))
            }
            Fork::Data => {
                eprintln!("data fork");
                None
            },
            Fork::Other(id) => {
                eprintln!("other fork, id={id}");
                None
            }
        }
    }
}

fn main() {

    let cmd = InfoCommand::parse();

    eprintln!(
        "info on {:?}",
        style(&cmd.input).yellow(),
    );

    let mut h = Handler;

    let archive = applesingle::parse(cmd.input, &mut h)
        .expect("could not decode as applesingle");

    eprintln!("format={}", style(archive.format()).cyan());

}
