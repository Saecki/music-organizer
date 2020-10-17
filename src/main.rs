use clap::{App, Arg, Shell};
use music_organizer::{Changes, FileOpType, FileOperation, MusicIndex, ReadMusicIndexIter, Song};
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

static mut LAST_LEN: usize = 0;

fn main() {
    let mut app = App::new("music organizer")
        .version("0.1.0")
        .author("Saecki")
        .about("Moves or copies and renames Music files using their metadata information.")
        .arg(
            Arg::with_name("music-dir")
                .short("m")
                .long("music-dir")
                .help("The directory which will be searched for music files")
                .takes_value(true)
                .required_unless("generate-completion")
                .conflicts_with("generate-completion"),
        )
        .arg(
            Arg::with_name("output-dir")
                .short("o")
                .long("output-dir")
                .help("The directory which the content will be written to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("copy")
                .short("c")
                .long("copy")
                .help("Copy the files instead of moving")
                .requires("output-dir")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("assume-yes")
                .short("y")
                .long("assume-yes")
                .help("Assumes yes as a answer for all questions")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .long("verbosity")
                .help("Verbosity level of the output. 0 means least 2 means most verbose ouput.")
                .takes_value(true)
                .possible_values(&["0", "1", "2"]),
        )
        .arg(
            Arg::with_name("generate-completion")
                .short("g")
                .long("generate-completion")
                .value_name("shell")
                .help("Generates a completion script for the specified shell")
                .conflicts_with("music-dir")
                .requires("output-dir")
                .takes_value(true)
                .possible_values(&["bash", "zsh", "fish", "elvish", "powershell"]),
        );

    let matches = app.clone().get_matches();
    let generate_completion = matches.value_of("generate-completion").unwrap_or("");

    if generate_completion != "" {
        let output_dir = PathBuf::from(matches.value_of("output-dir").unwrap());
        if !output_dir.exists() {
            match std::fs::create_dir_all(&output_dir) {
                Ok(_) => println!("created dir: {}", output_dir.display()),
                Err(e) => println!("error creating dir: {}\n{}", output_dir.display(), e),
            }
        }

        println!("generating completions...");
        let shell = Shell::from_str(generate_completion).unwrap();
        app.gen_completions("music_organizer", shell, output_dir);
        println!("done");
        exit(0);
    }

    let music_dir = PathBuf::from(matches.value_of("music-dir").unwrap());
    let verbosity = matches
        .value_of("verbosity")
        .map(|v| v.parse::<usize>().unwrap())
        .unwrap_or(0);
    let operation = FileOpType::from(matches.is_present("copy"));
    let yes = matches.is_present("assume-yes");

    let abs_music_dir = match PathBuf::from(&music_dir).canonicalize() {
        Ok(t) => t,
        Err(e) => {
            println!(
                "Not a valid music dir path: {}\n{:?}",
                music_dir.display(),
                e
            );
            exit(1)
        }
    };

    let output_dir = match matches.value_of("output-dir") {
        Some(s) => PathBuf::from(s),
        None => abs_music_dir.clone(),
    };

    println!("indexing...");
    let mut index = MusicIndex::from(music_dir);

    for (i, s) in &mut index.read_iter().enumerate() {
        print_verbose(
            &format!("{} {} - {}", i + 1, &s.artist, &s.title),
            verbosity >= 2,
        );
    }
    println!();

    println!("changes...");
    let changes = Changes::from(&index, output_dir);

    if changes.dir_creations.is_empty() && changes.file_moves.is_empty() {
        println!("nothing to do exiting...");
        return;
    }

    if !yes {
        if verbosity >= 1 {
            if !changes.dir_creations.is_empty() {
                println!("dirs:");
                for (i, d) in changes.dir_creations.iter().enumerate() {
                    println!("{} {}", i + 1, d.path.display());
                }
                println!();
            }
            if !changes.file_moves.is_empty() {
                println!("files:");
                for (i, f) in changes.file_moves.iter().enumerate() {
                    println!("{} {}", i + 1, f.new.display());
                }
                println!();
            }
        }

        let ok = input_confirmation_loop(&format!(
            "{} dirs will be created.\n{} files will be {}.\n Continue",
            changes.dir_creations.len(),
            changes.file_moves.len(),
            match operation {
                FileOpType::Copy => "copied",
                FileOpType::Move => "moved",
            }
        ));

        if !ok {
            println!("exiting...");
            exit(1);
        }
    }

    println!("\nwriting...");
    for e in changes.write(operation) {
        println!("Error: {}", e);
    }
}

#[inline]
fn print_verbose(str: &str, verbose: bool) {
    if verbose {
        println!("{}", str);
    } else {
        let len = str.chars().count();
        let diff = unsafe { LAST_LEN as i32 - len as i32 };

        print!("\r{}", str);
        for _ in 0..diff {
            print!(" ");
        }
        let _ = std::io::stdout().flush().is_ok();

        unsafe {
            LAST_LEN = len;
        }
    }
}

fn input_loop(str: &str, predicate: fn(&str) -> bool) -> String {
    let mut input = String::with_capacity(10);

    loop {
        println!("{}", str);

        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                if predicate(&input) {
                    return input;
                }
            }
            Err(e) => println!("error:\n {}", e),
        }
    }
}

fn input_options_loop(options: &[&str]) -> usize {
    let mut input = String::with_capacity(2);

    loop {
        for (i, s) in options.iter().enumerate() {
            println!("[{}] {}", i, s);
        }

        match std::io::stdin().read_line(&mut input) {
            Ok(_) => match usize::from_str(input.trim_matches('\n')) {
                Ok(i) => {
                    if i < options.len() {
                        return i;
                    } else {
                        println!("invalid input")
                    }
                }
                Err(_) => println!("invalid input"),
            },
            Err(e) => println!("error:\n {}", e),
        }
    }
}

fn input_confirmation_loop(str: &str) -> bool {
    let mut input = String::with_capacity(2);

    loop {
        print!("{} [y/N]?", str);
        let _ = std::io::stdout().flush().is_ok();

        if let Err(e) = std::io::stdin().read_line(&mut input) {
            println!("error:\n {}", e);
        } else {
            input.make_ascii_lowercase();

            if input == "\n" || input == "n\n" {
                return false;
            } else if input == "y\n" {
                return true;
            } else {
                println!("invalid input");
            }
        }
    }
}
