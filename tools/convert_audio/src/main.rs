use serde::Deserialize;
use std::{fs::File, io::Write, path::PathBuf};

#[derive(Deserialize)]
struct Args {
    input: PathBuf,
    output: PathBuf,
}

fn main() {
    let args: Args = match serde_args::from_env() {
        Ok(args) => args,
        Err(error) => {
            println!("{error:#}");
            return;
        }
    };

    let mut reader = hound::WavReader::open(args.input).unwrap();
    let mut file = File::create(args.output).unwrap();
    file.write_all(&reader.spec().sample_rate.to_le_bytes())
        .unwrap();
    file.write_all(
        &reader
            .samples::<i8>()
            .map(|i| i.unwrap() as u8)
            .collect::<Vec<_>>(),
    )
    .unwrap();
}
