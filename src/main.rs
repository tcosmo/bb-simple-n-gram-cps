mod ngram;
mod program;

use clap::Parser;
use program::{LoopsForever, MayHalt, Program};

use std::io::{Read, Seek, Write};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // The machine to handle, in 30-character or 34-character format.
    #[clap(
        short,
        long,
        value_parser,
        default_value_t = String::new(),
        help = "A machine, as either a 34-character string '1RB0LC_0LA1RD_1LA0RB_1LE---_0RA1RE' or a 30-character string like '1RB0LC0LA1RD1LA0RB1LE---0RA1RE'."
    )]
    machine: String,

    #[clap(long, default_value_t = String::new())]
    seed_database: String,

    #[clap(long, default_value_t = String::new())]
    undecided_index: String,

    #[clap(long, default_value_t = 4)]
    radius: u8,

    #[clap(long, default_value_t = 1_000_000)]
    max_context_count: usize,
}

fn main() -> Result<(), i32> {
    use ngram::classify as classify_fn;

    use std::time::Instant;

    let start_time = Instant::now();

    let args = Args::parse();
    println!("args: {:?}", args);

    if !args.seed_database.is_empty() {
        let mut output_file_looping =
            std::fs::File::create(format!("index-looping-n-{}", args.radius))
                .expect("can create index-looping-n-{}");
        let mut output_file_halting =
            std::fs::File::create(format!("index-undecided-n-{}", args.radius))
                .expect("can create index-undecided-n-{}");

        let mut seed_database =
            std::fs::File::open(args.seed_database).expect("--seed_database can be opened");
        let mut previously_undecided_index =
            std::fs::File::open(args.undecided_index).expect("--undecided_index can be opened");

        let mut count_processed = 0;
        let mut count_loops = 0;
        let mut count_undecided = 0;

        loop {
            let mut machine_index_bytes_be: [u8; 4] = [0; 4];
            let count_read = previously_undecided_index
                .read(&mut machine_index_bytes_be)
                .expect("can read bytes");
            if count_read == 0 {
                break;
            }
            if count_read != 4 {
                panic!("invalid");
            }
            let machine_index = u32::from_be_bytes(machine_index_bytes_be);
            seed_database
                .seek(std::io::SeekFrom::Start((machine_index + 1) as u64 * 30))
                .expect("seed succeeded");

            let mut machine_bytes: [u8; 30] = [0; 30];
            let count = seed_database
                .read(&mut machine_bytes)
                .expect("read succeeds");
            if count != machine_bytes.len() {
                panic!(
                    "unexpected read; only got {} of {} expected for machine_index={machine_index}",
                    count,
                    machine_bytes.len()
                );
            }

            let machine = Program::from_string(
                std::str::from_utf8(&machine_bytes).expect("valid utf8, barely"),
            );

            count_processed += 1;
            match classify_fn(&machine, args.radius, args.max_context_count) {
                Ok(LoopsForever) => {
                    count_loops += 1;
                    let count = output_file_looping
                        .write(&machine_index_bytes_be)
                        .expect("ok");
                    assert!(count == machine_index_bytes_be.len());
                }
                Err(MayHalt) => {
                    count_undecided += 1;
                    let count = output_file_halting
                        .write(&machine_index_bytes_be)
                        .expect("ok");
                    assert!(count == machine_index_bytes_be.len());
                }
            }

            if count_processed % 100 == 0 {
                println!(
                    "processed {} :: {}% are looping",
                    count_processed,
                    count_loops * 100 / count_processed
                );
            }
        }

        println!("done");
        println!(" - total:      {count_processed:>8}");
        println!(" - loops:      {count_loops:>8}");
        println!(" - undecided:  {count_undecided:>8}");

        let elapsed = start_time.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    } else {
        match classify_fn(
            &Program::from_string(&args.machine),
            args.radius,
            args.max_context_count,
        ) {
            Ok(LoopsForever) => {
                println!("{} loops forever", args.machine);
            }
            Err(MayHalt) => {
                println!("{} may halt", args.machine);
            }
        }
    }
    Ok(())
}
