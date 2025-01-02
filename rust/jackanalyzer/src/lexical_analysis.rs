// Lexical analysis package
// コンピュータシステムの理論と実装 §10, §11

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub struct Lexicon {
    pub token: String,
    pub token_type: String,
    pub row_number: usize,
}

impl Lexicon {
    fn new(token: String, token_type: String, row_number: usize) -> Self {
        Lexicon {token, token_type, row_number}
    }

    pub fn lexical_analysis(path: &PathBuf) -> Vec<Lexicon> {
        let filename = path.file_name().expect("error: invalid filename").to_str().expect("error: invalid filename");
        let jackfile = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
            Ok(file) => file,
        };

        let mut lexicon_vec = vec![];
        let mut comment_handler = false;
        for (row_num, line) in BufReader::new(jackfile).lines().enumerate() {
            let line_unwrapped = match line {
                Err(why) => panic!("error: {}: couldn't read line {} in {}", why, row_num, filename),
                Ok(content) => content,
            };

            let line_preprocessed = Self::scanning_preprocess(line_unwrapped.trim(), &mut comment_handler);
            let lexes_in_line = Self::scanning_main_process(&line_preprocessed, &mut comment_handler, row_num, filename);
            for lex in lexes_in_line {
                lexicon_vec.push(lex);
            }
        }
        return lexicon_vec;
    }

    // scanning by finite automaton algorithm
    fn scanning_main_process(pre_scan: &String, comment_handler: &mut bool, row_number: usize, filename: &str) -> Vec<Lexicon> {
        let symbols = vec!['{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<', '>', '=', '~'];
        let mut tokens_vec = vec![];
        let mut token = "".to_string();
        let mut state = 0;
        for c in pre_scan.chars() {
            if state == 0 {
                // state normal
                if c == '/' {
                    if token.len() != 0 { 
                        tokens_vec.push(Self::assign_lex_type(token, row_number));
                    }
                    token = "".to_string();
                    state = 1;
                } else if c == '"' {
                    if token.len() != 0 { 
                        tokens_vec.push(Self::assign_lex_type(token, row_number));
                    }
                    token = "".to_string();
                    state = 2;
                } else if c == '\\' {
                    if token.len() != 0 { 
                        tokens_vec.push(Self::assign_lex_type(token, row_number));
                    }
                    token = "".to_string();
                    state = 5;
                } else if c == ' ' {
                    if token.len() != 0 { 
                        tokens_vec.push(Self::assign_lex_type(token, row_number));
                    }
                    token = "".to_string();
                } else if symbols.contains(&c) {
                    if token.len() != 0 { 
                        tokens_vec.push(Self::assign_lex_type(token, row_number));
                    }
                    tokens_vec.push(Self::new(c.to_string(), "symbol".to_string(), row_number));
                    token = "".to_string();
                } else {
                    token = format!("{}{}", token, c);
                }
            } else if state == 1 {
                // state '/'
                if c == '/' {
                    state = 0;
                    break;
                } else if c == '"' {
                    tokens_vec.push(Self::new('/'.to_string(), "symbol".to_string(), row_number));
                    state = 2;
                } else if c == '*' {
                    state = 3;
                    *comment_handler = true;
                } else if c == '\\' {
                    tokens_vec.push(Self::new('/'.to_string(), "symbol".to_string(), row_number));
                    state = 5;
                } else if c == ' ' {
                    tokens_vec.push(Self::new('/'.to_string(), "symbol".to_string(), row_number));
                    state = 0;
                } else if symbols.contains(&c) {
                    tokens_vec.push(Self::new('/'.to_string(), "symbol".to_string(), row_number));
                    tokens_vec.push(Self::new(c.to_string(), "symbol".to_string(), row_number));
                    state = 0;
                } else {
                    tokens_vec.push(Self::new('/'.to_string(), "symbol".to_string(), row_number));
                    token = format!("{}{}", token, c);
                    state = 0;
                }
            } else if state == 2 {
                // state '"'
                if c == '"' {
                    tokens_vec.push(Self::new(token, "string_constant".to_string(), row_number));
                    token = "".to_string();
                    state = 0;
                } else {
                    token = format!("{}{}", token, c);
                }
            } else if state == 3 {
                // state '/*'
                if c == '*' {
                    state = 4;
                }
            } else if state == 4 {
                // state '/* *'
                if c == '/' {
                    state = 0;
                    *comment_handler = false;
                } else if c != '*' {
                    state = 3;
                }
            } else if state == 5 {
                // state '\'
                if c == 't' {
                    state = 0;
                } else {
                    panic!("syntax error: escape sequences are not supported except for \\n and \\t: in {} line {}", filename, row_number);
                }
            }
        }
        if state == 1 || state == 2 || state == 5 {
            panic!("syntax error: in {} line {}", filename, row_number)
        }
        return tokens_vec;
    }

    // ignore /* */ if comment_handler is true
    fn scanning_preprocess(line: &str, comment_handler: &mut bool) -> String {
        if *comment_handler {
            let mut new_line = "".to_string();
            let mut star_flag = false;
            let mut comment_end_flag = false;
            for c in line.chars() {
                if comment_end_flag {
                    new_line = format!("{}{}", new_line, c);
                } else {
                    if star_flag {
                        if c == '/' {
                            comment_end_flag = true;
                            *comment_handler = false;
                        } else if c != '*' {
                            star_flag = false;
                        }
                    } else {
                        if c == '*' {
                            star_flag = true;
                        }
                    }
                }
            }
            return new_line;
        } else {
            return line.to_string();
        }
    }

    fn assign_lex_type(token: String, row_number: usize) -> Lexicon {
        let keyword = vec!["class".to_string(), "constructor".to_string(), "function".to_string(), "method".to_string(),
                           "field".to_string(), "static".to_string(), "var".to_string(), "int".to_string(), "char".to_string(), 
                           "boolean".to_string(), "void".to_string(), "true".to_string(), "false".to_string(), "null".to_string(),
                           "this".to_string(), "let".to_string(), "do".to_string(), "if".to_string(), "else".to_string(), 
                           "while".to_string(), "return".to_string()];
        if token.len() != 0 {
            if keyword.contains(&token) {
                return Self::new(token, "keyword".to_string(), row_number);
            } else if let Ok(_) = token.parse::<i64>() {
                return Self::new(token, "integer_constant".to_string(), row_number);
            } else {
                return Self::new(token, "identifier".to_string(), row_number);
            }
        } else {
            panic!("program error: invalid function call: in assign_lex_type");
        }
    }

    pub fn lex_to_xml(lex: &Lexicon) -> String {
        if lex.token_type == "keyword".to_string() {
            return format!("<keyword> {} </keyword>", lex.token);
        } else if lex.token_type == "symbol".to_string() {
            if lex.token == "<" {
                return "<symbol> &lt; </symbol>".to_string();
            } else if lex.token == ">" {
                return "<symbol> &gt; </symbol>".to_string();
            } else if lex.token == "&" {
                return "<symbol> &amp; </symbol>".to_string();
            } else {
                return format!("<symbol> {} </symbol>", lex.token);
            }
        } else if lex.token_type == "integer_constant".to_string() {
            return format!("<integerConstant> {} </integerConstant>", lex.token);
        } else if lex.token_type == "string_constant".to_string() {
            return format!("<stringConstant> {} </stringConstant>", lex.token);
        } else if lex.token_type == "identifier".to_string() {
            return format!("<identifier> {} </identifier>", lex.token);
        } else {
            panic!("program error: invalid function call: in lex_to_xml");
        }
    }
}