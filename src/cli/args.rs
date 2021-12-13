use clap::{crate_authors, crate_version, App, Arg, ColorChoice, ValueHint};
use clap_generate::generate;
use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};
use music_organizer::FileOpType;
use std::path::PathBuf;
use std::process::exit;

const BIN_NAME: &str = "music-organizer";

const BASH: &str = "bash";
const ELVISH: &str = "elvish";
const FISH: &str = "fish";
const PWRSH: &str = "powershell";
const ZSH: &str = "zsh";

pub struct Args {
    pub music_dir: PathBuf,
    pub output_dir: PathBuf,
    pub verbosity: usize,
    pub op_type: FileOpType,
    pub assume_yes: bool,
    pub dry_run: bool,
    pub no_check: bool,
    pub keep_embedded_artworks: bool,
    pub no_cleanup: bool,
}

pub fn parse_args() -> Args {
    let mut app = App::new("music organizer")
        .color(ColorChoice::Auto)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Moves/copies, renames and retags Music files using their metadata.")
        .arg(
            Arg::new("music-dir")
                .short('m')
                .long("music-dir")
                .help("The directory which will be searched for music files")
                .takes_value(true)
                .default_value("~/Music")
                .value_hint(ValueHint::DirPath),
        )
        .arg(
            Arg::new("output-dir")
                .short('o')
                .long("output-dir")
                .help("The directory which the content will be written to")
                .takes_value(true)
                .value_hint(ValueHint::DirPath),
        )
        .arg(
            Arg::new("copy")
                .short('c')
                .long("copy")
                .help("Copy the files instead of moving")
                .requires("output-dir"),
        )
        .arg(
            Arg::new("nocheck")
                .short('n')
                .long("nocheck")
                .help("Don't check for inconsistencies")
                .takes_value(false),
        )
        .arg(
            Arg::new("keep embedded artworks")
                .short('e')
                .long("keep-embedded-artworks")
                .help("Keep embedded artworks")
                .takes_value(false),
        )
        .arg(
            Arg::new("nocleanup")
                .long("nocleanup")
                .help("Don't remove empty directories")
                .takes_value(false),
        )
        .arg(
            Arg::new("assume-yes")
                .short('y')
                .long("assume-yes")
                .help("Assumes yes as a answer for questions")
                .takes_value(false),
        )
        .arg(
            Arg::new("dryrun")
                .short('d')
                .long("dryrun")
                .help("Only check files don't change anything")
                .takes_value(false)
                .conflicts_with("assume-yes"),
        )
        .arg(
            Arg::new("verbosity")
                .short('v')
                .long("verbosity")
                .value_name("level")
                .help("Verbosity level of the output. 0 means least 2 means most verbose ouput.")
                .takes_value(true)
                .possible_values(&["0", "1", "2"])
                .default_value("1"),
        )
        .arg(
            Arg::new("generate-completion")
                .short('g')
                .long("generate-completion")
                .value_name("shell")
                .help("Generates a completion script for the specified shell")
                .conflicts_with("music-dir")
                .takes_value(true)
                .possible_values(&[BASH, ZSH, FISH, ELVISH, PWRSH]),
        );

    let matches = app.clone().get_matches();

    let generate_completion = matches.value_of("generate-completion");
    if let Some(shell) = generate_completion {
        let mut stdout = std::io::stdout();
        match shell {
            BASH => generate(Bash, &mut app, BIN_NAME, &mut stdout),
            ELVISH => generate(Elvish, &mut app, BIN_NAME, &mut stdout),
            FISH => generate(Fish, &mut app, BIN_NAME, &mut stdout),
            ZSH => generate(Zsh, &mut app, BIN_NAME, &mut stdout),
            PWRSH => generate(PowerShell, &mut app, BIN_NAME, &mut stdout),
            _ => unreachable!(),
        }
        exit(0);
    }

    let music_dir = {
        let dir = shellexpand::tilde(matches.value_of("music-dir").unwrap());
        let path = PathBuf::from(dir.as_ref());
        if !path.exists() {
            println!("Not a valid music dir path: {}", dir);
            exit(1)
        }
        path
    };

    let output_dir = match matches.value_of("output-dir") {
        Some(s) => {
            let dir = shellexpand::tilde(s);
            PathBuf::from(dir.as_ref())
        }
        None => music_dir.clone(),
    };

    Args {
        music_dir,
        output_dir,
        verbosity: matches.value_of("verbosity").map(|v| v.parse::<usize>().unwrap()).unwrap_or(1),
        op_type: match matches.is_present("copy") {
            true => FileOpType::Copy,
            false => FileOpType::Move,
        },
        assume_yes: matches.is_present("assume-yes"),
        no_check: matches.is_present("nocheck"),
        keep_embedded_artworks: matches.is_present("keep embedded artworks"),
        no_cleanup: matches.is_present("nocleanup"),
        dry_run: matches.is_present("dryrun"),
    }
}
