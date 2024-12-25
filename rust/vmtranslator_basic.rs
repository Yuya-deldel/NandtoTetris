// converting VM to hack assembly lang
// コンピュータシステムの理論と実装 §7,8: boot strap code の手前まで

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    // get path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("input path: ./vmtranslator_basic path/to/foo.vm");
    }
    let path = PathBuf::from(&args[1]);

    // convert foo.vm to assembly lang
    let asm_string = vm_to_asm(&path);

    // write assembly lang into file
    let mut new_path = path.clone();
    new_path.set_extension("asm");
    let mut asmfile = match File::create(&new_path) {
        Err(why) => panic!("couldn't create {}: {}", new_path.display(), why),
        Ok(file) => file,
    };
    writeln!(asmfile, "{}", asm_string).expect("couldn't write to file");
}

fn vm_to_asm(path: &PathBuf) -> String {
    let filename = path.file_name().expect("error: invalid filename").to_str().expect("error: invalid filename");
    let vmfile = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    let mut eq_gt_lt_count = 0;
    let mut return_address_count = 0;
    let mut asm_string = "".to_string();
    for (row_num, line) in BufReader::new(vmfile).lines().enumerate() {
        let unwraped_line = line.unwrap();
        let line_vec: Vec<&str> = unwraped_line.trim().split_whitespace().collect();

        if line_vec.len() > 0 {
            let line_length = line_vec[0].chars().count();
            if line_length == 0 {
                // skip blank line: do nothing
            } else if line_length == 1 {
                panic!("syntax error: in {} line {}", filename, row_num);
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
            } else if line_vec[0] == "eq" || line_vec[0] == "gt" || line_vec[0] == "lt" {
                asm_string += &eq_gt_lt_to_asm(line_vec[0], &mut eq_gt_lt_count);
            } else if line_vec[0] == "and" {
                asm_string += "// and\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D&M\n@SP\nA=M-1\nM=D\n";
            } else if line_vec[0] == "or" {
                asm_string += "// or\n@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D|M\n@SP\nA=M-1\nM=D\n";
            } else if line_vec[0] == "not" {
                asm_string += "// not\n@SP\nA=M-1\nM=!M\n";
            } else if line_vec[0] == "label" || line_vec[0] == "goto" || line_vec[0] == "if-goto" {
                asm_string += &conditional_branch_to_asm(line_vec, filename, row_num);
            } else if line_vec[0] == "call" || line_vec[0] == "function" {
                asm_string += &function_to_asm(line_vec, filename, &mut return_address_count, row_num);
            } else if line_vec[0] == "return" {
                asm_string += &return_to_asm();
            } else {
                panic!("syntax error: in {} line {}", filename, row_num);
            }
        }
    }
    // infinite loop code at end of program 
    asm_string += "// end\n(ENDLOOP)\n@ENDLOOP\n0;JMP\n";
    
    return asm_string;
}

// subroutines
fn push_or_pop_to_asm(args: Vec<&str>, filename: &str, row_num: usize) -> String {
    // error handling
    let words_num = args.len();
    if words_num < 3 {
        panic!("syntax error: in {} line {}", filename, row_num);
    } else if words_num > 3 {
        if args[3].chars().count() < 2 {
            eprintln!("warning: meaningless vm_code was skipped: in {} line {}", filename, row_num);
        } else if !(args[3].chars().nth(0).unwrap() == '/' && args[3].chars().nth(1).unwrap() == '/') {
            eprintln!("warning: meaningless vm_code was skipped: in {} line {}", filename, row_num);
        }
    }

    let arg2_num = match args[2].parse::<usize>() {
        Ok(int) => int,
        Err(_) => panic!("syntax error: in {} line {}", filename, row_num),
    };
    
    // converting [(push/pop) (local/argument/this/that/pointer/temp/constant/static) x] to asm
    if args[1] == "local" {
        if args[0] == "push" {
            return format!("// push local {}\n@LCL\nD=M\n@{}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop local {}\n@LCL\nD=M\n@{}\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in {} line {}", filename, row_num);
        }
    } else if args[1] == "argument" {
        if args[0] == "push" {
            return format!("// push argument {}\n@ARG\nD=M\n@{}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop argument {}\n@ARG\nD=M\n@{}\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in {} line {}", filename, row_num);
        }
    } else if args[1] == "this" {
        if args[0] == "push" {
            return format!("// push this {}\n@THIS\nD=M\n@{}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop this {}\n@THIS\nD=M\n@{}\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in {} line {}", filename, row_num);
        }
    } else if args[1] == "that" {
        if args[0] == "push" {
            return format!("// push that {}\n@THAT\nD=M\n@{}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop that {}\n@THAT\nD=M\n@{}\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in {} line {}", filename, row_num);
        }
    } else if args[1] == "pointer" {
        if arg2_num == 0 {
            if args[0] == "push" {
                return "// push pointer 0\n@THIS\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n".to_string();
            } else if args[0] == "pop" {
                return "// pop pointer 0\n@SP\nAM=M-1\nD=M\n@THIS\nM=D\n".to_string();
            } else {
                panic!("syntax error: in {} line {}", filename, row_num);
            }
        } else if arg2_num == 1 {
            if args[0] == "push" {
                return "// push pointer 1\n@THAT\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n".to_string();
            } else if args[0] == "pop" {
                return "// pop pointer 1\n@SP\nAM=M-1\nD=M\n@THAT\nM=D\n".to_string();
            } else {
                panic!("syntax error: in {} line {}", filename, row_num);
            }
        } else {
            panic!("syntax error: in {} line {}", filename, row_num);
        }
    } else if args[1] == "temp" {
        if arg2_num >= 8 {
            panic!("syntax error: in {} line {}", filename, row_num);
        }
        if args[0] == "push" {
            return format!("// push temp {}\n@{}\nD=A\n@5\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop temp {}\n@{}\nD=A\n@5\nD=D+A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: in {} line {}", filename, row_num);
        }
    } else if args[1] == "constant" {
        if args[0] == "push" {
            return format!("// push constant {}\n@{}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, arg2_num);
        } else {
            panic!("syntax error: [pop constant x] is not defined: in {} line {}", filename, row_num);
        }
    } else if args[1] == "static" {
        if args[0] == "push" {
            return format!("// push static {}\n@{}.{}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", arg2_num, filename, arg2_num);
        } else if args[0] == "pop" {
            return format!("// pop static {}\n@SP\nAM=M-1\nD=M\n@{}.{}\nM=D\n", arg2_num, filename, arg2_num);
        } else {
            panic!("syntax error: in {} line {}", filename, row_num);
        }
    } else {
        panic!("syntax error: in {} line {}", filename, row_num);
    }
}

fn eq_gt_lt_to_asm(code: &str, count: &mut usize) -> String {
    let upper_eq_gt_lt = code.to_uppercase();           // EQ or GT or LT
    let mut asm_string = format!("// {}\n", code);      // initial comment: [// eq] or [// gt] or [// lt]
    asm_string += "@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@R13\nD=D-M\n";                                                    // pop + pop
    asm_string = format!("{}@TRUECASE{}\nD;J{}\n@SP\nA=M-1\nM=0\n@RESULT{}\n0;JMP\n", asm_string, count, upper_eq_gt_lt, count);    // FALSE case
    asm_string = format!("{}(TRUECASE{})\n@SP\nA=M-1\nM=-1\n(RESULT{})\n", asm_string, count, count);                               // TRUE case
    *count += 1;
    return asm_string;
}

fn conditional_branch_to_asm(args: Vec<&str>, filename: &str, row_num: usize) -> String {
    // error handling
    let words_num = args.len();
    if words_num < 2 {
        panic!("syntax error: in {} line {}", filename, row_num);
    } else if words_num > 2 {
        if args[2].chars().count() < 2 {
            eprintln!("warning: meaningless vm_code was skipped: in {} line {}", filename, row_num);
        } else if !(args[2].chars().nth(0).unwrap() == '/' && args[2].chars().nth(1).unwrap() == '/') {
            eprintln!("warning: meaningless vm_code was skipped: in {} line {}", filename, row_num);
        }
    }

    let label = match args[1].parse::<i64>() {
        Ok(_) => panic!("syntax error: invalid label: in {} line {}", filename, row_num),
        Err(_) => args[1],
    };

    // converting [(label/goto/if-goto) LABEL] to asm
    if args[0] == "label" {
        return format!("({}) // label\n", label);
    } else if args[0] == "goto" {
        return format!("@{} // goto\n0;JMP\n", label);
    } else if args[0] == "if-goto" {
        return format!("// if-goto\n@SP\nAM=M-1\nD=M\n@{}\nD;JNE\n", label);
    } else {
        panic!("syntax error: in {} line {}", filename, row_num);
    }
}

fn function_to_asm(args: Vec<&str>, filename: &str, return_address_count: &mut usize, row_num: usize) -> String {
    // error handling
    let words_num = args.len();
    if words_num < 3 {
        panic!("syntax error: in {} line {}", filename, row_num);
    } else if words_num > 3 {
        if args[3].chars().count() < 2 {
            eprintln!("warning: meaningless vm_code was skipped: in {} line {}", filename, row_num);
        } else if !(args[3].chars().nth(0).unwrap() == '/' && args[3].chars().nth(1).unwrap() == '/') {
            eprintln!("warning: meaningless vm_code was skipped: in {} line {}", filename, row_num);
        }
    }
    
    let vars_num = match args[2].parse::<usize>() {
        Ok(int) => int,
        Err(_) => panic!("syntax error: invalid variable's number: in {} line {}", filename, row_num),
    };

    // converting [(call/function) func_name vars_num] to asm
    let func_name = args[1];    // wrong:=> format!("{}.{}", filename, args[1]);
    if args[0] == "call" {
        let mut asm_func_call = format!("// call {}\n", &func_name);
        asm_func_call += &format!("@ReturnAddress{}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", return_address_count);      // push return_address
        asm_func_call += "@LCL\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";                                                  // push LCL
        asm_func_call += "@ARG\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";                                                  // push ARG
        asm_func_call += "@THIS\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";                                                 // push THIS
        asm_func_call += "@THAT\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";                                                 // push THAT
        asm_func_call += "@SP\nD=M\n@LCL\nM=D\n";                                                                   // LCL = SP
        asm_func_call += &format!("@SP\nD=M\n@5\nD=D-A\n@{}\nD=D-A\n@ARG\nM=D\n", vars_num);                        // ARG = SP - 5 - vars_num
        asm_func_call += &format!("@{}\n0;JMP\n(ReturnAddress{})\n", &func_name, return_address_count);             // goto func + (return_address)
        *return_address_count += 1;
        return asm_func_call;
    } else if args[0] == "function" {
        let mut asm_def_func = format!("// function {}\n({})\n", &func_name, &func_name);   // (function)
        for _i in 0..vars_num {
            asm_def_func += "@SP\nA=M\nM=0\n@SP\nM=M+1\n";                                  // push 0 * vars_num: initializing LCL and set SP 
        }
        return asm_def_func;
    } else {
        panic!("syntax error: in {} line {}", filename, row_num);
    }
}

fn return_to_asm() -> String {
    let mut asm_string = "// return\n".to_string();
    asm_string += "@SP\nAM=M-1\nD=M\n@R14\nM=D\n";                  // return_value: pop -> R14
    asm_string += "@ARG\nD=M\n@SP\nM=D+1\n";                        // SP_new = ARG 1
    asm_string += "@LCL\nD=M\n@R13\nAM=D-1\nD=M\n@THAT\nM=D\n";     // THAT_new = LCL - 1
    asm_string += "@R13\nAM=M-1\nD=M\n@THIS\nM=D\n";                // THIS_new = LCL - 2
    asm_string += "@R13\nAM=M-1\nD=M\n@ARG\nM=D\n";                 // ARG_new = LCL - 3
    asm_string += "@R13\nAM=M-1\nD=M\n@LCL\nM=D\n";                 // LCL_new = LCL - 4
    asm_string += "@R13\nA=M-1\nD=M\n@R13\nM=D\n";                  // return_address = LCL - 5: -> R13
    asm_string += "@R14\nD=M\n@SP\nA=M-1\nM=D\n";                   // R14 -> *(SP_new - 1)
    asm_string += "@R13\nA=M\n0;JMP\n";                             // return_address -> jump
    return asm_string;
}