// converting VM to hack assembly lang
// コンピュータシステムの理論と実装 §7,8

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    // get path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("input filename: ./vmtranslator path/to/foo.vm");
    }
    let path = PathBuf::from(&args[1]);
    let filename = path.file_stem().expect("error: invalid filename").to_str().expect("error: invalid filename"); 
    
    // open foo.vm file
    let vmfile = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    // convert foo.vm to assembly lang
    let asm_string = vm_to_asm(&vmfile, filename);

    // write assembly lang to file
    let mut new_path = path.clone();
    new_path.set_extension("asm");
    let mut asmfile = match File::create(&new_path) {
        Err(why) => panic!("couldn't create {}: {}", new_path.display(), why),
        Ok(file) => file,
    };
    writeln!(asmfile, "{}", asm_string);
}

fn vm_to_asm(file: &File, filename: &str) -> String {
    let mut asm_string = "".to_string();
    let mut conditional_branch_count = 0;

    for (row_num, line) in BufReader::new(file).lines().enumerate() {
        let unwraped_line = line.unwrap();
        let line_vec: Vec<&str> = unwraped_line.trim().split(' ').collect();

        if line_vec.len() > 0 {
            let line_length = line_vec[0].chars().count();
            if line_length == 0 {
                // skip blank line: do nothing
            } else if line_length == 1 {
                panic!("syntax error: in line {}", row_num);
            } else if line_vec[0].chars().nth(0).unwrap() == '/' && line_vec[0].chars().nth(1).unwrap() == '/' {
                // skip comments: do nothing
            } else if line_vec[0] == "push" || line_vec[0] == "pop" {
                asm_string += &push_or_pop_to_asm(line_vec, filename, row_num);
            } else if line_vec[0] == "add" {
                asm_string += "// add\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D+M\n@SP\nA=M-1\nM=D\n";
            } else if line_vec[0] == "sub" {
                asm_string += "// sub\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D-M\n@SP\nA=M-1\nM=D\n";
            } else if line_vec[0] == "neg" {
                asm_string += "// neg\n@SP\nA=M-1\nM=-M\n";
            } else if line_vec[0] == "eq" {
                asm_string += &format!("// eq\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D-M\n@EQ{}TRUE\nD;JEQ\n@SP\nA=M-1\nM=0\n@EQ{}RESULT\n0;JMP\n(EQ{}TRUE)\n@SP\nA=M-1\nM=-1\n(EQ{}RESULT)\n", conditional_branch_count, conditional_branch_count, conditional_branch_count, conditional_branch_count);
                conditional_branch_count += 1;
            } else if line_vec[0] == "gt" {
                asm_string += &format!("// gt\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D-M\n@GT{}TRUE\nD;JGT\n@SP\nA=M-1\nM=0\n@GT{}RESULT\n0;JMP\n(GT{}TRUE)\n@SP\nA=M-1\nM=-1\n(GT{}RESULT)\n", conditional_branch_count, conditional_branch_count, conditional_branch_count, conditional_branch_count);
                conditional_branch_count += 1;
            } else if line_vec[0] == "lt" {
                asm_string += &format!("// lt\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D-M\n@LT{}TRUE\nD;JLT\n@SP\nA=M-1\nM=0\n@LT{}RESULT\n0;JMP\n(LT{}TRUE)\n@SP\nA=M-1\nM=-1\n(LT{}RESULT)\n", conditional_branch_count, conditional_branch_count, conditional_branch_count, conditional_branch_count);
                conditional_branch_count += 1;
            } else if line_vec[0] == "and" {
                asm_string += "// and\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D&M\n@SP\nA=M-1\nM=D\n";
            } else if line_vec[0] == "or" {
                asm_string += "// or\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D|M\n@SP\nA=M-1\nM=D\n";
            } else if line_vec[0] == "not" {
                asm_string += "// not\n@SP\nA=M-1\nM=!M\n";
            } else {
                panic!("syntax error: in line {}", row_num);
            }
        }
    }
    // infinite loop code at end of program 
    asm_string += "// end\n(ENDLOOP)\n@ENDLOOP\n0;JMP\n";
    return asm_string;
}

// subroutines
fn push_or_pop_to_asm(args: Vec<&str>, filename: &str, row_num: usize) -> String {
    let words_num = args.len();
    if words_num < 3 {
        panic!("syntax error: in line {}", row_num);
    } else if words_num > 3 {
        if args[3].chars().count() < 2 {
            eprintln!("warning: meaningless vm_code was skipped: in line {}", row_num);
        } else if !(args[3].chars().nth(0).unwrap() == '/' && args[3].chars().nth(1).unwrap() == '/') {
            eprintln!("warning: meaningless vm_code was skipped: in line {}", row_num);
        }
    }

    let arg2_num = match args[2].parse::<usize>() {
        Ok(int) => int,
        Err(_) => panic!("syntax error: in line {}", row_num),
    };
    
    if args[1] == "local" {
        if args[0] == "push" {
            return format!("// push local {}\n@LCL\nD=M\n@{}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop local {}\n@LCL\nD=M\n@{}\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in line {}", row_num);
        }
    } else if args[1] == "argument" {
        if args[0] == "push" {
            return format!("// push argument {}\n@ARG\nD=M\n@{}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop argument {}\n@ARG\nD=M\n@{}\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in line {}", row_num);
        }
    } else if args[1] == "this" {
        if args[0] == "push" {
            return format!("// push this {}\n@THIS\nD=M\n@{}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop this {}\n@THIS\nD=M\n@{}\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in line {}", row_num);
        }
    } else if args[1] == "that" {
        if args[0] == "push" {
            return format!("// push that {}\n@THAT\nD=M\n@{}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop that {}\n@THAT\nD=M\n@{}\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in line {}", row_num);
        }
    } else if args[1] == "pointer" {
        if arg2_num == 0 {
            if args[0] == "push" {
                return "// push pointer 0\n@THIS\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n".to_string();
            } else if args[0] == "pop" {
                return "// pop pointer 0\n@SP\nAM=M-1\nD=M\n@THIS\nM=D\n".to_string();
            } else {
                panic!("syntax error: in line {}", row_num);
            }
        } else if arg2_num == 1 {
            if args[0] == "push" {
                return "// push pointer 1\n@THAT\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n".to_string();
            } else if args[0] == "pop" {
                return "// pop pointer 1\n@SP\nAM=M-1\nD=M\n@THAT\nM=D\n".to_string();
            } else {
                panic!("syntax error: in line {}", row_num);
            }
        } else {
            panic!("syntax error: in line {}", row_num);
        }
    } else if args[1] == "temp" {
        if arg2_num >= 8 {
            panic!("syntax error: in line {}", row_num);
        }
        if args[0] == "push" {
            return format!("// push temp {}\n@{}\nD=A\n@5\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop temp {}\n@{}\nD=A\n@5\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in line {}", row_num);
        }
    } else if args[1] == "constant" {
        if args[0] == "push" {
            return format!("// push constant {}\n@{}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in line {}", row_num);
        }
    } else if args[1] == "static" {
        if arg2_num >= 240 {
            panic!("overflow: too many variables: in line {}", row_num);
        }
        if args[0] == "push" {
            return format!("// push static {}\n@{}.{}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, filename, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop static {}\n@SP\nAM=M-1\nD=M\n@{}.{}\nM=D\n", arg2_num, filename, arg2_num);
        } else {
            panic!("syntax error: in line {}", row_num);
        }
    } else {
        panic!("syntax error: in line {}", row_num);
    }
}