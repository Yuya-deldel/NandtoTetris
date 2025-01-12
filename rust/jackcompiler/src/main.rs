// jack lang's compiler
// コンピュータシステムの理論と実装 §10, §11

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod lexical_analysis;
use crate::lexical_analysis::Lexicon;

mod parser;
use crate::parser::compile_starter;

fn main() {
    // get path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("input path: ./jackanalyzer path/to/dir || path/to/foo.jack");
    }
    let path = PathBuf::from(&args[1]);

    if path.is_dir() {
        // parse all .jack file in the directory
        let directory = path.read_dir().expect("couldn't open the directory");
        for dir_entry in directory {
            if let Ok(entry) = dir_entry {
                let path_of_entry = entry.path();
                if let Some(extension) = path_of_entry.extension() {
                    if extension == "jack" {
                        compiler(&path_of_entry);
                    }
                }
            } else {
                eprintln!("warning: couldn't access to some entry");
                continue;
            }
        }
    } else if let Some(extension) = path.extension() {
        // parse .jack file
        if extension == "jack" {
            compiler(&path)
        } else {
            panic!("error: invalid path: input ./jackanalyzer path/to/foo.jack");
        }
    } else {
        panic!("error: invalid path: input ./jackanalyzer path/to/foo.jack");
    }
}

fn compiler(path: &PathBuf) {
    let lex_vec = Lexicon::lexical_analysis(path);
    let filename = path.file_name().expect("error: invalid filename").to_str().expect("error: invalid filename");
    let contents = compile_starter(lex_vec, filename);
    write_to_vmfile(path, contents);
}

fn write_to_vmfile(path: &PathBuf, contents: String) {
    let mut new_path = path.clone();
    new_path.set_extension("vm");
    let mut vmfile = match File::create(&new_path) {
        Err(why) => panic!("couldn't create {}: {}", new_path.display(), why),
        Ok(file) => file,
    };
    writeln!(vmfile, "{}", contents).expect("couldn't write to file");
}
