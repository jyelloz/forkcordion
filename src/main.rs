use clap::Parser;
use clio::Input;
use std::io::{Seek, Write};

use console::style;

use forkcordion::{
    Archive,
    Format,
    SeekableArchive,
    applesingle::{self, Fork},
};

#[derive(Parser, Debug)]
#[clap(name = "applesingle-info", author, version, about)]
struct InfoCommand {
    #[clap(value_parser, default_value = "-")]
    input: Input,
    #[clap(value_parser)]
    output_rsrc: Option<clio::Output>,
    #[clap(value_parser)]
    output_data: Option<clio::Output>,
}

impl InfoCommand {
    fn seekable(&mut self) -> bool {
        match self.input.seek(std::io::SeekFrom::Current(0)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

struct Handler {
    output_rsrc: Option<clio::Output>,
    output_data: Option<clio::Output>,
}

impl Handler {
    fn boxed_output_stream<'a>(
        stream: &'a mut Option<clio::Output>
    ) -> Option<Box<dyn Write + 'a>> {
        if let Some(stream) = stream {
            Some(Box::new(stream))
        } else {
            None
        }
    }
}

impl applesingle::Handler for Handler {
    fn sink<'a>(
        &'a mut self,
        fork: applesingle::Fork
    ) -> Option<Box<dyn Write + 'a>> {
        match fork {
            Fork::Rsrc => Self::boxed_output_stream(&mut self.output_rsrc),
            Fork::Data => Self::boxed_output_stream(&mut self.output_data),
            Fork::Other(id) => {
                eprintln!("other fork, id={id}");
                None
            }
        }
    }
}

enum ArchiveKind<R> {
    Seekable(SeekableArchive<R>),
    Streaming(Archive),
}

impl <R: std::io::Read + std::io::Seek> ArchiveKind<R> {
    fn format(&self) -> Format {
        match self {
            Self::Seekable(a) => a.format(),
            Self::Streaming(a) => a.format(),
        }
    }
    fn name(&self) -> Option<forkcordion::Filename> {
        match self {
            Self::Seekable(a) => a.name(),
            Self::Streaming(a) => a.name(),
        }
    }
    fn finder_info(&self) -> Option<forkcordion::FinderInfo> {
        match self {
            Self::Seekable(a) => a.finder_info(),
            Self::Streaming(a) => a.finder_info(),
        }
    }
}

fn main() {

    let mut cmd = InfoCommand::parse();
    let seekable = cmd.seekable();
    let InfoCommand { input, mut output_rsrc, mut output_data } = cmd;

    eprintln!(
        "info on {:?}",
        style(&input).yellow(),
    );

    let archive = if seekable {
        let mut archive = applesingle::parse_seekable(input)
            .expect("failed to parse seekable archive");
        if let (Some(out), Ok(Some(mut fork))) = (&mut output_data, archive.data_fork()) {
            std::io::copy(&mut fork, out)
                .expect("failed to export data fork");
        }
        if let (Some(out), Ok(Some(mut fork))) = (&mut output_rsrc, archive.rsrc_fork()) {
            std::io::copy(&mut fork, out)
                .expect("failed to export rsrc fork");
        }
        ArchiveKind::Seekable(archive)
    } else {
        let mut h = Handler { output_rsrc, output_data } ;
        let archive = applesingle::parse(input, &mut h)
            .expect("failed to parse streaming archive");
        ArchiveKind::Streaming(archive)
    };

    eprintln!("format={}", style(archive.format()).cyan());
    if let Some(name) = archive.name() {
        eprintln!("name={}", style(name).cyan());
    }
    if let Some(finf) = archive.finder_info() {
        eprintln!("finf={:?}", style(finf).cyan());
    }

}
