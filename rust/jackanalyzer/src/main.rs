// Jack lang's compiler
// コンピュータシステムの理論と実装 §10, §11

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod lexical_analysis;
use crate::lexical_analysis::Lexicon;

mod parser;
use crate::parser::parse_class;

mod parser_to_xml;

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
                        parser(&path_of_entry);
//                        tokenize_to_xml(&path_of_entry);
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
            parser(&path);
//            tokenize_to_xml(&path);
        } else {
            panic!("error: invalid path: input ./jackanalyzer path/to/foo.jack");
        }
    } else {
        panic!("error: invalid path: input ./jackanalyzer path/to/foo.jack");
    }
}


fn write_to_xmlfile(path: &PathBuf, contents: String) {
    let mut new_path = path.clone();
    new_path.set_extension("xml");
    let mut xmlfile = match File::create(&new_path) {
        Err(why) => panic!("couldn't create {}: {}", new_path.display(), why),
        Ok(file) => file,
    };
    writeln!(xmlfile, "{}", contents).expect("couldn't write to file");
}

// functions for unit test 
fn tokenize_to_xml(path: &PathBuf) {
    let lex_vec = Lexicon::lexical_analysis(path);
    let mut contents = "<tokens>\n".to_string();
    for lex in lex_vec {
        contents += &Lexicon::lex_to_xml(&lex);
        contents += "\n";
    }
    contents += "</tokens>";
    write_to_xmlfile(path, contents);
}

fn parser(path: &PathBuf) {
    let lex_vec = Lexicon::lexical_analysis(path);
    let filename = path.file_name().expect("error: invalid filename").to_str().expect("error: invalid filename");
    let contents = parser_to_xml::parse_class(&lex_vec, filename);
    write_to_xmlfile(path, contents);
}

fn write_to_xmlfile(path: &PathBuf, contents: String) {
    let mut new_path = path.clone();
    new_path.set_extension("xml");
    let mut xmlfile = match File::create(&new_path) {
        Err(why) => panic!("couldn't create {}: {}", new_path.display(), why),
        Ok(file) => file,
    };
    writeln!(xmlfile, "{}", contents).expect("couldn't write to file");
}