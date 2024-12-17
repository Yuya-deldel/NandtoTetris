use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::collections::HashMap;

fn main() {
    // get path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("input filename: ./hackassembler path/to/foo.asm");
    }
    let path = PathBuf::from(&args[1]);

    // open foo.asm file
    let asmfile = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };
//    let mut buf_reader = BufReader::new(asmfile);
//    let mut contents = String::new();
//    buf_reader.read_to_string(&mut contents).unwrap();

    // convert foo.asm to binary
    let asm_mid_code = MidAsmCode::asm_to_midcode(&asmfile);
    let binary_mid_code = MidAsmCode::midcode_to_binary(asm_mid_code);
    let binary_str = MidAsmCode::binary_to_Str(binary_mid_code);

    // write binary data to file
    let mut new_path = path.clone();
    new_path.set_extension("hack");
    let mut hackfile = match File::create(&new_path) {
        Err(why) => panic!("couldn't create {}: {}", new_path.display(), why),
        Ok(file) => file,
    };
    writeln!(hackfile, "{}", binary_str);
}


struct Row {
    new_row_num: usize,
    row_num: usize,
    code: String,
}

impl Row {
    fn new(new_row_num: usize, row_num: usize, code: &String) -> Self {
        Row {new_row_num, row_num, code: code.to_string()}
    }
}

struct MidAsmCode {
    row: Vec<Row>,
    var: HashMap<String, usize>,
}

impl MidAsmCode {
    fn new() -> Self {
        MidAsmCode {row: vec![], var: HashMap::new()}
    }

    fn asm_to_midcode(file: &File) -> MidAsmCode {
        let mut midcode = MidAsmCode::new();

        // register defined symbols
        let defined_symbols: HashMap<String, usize> = vec![
            ("R0".to_string(), 0),
            ("R1".to_string(), 1),
            ("R2".to_string(), 2),
            ("R3".to_string(), 3),
            ("R4".to_string(), 4),
            ("R5".to_string(), 5),
            ("R6".to_string(), 6),
            ("R7".to_string(), 7),
            ("R8".to_string(), 8),
            ("R9".to_string(), 9),
            ("R10".to_string(), 10),
            ("R11".to_string(), 11),
            ("R12".to_string(), 12),
            ("R13".to_string(), 13),
            ("R14".to_string(), 14),
            ("R15".to_string(), 15),
            ("SP".to_string(), 0),
            ("LCL".to_string(), 1),
            ("ARG".to_string(), 2),
            ("THIS".to_string(), 3),
            ("THAT".to_string(), 4),
            ("SCREEN".to_string(), 16384),
            ("KBD".to_string(), 24576),
        ].into_iter().collect();
        midcode.var = defined_symbols;

        let mut new_row_num = 0;
        for (row_num, line) in BufReader::new(file).lines().enumerate() {
            let unwraped_line = line.unwrap(); 
            let line_vec: Vec<&str> = unwraped_line.trim().split(' ').collect();
            let words_num = line_vec.len();
            let mut comment_error_handle = true;

            if words_num != 0 {
                let line_length = line_vec[0].chars().count();
                if line_length == 0 {
                    // skip blank line: do nothing
                } else if line_length == 1 {
                    eprintln!("warning: meaningless C-type order was skipped: in line {}", row_num);
                } else if line_vec[0].chars().nth(0).unwrap() == '/' && line_vec[0].chars().nth(1).unwrap() == '/' {
                    // skip comments
                    comment_error_handle = false;
                } else if line_vec[0].chars().nth(0).unwrap() == '(' && line_vec[0].chars().last().unwrap() == ')' {
                    // register label variables
                    if line_length == 2 {
                        panic!("syntax error: invalid label: in line {}", row_num);
                    } else {
                        let label: String = line_vec[0].chars().skip(1).take(line_length - 2).collect();
                        midcode.var.insert(label, new_row_num);
                    }
                } else {
                    // assign new row_number to A or C type code
                    let new_row = Row::new(new_row_num, row_num, &line_vec[0].to_string());
                    midcode.row.push(new_row);
                    new_row_num += 1;
                }
            }

            if words_num >= 2 {
                if !(line_vec[1].chars().nth(0).unwrap() == '/' && line_vec[1].chars().nth(1).unwrap() == '/') {
                    if comment_error_handle {
                        eprintln!("warning: some codes after blank will be ignored: in line {}", row_num);
                    }
                }
            }
        }
        return midcode;
    }

    fn midcode_to_binary(midcode: MidAsmCode) -> MidAsmCode {
        let mut binary = MidAsmCode::new();
        binary.var = midcode.var;
        let mut new_variable_counter = 16;

        for line in midcode.row {
            if let Some('@') = line.code.chars().nth(0) {
                // A-type code: relating value and variables
                let variable: String = line.code.chars().skip(1).collect();
                let mut value = 0;
                if let Ok(val) = variable.parse::<usize>() {
                    // when input string is number(usize)
                    value = val;
                } else {
                    if let Some(val) = binary.var.get(&variable) {
                        // when variable is already registered
                        value = *val;
                    } else {
                        // when variable is not registered
                        value = new_variable_counter;
                        binary.var.insert(variable, value);
                        new_variable_counter += 1;
                    }
                }

                if new_variable_counter >= 1 << 15 {
                    panic!("warning: over flow: too many variables");
                }

                let new_row = Row::new(line.new_row_num, line.row_num, &format!("{:>016b}", value));
                binary.row.push(new_row);

            } else {
                // C-type code
                // dividing C-type code to dest/comp/jump order
                let mut dest_code = "".to_string();
                let mut comp_code = "".to_string();
                let mut jump_code = "".to_string();
                let mut temp = "".to_string();
                if let Some(position_eq) = line.code.chars().position(|c| c == '=') {
                    if position_eq == 0 || position_eq == line.code.chars().count() - 1 {
                        panic!("syntax error: invalid C-type order: in line {}", line.row_num);
                    } else {
                        dest_code = line.code.chars().take(position_eq).collect();
                        temp = line.code.chars().skip(position_eq + 1).collect();
                    }
                } else {
                    temp = line.code;
                }
                if let Some(position_semicolon) = temp.chars().position(|c| c == ';') {
                    if position_semicolon == 0 || position_semicolon == temp.chars().count() - 1 {
                        panic!("syntax error: invalid C-type order: in line {}", line.row_num);
                    } else {
                        comp_code = temp.chars().take(position_semicolon).collect();
                        jump_code = temp.chars().skip(position_semicolon + 1).collect();
                    }
                } else {
                    comp_code = temp;
                }

                // converting dest/comp/jump to binary and conbinding them
                let converted = c_order_to_binary(&comp_code, &dest_code, &jump_code, line.row_num);
                let new_row = Row::new(line.new_row_num, line.row_num, &converted);
                binary.row.push(new_row);
            }
        }
        return binary;
    }

    fn binary_to_Str(midcode: MidAsmCode) -> String {
        let mut string: String = "".to_string();
        for line in midcode.row {
            string = format!("{}{}\n", string, line.code);
        }
        return string;
    }
}

fn c_order_to_binary(comp: &String, dest: &String, jump: &String, row_number: usize) -> String {
    let comp_binary = if comp == "0" {"0101010"}
                      else if comp == "1" {"0111111"}
                      else if comp == "-1" {"0111010"}
                      else if comp == "D" {"0001100"}
                      else if comp == "A" {"0110000"}
                      else if comp == "M" {"1110000"}
                      else if comp == "!D" {"0001101"}
                      else if comp == "!A" {"0110001"}
                      else if comp == "!M" {"1110001"}
                      else if comp == "-D" {"0001111"}
                      else if comp == "-A" {"0110011"}
                      else if comp == "-M" {"1110011"}
                      else if comp == "D+1" {"0011111"}
                      else if comp == "A+1" {"0110111"}
                      else if comp == "M+1" {"1110111"}
                      else if comp == "D-1" {"0001110"}
                      else if comp == "A-1" {"0110010"}
                      else if comp == "M-1" {"1110010"}
                      else if comp == "D+A" {"0000010"}
                      else if comp == "D+M" {"1000010"}
                      else if comp == "D-A" {"0010011"}
                      else if comp == "D-M" {"1010011"}
                      else if comp == "A-D" {"0000111"}
                      else if comp == "M-D" {"1000111"}
                      else if comp == "D&A" {"0000000"}
                      else if comp == "D&M" {"1000000"}
                      else if comp == "D|A" {"0010101"}
                      else if comp == "D|M" {"1010101"}
                      else {panic!("syntax error: invalid C-type order: in line {}", row_number)};

    let dest_binary = if dest == "" {"000"}
                      else if dest == "M" {"001"}
                      else if dest == "D" {"010"}
                      else if dest == "DM" || dest == "MD" {"011"}
                      else if dest == "A" {"100"}
                      else if dest == "AM" {"101"}
                      else if dest == "AD" {"110"}
                      else if dest == "ADM" {"111"}
                      else {panic!("syntax error: invalid C-type order: in line {}", row_number)};
    
    let jump_binary = if jump == "" {"000"}
                      else if jump == "JGT" {"001"}
                      else if jump == "JEQ" {"010"}
                      else if jump == "JGE" {"011"}
                      else if jump == "JLT" {"100"}
                      else if jump == "JNE" {"101"}
                      else if jump == "JLE" {"110"}
                      else if jump == "JMP" {"111"}
                      else {panic!("syntax error: invalid C-type order: in line {}", row_number)};
    
    return format!("111{}{}{}", comp_binary, dest_binary, jump_binary);
}