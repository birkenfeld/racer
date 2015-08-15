#![cfg_attr(all(test, feature = "nightly"), feature(test))] // we only need test feature when testing

#[macro_use] extern crate log;

extern crate syntex_syntax;
extern crate toml;
extern crate env_logger;
#[macro_use]
extern crate clap;

extern crate racer;

#[cfg(not(test))]
use racer::core;
#[cfg(not(test))]
use racer::util;
#[cfg(not(test))]
use racer::core::Match;
#[cfg(not(test))]
use racer::util::{getline, path_exists};
#[cfg(not(test))]
use racer::nameres::{do_file_search, do_external_search, PATH_SEP};
#[cfg(not(test))]
use racer::scopes;
#[cfg(not(test))]
use std::path::Path;
#[cfg(not(test))]
use clap::{App, Arg, ArgMatches, SubCommand};

#[cfg(not(test))]
fn match_with_snippet_fn(m: Match, session: &core::Session) {
    if let Some((linenum, charnum)) = scopes::point_to_coords_from_file(&m.filepath,
                                                                        m.point,
                                                                        session) {
        if m.matchstr == "" {
            panic!("MATCHSTR is empty - waddup?");
        }

        let snippet = racer::snippets::snippet_for_match(&m, session);
        println!("MATCH {};{};{};{};{};{:?};{}", m.matchstr,
                                        snippet,
                                        linenum.to_string(),
                                        charnum.to_string(),
                                        m.filepath.to_str().unwrap(),
                                        m.mtype,
                                        m.contextstr
                 );
    } else {
        error!("Could not resolve file coords for match {:?}", m);
    }
}

#[cfg(not(test))]
fn match_fn(m: Match, session: &core::Session) {
    if let Some((linenum, charnum)) = scopes::point_to_coords_from_file(&m.filepath,
                                                                        m.point,
                                                                        session) {
        println!("MATCH {},{},{},{},{:?},{}", m.matchstr,
                                    linenum.to_string(),
                                    charnum.to_string(),
                                    m.filepath.to_str().unwrap(),
                                    m.mtype,
                                    m.contextstr
                 );
    } else {
        error!("Could not resolve file coords for match {:?}", m);
    }
}

#[cfg(not(test))]
fn complete(match_found: &Fn(Match, &core::Session), args: &ArgMatches) {
    match args.value_of("fqn or linenum").unwrap().parse::<usize>() {
        Ok(linenum) => {
            let charnum = value_t_or_exit!(args.value_of("charnum"), usize);
            let fpath = Path::new(args.value_of("fname").unwrap());
            let substitute_file = args.value_of("substitute_file").map_or(fpath, Path::new);
            let session = core::Session::from_path(&fpath, &substitute_file);
            let src = session.load_file(&fpath);
            let line = &getline(&substitute_file, linenum, &session);
            let (start, pos) = util::expand_ident(line, charnum);
            println!("PREFIX {},{},{}", start, pos, &line[start..pos]);

            let point = scopes::coords_to_point(&src, linenum, charnum);
            for m in core::complete_from_file(&src, &fpath, point, &session) {
                match_found(m, &session);
            }
            println!("END");
        }
        Err(_) => {
            // input: a command line string passed in
            let arg = args.value_of("fqn or linenum").unwrap();
            let it = arg.split("::");
            let p: Vec<&str> = it.collect();
            let session = core::Session::from_path(&Path::new("."), &Path::new("."));

            for m in do_file_search(p[0], &Path::new(".")) {
                if p.len() == 1 {
                    match_found(m, &session);
                } else {
                    for m in do_external_search(&p[1..], &m.filepath, m.point,
                                                core::SearchType::StartsWith,
                                                core::Namespace::BothNamespaces, &session) {
                        match_found(m, &session);
                    }
                }
            }
        }
    }
}

#[cfg(not(test))]
fn prefix(args: &ArgMatches) {
    let linenum = value_t_or_exit!(args.value_of("linenum"), usize);
    let charnum = value_t_or_exit!(args.value_of("charnum"), usize);
    let fpath = Path::new(args.value_of("fname").unwrap());
    let substitute_file = args.value_of("substitute_file").map_or(fpath, Path::new);

    // print the start, end, and the identifier prefix being matched
    let session = core::Session::from_path(&fpath, &substitute_file);
    let line = &getline(&fpath, linenum, &session);
    let (start, pos) = util::expand_ident(line, charnum);
    println!("PREFIX {},{},{}", start, pos, &line[start..pos]);
}

#[cfg(not(test))]
fn find_definition(args: &ArgMatches) {
    let linenum = value_t_or_exit!(args.value_of("linenum"), usize);
    let charnum = value_t_or_exit!(args.value_of("charnum"), usize);
    let fpath = Path::new(args.value_of("fname").unwrap());
    let substitute_file = args.value_of("substitute_file").map_or(fpath, Path::new);
    let session = core::Session::from_path(&fpath, &substitute_file);
    let src = session.load_file(&fpath);
    let pos = scopes::coords_to_point(&src, linenum, charnum);

    core::find_definition(&src, &fpath, pos, &session).map(|m| match_fn(m, &session));
    println!("END");
}

#[cfg(not(test))]
fn check_rust_src_env_var() {
    if let Ok(srcpaths) = std::env::var("RUST_SRC_PATH") {
        let v = srcpaths.split(PATH_SEP).collect::<Vec<_>>();
        if !v.is_empty() {
            let f = Path::new(v[0]);
            if !path_exists(f) {
                println!("racer can't find the directory pointed to by the RUST_SRC_PATH variable \"{}\". Try using an absolute fully qualified path and make sure it points to the src directory of a rust checkout - e.g. \"/home/foouser/src/rust/src\".", srcpaths);
                std::process::exit(1);
            } else if !path_exists(f.join("libstd")) {
                println!("Unable to find libstd under RUST_SRC_PATH. N.B. RUST_SRC_PATH variable needs to point to the *src* directory inside a rust checkout e.g. \"/home/foouser/src/rust/src\". Current value \"{}\"", srcpaths);
                std::process::exit(1);
            }
        }
    } else {
        println!("RUST_SRC_PATH environment variable must be set to point to the src directory of a rust checkout. E.g. \"/home/foouser/src/rust/src\"");
        std::process::exit(1);
    }
}

#[cfg(not(test))]
fn daemon() {
    use std::io;
    let mut input = String::new();
    while let Ok(n) = io::stdin().read_line(&mut input) {
        if n == 0 {
            break;
        }
        let matches = build_cli().get_matches_from(input.split(" ").map(|s| s.trim().to_string()));
        run(matches);

        input.clear();
    }
}

fn build_cli<'a, 'b, 'c, 'd, 'e, 'f>() -> App<'a, 'b, 'c, 'd, 'e, 'f> {
    App::new("racer")
        .version("v1.0.0")
        .author("Phil Dawes")
        .about("A Rust code completion utility")
        .subcommand_required(true)
        .global_version(true)
        .subcommand(SubCommand::with_name("complete")
            .about("performs completion and returns matches")
            .arg_from_usage("<fqn or linenum>  'complete with a fully-qualified-name (e.g. std::io::) \
                             or line number'")
            .arg(Arg::from_usage("[charnum]    'The char number'")
                .requires("fname"))
            .arg_from_usage("[fname]           'The function name to match'")
            .arg_from_usage("[substitute_file] 'An optional substitute file'"))
        .subcommand(SubCommand::with_name("daemon")
            .about("start a process that receives the above commands via stdin"))
        .subcommand(SubCommand::with_name("find-definition")
            .about("finds the definition of a function")
            .args_from_usage("<linenum> 'The line number'
                              <charnum> 'The char number'
                              <fname>   'The function name to match'
                              [substitute_file] 'An optional substitute file'"))
        .subcommand(SubCommand::with_name("prefix")
            .args_from_usage("<linenum> 'The line number'
                              <charnum> 'The char number'
                              <fname>   'The function name to match'"))
        .subcommand(SubCommand::with_name("complete-with-snippet")
            .about("performs completion and returns more detailed matches")
            .arg_from_usage("<fqn or linenum>  'complete with a fully-qualified-name (e.g. std::io::) \
                             or line number'")
            .arg(Arg::from_usage("[charnum]    'The char number'")
                .requires("fname"))
            .arg_from_usage("[fname]           'The function name to match'")
            .arg_from_usage("[substitute_file] 'An optional substitute file'"))
        .after_help("For more information about a specific command try 'racer <command> --help'")
}

#[cfg(not(test))]
fn run(m: ArgMatches) {
    // match raw subcommand, and get it's matches
    match m.subcommand() {
        ("daemon", _)                   => daemon(),
        ("prefix", Some(sub_matches))   => prefix(&sub_matches),
        ("complete", Some(sub_matches)) => complete(&match_fn, &sub_matches),
        ("complete-with-snippet", Some(sub_matches)) => complete(&match_with_snippet_fn, &sub_matches),
        ("find-definition", Some(sub_matches))       => find_definition(&sub_matches),
        _ => unreachable!()
    }
}

#[cfg(not(test))]
fn main() {
    // make sure we get a stack trace ifwe panic
    ::std::env::set_var("RUST_BACKTRACE","1");
    env_logger::init().unwrap();

    check_rust_src_env_var();

    let matches = build_cli().get_matches();
    run(matches);
}
