// compiler package
// parsing jack code via LL(1) analysis & vm code generation
// コンピュータシステムの理論と実装 §10, §11

use std::collections::HashMap;
use crate::lexical_analysis::Lexicon;

struct Data {
    lexicons: Vec<Lexicon>,
    iter: usize,
    class: String,
    field_vars_count: usize,
    conditional_branch_count: usize,
    symbol_table: SymbolTable,
    filename: String,
}

impl Data {
    fn new(lexicons: Vec<Lexicon>, filename: &str) -> Self {
        Data {
            lexicons: lexicons,
            iter: 0,
            class: "".to_string(),
            field_vars_count: 0,
            conditional_branch_count: 0,
            symbol_table: SymbolTable::new(),
            filename: filename.to_string(),
        }
    }
}

pub fn compile_starter(lex_vec: Vec<Lexicon>, filename: &str) -> String {
    let mut data = Data::new(lex_vec, filename);
    return compile_class(&mut data);
}

// ---------- class ----------
// class :: "class" className "{" classVarDec* subroutineDec* "}"
fn compile_class(data: &mut Data) -> String {
    token_checker("class", data);
    data.class = identifier_register(data, "class", &"none".to_string());
    token_checker("{", data);

    // classVarDec* subroutineDec* 
    let mut class_contents = "".to_string();
    loop {
        if tokens_matching(vec!["static", "field"], data) {
            compile_class_var_dec(data);
        } else if tokens_matching(vec!["constructor", "function", "method"], data) {
            class_contents += &compile_subroutine_dec(data);
        } else {
            break;
        }
    }

    token_checker("}", data);
    return class_contents;
}

// classVarDec :: ("static" | "field") type varName ("," varName)* ";"
fn compile_class_var_dec(data: &mut Data) {
    let var_category = tokens_checker(vec!["static", "field"], data);
    if var_category == "field" {
        data.field_vars_count += 1;
    }

    let var_type = compile_type(data);
    identifier_register(data, &var_category, &var_type);

    // (, + var_name)*
    loop {
        if token_matching(",", data) {
            data.iter += 1;
            identifier_register(data, &var_category, &var_type);
            if var_category == "field" {
                data.field_vars_count += 1;
            }
        } else {
            break;
        }
    }

    token_checker(";", data);
}

fn compile_type(data: &mut Data) -> String {
    if token_matching("int", data) {
        data.iter += 1;
        return "int".to_string();
    } else if token_matching("char", data) {
        data.iter += 1;
        return "char".to_string();
    } else if token_matching("boolean", data) {
        data.iter += 1;
        return "boolean".to_string();
    } else if token_type_matching("identifier", data) {
        // class_name
        let class_name = &data.lexicons[data.iter].token;
        data.iter += 1;
        return class_name.to_string();
    } else {
        return print_error("code 10-compile_type: invalid type", data);
    }
}

// subroutineDec :: ("constructor" | "function" | "method") ("void" | type ) subroutineName "(" parameterList ")" subroutineBody
fn compile_subroutine_dec(data: &mut Data) -> String {
    // clear local symbol_table
    data.symbol_table.clear_local_table();

    let function_category = tokens_checker(vec!["constructor", "function", "method"], data);

    // ("void" | type )
    let return_type = if token_matching("void", data) {
        data.iter += 1;
        "void".to_string()
    } else {
        compile_type(data)
    };

    let subroutine_name = identifier_register(data, "subroutine", &return_type);
    token_checker("(", data);
    compile_parameter_list(data, &function_category);
    token_checker(")", data);
    let subroutine_body = compile_subroutine_body(data);

    // code generation
    let mut code = format!("function {}.{} {}\n", &data.class, subroutine_name, data.symbol_table.var_counter);
    if function_category == "constructor".to_string() {
        code += &format!("\tpush constant {}\n\tcall Memory.alloc 1\n\tpop pointer 0\n", data.field_vars_count);
    } else if function_category == "method".to_string() {
        code += "\tpush argument 0\n\tpop pointer 0\n";
    }
    return code + &subroutine_body;
}

// parameterList :: (type varName ("," type varName)*)?
fn compile_parameter_list(data: &mut Data, category: &String) {
    // when return_type is method, 'argument 0' is assigned to the pointer of object
    if category == "method" {
        data.symbol_table.define(&"this".to_string(), "arg", &data.class, &data.filename, data.lexicons[data.iter].row_number);
    }

    if !(token_matching(")", data)) {
        let var_type = compile_type(data);
        identifier_register(data, "arg", &var_type);
    
        // (, type var_name)*
        loop {
            if token_matching(",", data) {
                data.iter += 1;
                let var_type = compile_type(data);
                identifier_register(data, "arg", &var_type);
            } else {
                break;
            }
        }
    }
}

// subroutineBody :: "{" varDec* statements "}"
fn compile_subroutine_body(data: &mut Data) -> String {
    token_checker("{", data);
    
    // varDec*
    loop {
        if token_matching("var", data) {
            compile_var_dec(data);
        } else {
            break;
        }
    }

    let statements = compile_statements(data);
    token_checker("}", data);
    return statements;
}

// varDec :: "var" type varName ("," varName)* ";"
fn compile_var_dec(data: &mut Data) {
    token_checker("var", data);
    let var_type = compile_type(data);
    identifier_register(data, "var", &var_type);
    
    // (, var_name)*
    loop {
        if token_matching(",", data) {
            data.iter += 1;
            identifier_register(data, "var", &var_type);
        } else {
            break;
        }
    }

    token_checker(";", data);
}

// ---------- statements ----------
// statements :: (letStatement | ifStatement | whileStatement | doStatement | returnStatement)*
fn compile_statements(data: &mut Data) -> String {
    let mut statements = "".to_string();
    loop {
        if token_matching("let", data) {
            statements += &compile_let_statement(data);
        } else if token_matching("if", data) {
            statements += &compile_if_statement(data);
        } else if token_matching("while", data) {
            statements += &compile_while_statement(data);
        } else if token_matching("do", data) {
            statements += &compile_do_statement(data);
        } else if token_matching("return", data) {
            statements += &compile_return_statement(data);
        } else {
            break;
        }
    }
    return statements;
}

// letStatement :: "let" varName ("[" expression "]")? "=" expression ";"
fn compile_let_statement(data: &mut Data) -> String {
    token_checker("let", data);
    let var_name = identifier_use(data);

    // array: [ expression ]
    let mut array_flag = false;
    let mut array_index = "".to_string();
    if token_matching("[", data) {
        array_flag = true;
        data.iter += 1;
        array_index += &compile_expression(data);
        token_checker("]", data);
    }

    token_checker("=", data);
    let expression = compile_expression(data);
    token_checker(";", data);

    // code generation
    if array_flag {
        return format!("\tpush {}\n{}\tadd\n{}\tpop temp 0\n\tpop pointer 1\n\tpush temp 0\n\tpop that 0\n", var_name, array_index, expression);
    } else {
        return format!("{}\tpop {}\n", expression, var_name);
    }
}

// ifStatement :: "if" "(" expression ")" "{" statements "}" ("else" "{" statements "}")?
fn compile_if_statement(data: &mut Data) -> String {
    token_checker("if", data);
    token_checker("(", data);
    let condition = compile_expression(data);
    token_checker(")", data);
    token_checker("{", data);
    let if_statement = compile_statements(data);
    token_checker("}", data);

    // (else { statements })?
    let mut else_flag = false;
    let mut else_statements = "".to_string();
    if token_matching("else", data) {
        else_flag = true;
        data.iter += 1;
        token_checker("{", data);
        else_statements += &compile_statements(data);
        token_checker("}", data);
    }

    // code generation
    let mut code = format!("{}\tnot\n\tif-goto {}_FALSECASE_{}\n{}", condition, &data.class, data.conditional_branch_count, if_statement);
    if else_flag {
        code += &format!("\tgoto {}_TRUECASE_{}\n", &data.class, data.conditional_branch_count);
    }
    code += &format!("label {}_FALSECASE_{}\n", &data.class, data.conditional_branch_count);
    if else_flag {
        code += &format!("{}label {}_TRUECASE_{}\n", else_statements, &data.class, data.conditional_branch_count);
    }
    data.conditional_branch_count += 1;
    return code;
}

// whileStatements :: "while" "(" expression ")" "{" statements "}"
fn compile_while_statement(data: &mut Data) -> String {
    token_checker("while", data);
    token_checker("(", data);
    let condition = compile_expression(data);
    token_checker(")", data);
    token_checker("{", data);
    let statements = compile_statements(data);
    token_checker("}", data);

    // code generation
    let while_label = format!("{}_WHILE_{}", &data.class, data.conditional_branch_count);
    let break_label = format!("{}_BREAK_{}", &data.class, data.conditional_branch_count);
    let code = format!("label {}\n{}\tnot\n\tif-goto {}\n{}\tgoto {}\nlabel {}\n", while_label, condition, break_label, statements, while_label, break_label);
    data.conditional_branch_count += 1;
    return code;
}

// doStatements :: "do" subroutineCall ";"
fn compile_do_statement(data: &mut Data) -> String {
    token_checker("do", data);
    let subroutine = compile_subroutine_call(data);
    token_checker(";", data);

    return subroutine + "\tpop temp 0\n";
}

// returnStatements :: "return" expression? ";"
fn compile_return_statement(data: &mut Data) -> String {
    token_checker("return", data);
    let mut expression = "\tpush constant 0\n".to_string();
    if !(token_matching(";", data)) {
        expression = compile_expression(data);
    }
    token_checker(";", data);

    return format!("{}\treturn\n", expression);
}

// ---------- expression ----------
// subroutineCall :: ((className | varName) ".")? subroutineName "(" expressionList ")"
fn compile_subroutine_call(data: &mut Data) -> String {
    if data.lexicons.len() <= data.iter + 1 {
        panic!("syntax error: code 11-compile_subroutine_call: irregular end of program: in {} line {}", data.filename, data.lexicons[data.iter - 1].row_number);

    // subroutineName "(" expressionList ")" := (this.) method(arguments)
    } else if data.lexicons[data.iter + 1].token == "(".to_string() && data.lexicons[data.iter + 1].token_type == "symbol".to_string() {
        let (subroutine_name, expression_list, argument_counter) = subroutine_for_subroutine_call(data);
        return format!("\tpush pointer 0\n{}\tcall {}.{} {}\n", expression_list, &data.class, subroutine_name, argument_counter + 1);

    // (className | varName) "." subroutineName "(" expressionList ")"
    } else if data.lexicons[data.iter + 1].token == ".".to_string() && data.lexicons[data.iter + 1].token_type == "symbol".to_string() {
        // (className | varName)
        let name = if token_type_matching("identifier", data) {
            data.lexicons[data.iter].token.clone()
        } else {
            print_error("code 12-compile_subroutine_call: invalid name of subroutine", data)
        };
        data.iter += 1;

        token_checker(".", data);
        let (subroutine_name, expression_list, argument_counter) = subroutine_for_subroutine_call(data);

        // code generation
        let primitive_type = vec!["int".to_string(), "char".to_string(), "boolean".to_string(), "void".to_string()];
        if let Some(info) = data.symbol_table.find(&name) {
            if info.category == "subroutine".to_string() {
                return print_error("code 13-compile_subroutine_call: invalid method call", data);
            } else if info.category != "class".to_string() {
                if primitive_type.contains(&info.var_type) {
                    return print_error("code 14-compile_subroutine_call: invalid method call", data);
                } else {
                    // varName.method(arguments)
                    return format!("\tpush {}\n{}\tcall {}.{} {}\n", info.index, expression_list, info.var_type, subroutine_name, argument_counter + 1);
                }
            }
        }
        // className.function(arguments)
        return format!("{}\tcall {}.{} {}\n", expression_list, &name, subroutine_name, argument_counter);
    } else {
        return print_error("code 15-compile_subroutine_call: invalid subroutine call", data);
    }
}

// subroutineName "(" expressionList ")"
fn subroutine_for_subroutine_call(data: &mut Data) -> (String, String, usize) {
    // subroutineName
    let subroutine_name = if token_type_matching("identifier", data) {
        data.lexicons[data.iter].token.clone()
    } else {
        print_error("code 14-compile_subroutine_call: invalid name of subroutine", data)
    };
    data.iter += 1;

    token_checker("(", data);

    // expressionList ")" :: (expression ("," expression)*)? ")"
    let mut expression_list = "".to_string();
    let mut argument_counter = 0;
    if token_matching(")", data) {
        data.iter += 1;
    } else {
        expression_list += &compile_expression(data);
        argument_counter += 1;
        // ("," expression)* ")"
        loop {
            if token_matching(")", data) {
                data.iter += 1;
                break;
            } else if token_matching(",", data) {
                data.iter += 1;
                expression_list += &compile_expression(data);
                argument_counter += 1;
            } else {
                print_error("code 15-compile_subroutine_call: invalid argument of subroutine", data);
            }
        }
    }
    return (subroutine_name, expression_list, argument_counter);
}

// expression :: term (op term)*
fn compile_expression(data: &mut Data) -> String {
    let mut expression = compile_term(data);
    loop {
        if token_matching("+", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\tadd\n";
        } else if token_matching("-", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\tsub\n";
        } else if token_matching("*", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\tcall Math.multiply 2\n";
        } else if token_matching("/", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\tcall Math.divide 2\n";
        } else if token_matching("&", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\tand\n";
        } else if token_matching("|", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\tor\n";
        } else if token_matching("<", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\tlt\n";
        } else if token_matching(">", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\tgt\n";
        } else if token_matching("=", data) {
            data.iter += 1;
            expression += &compile_term(data);
            expression += "\teq\n";
        } else {
            break;
        }
    }
    return expression;
}

// term :: integerConstant | stringConstant | keywordConstant | varName | varName "[" expression "]" | "(" expression ")" | unaryOp term | subroutineCall
fn compile_term(data: &mut Data) -> String {
    // integerConstant
    if token_type_matching("integer_constant", data) {
        let integer = &data.lexicons[data.iter].token;
        data.iter += 1;
        return format!("\tpush constant {}\n", integer);

    // stringConstant
    } else if token_type_matching("string_constant", data) {
        let chars_table = HashMap::from([
            (' ', 32), ('!', 33), ('"', 34), ('#', 35), ('$', 36), ('%', 37), ('&', 38), ('\'', 39), ('(', 40), (')', 41),
            ('*', 42), ('+', 43), (',', 44), ('-', 45), ('.', 46), ('/', 47), ('0', 48), ('1', 49), ('2', 50), ('3', 51),
            ('4', 52), ('5', 53), ('6', 54), ('7', 55), ('8', 56), ('9', 57), (':', 58), (';', 59), ('<', 60), ('=', 61),
            ('>', 62), ('?', 63), ('@', 64), ('A', 65), ('B', 66), ('C', 67), ('D', 68), ('E', 69), ('F', 70), ('G', 71),
            ('H', 72), ('I', 73), ('J', 74), ('K', 75), ('L', 76), ('M', 77), ('N', 78), ('O', 79), ('P', 80), ('Q', 81),
            ('R', 82), ('S', 83), ('T', 84), ('U', 85), ('V', 86), ('W', 87), ('X', 88), ('Y', 89), ('Z', 90), ('[', 91),
            ('\\', 92), (']', 93), ('^', 94), ('_', 95), ('`', 96), ('a', 97), ('b', 98), ('c', 99), ('d', 100), ('e', 101),
            ('f', 102), ('g', 103), ('h', 104), ('i', 105), ('j', 106), ('k', 107), ('l', 108), ('m', 109), ('n', 110), ('o', 111),
            ('p', 112), ('q', 113), ('r', 114), ('s', 115), ('t', 116), ('u', 117), ('v', 118), ('w', 119), ('x', 120), ('y', 121),
            ('z', 122), ('{', 123), ('|', 124), ('}', 125), ('~', 126),
        ]);
        let string = &data.lexicons[data.iter].token;
        let length = string.chars().count();
        let mut code = format!("\tpush constant {}\n\tcall String.new 1\n", length);
        for c in string.chars() {
            let char_number = match chars_table.get(&c) {
                Some(number) => number,
                None => panic!("syntax error: code 16-compile_term: invalid char in string_constant: in {} line {}", data.filename, data.lexicons[data.iter].row_number),
            };
            code += &format!("\tpush constant {}\n\tcall String.appendChar 2\n", char_number);
        }
        data.iter += 1;
        return code;

    } else if token_type_matching("keyword", data) {
        // keywordConstant
        if tokens_matching(vec!["false", "null"], data) {
            data.iter += 1;
            return "\tpush constant 0\n".to_string();
        } else if token_matching("true", data) {
            data.iter += 1;
            return "\tpush constant 1\n\tneg\n".to_string();
        } else if token_matching("this", data) {
            data.iter += 1;
            return "\tpush pointer 0\n".to_string();
        } else {
            return print_error("code 17-compile_term: invalid keyword term", data);
        }

    } else if token_type_matching("symbol", data) {
        // unaryOp term
        if token_matching("-", data) {
            data.iter += 1;
            let term = compile_term(data);
            return term + "\tneg\n";
        } else if token_matching("~", data) {
            data.iter += 1;
            let term = compile_term(data);
            return term + "\tnot\n";

        // "(" expression ")"
        } else if token_matching("(", data) {
            data.iter += 1;
            let expression = compile_expression(data);
            token_checker(")", data);
            return expression;
        } else {
            return print_error("code 18-compile_term: invalid symbol", data);
        }

    } else if token_type_matching("identifier", data) {
        if data.lexicons.len() > data.iter + 1 {
            // varName "[" expression "]"
            if data.lexicons[data.iter + 1].token == "[".to_string() && data.lexicons[data.iter + 1].token_type == "symbol".to_string() {
                let var_index = identifier_use(data);
                data.iter += 1;
                let expression = compile_expression(data);
                token_checker("]", data);
                return format!("\tpush {}\n{}\tadd\n\tpop pointer 1\n\tpush that 0\n", var_index, expression);

            // subroutineCall :: ((className | varName) ".")? subroutineName "(" expressionList ")"
            } else if (data.lexicons[data.iter + 1].token == ".".to_string() || data.lexicons[data.iter + 1].token == "(".to_string()) && data.lexicons[data.iter + 1].token_type == "symbol".to_string() {
                return compile_subroutine_call(data);
            }
        }
        // varName
        let var_index = identifier_use(data);
        return format!("\tpush {}\n", var_index);
    } else {
        return print_error("code 19-compile_term", data);
    }
}

// subroutine
fn print_error<T: std::fmt::Display>(error_code: T, data: &mut Data) -> String {
    if data.lexicons.len() <= data.iter {
        panic!("syntax error: code 00-token_matching: irregular end of program: in {} line {}", data.filename, data.lexicons[data.iter - 1].row_number);
    } else {
        panic!("syntax error: {}: in {} line {}", error_code, data.filename, data.lexicons[data.iter].row_number);
    }
    "_ERROR".to_string()
}

fn token_matching(token: &str, data: &mut Data) -> bool {
    let keywords = vec!["class", "constructor", "function", "method", "field", "static", "var", "int", "char", 
                       "boolean", "void", "true", "false", "null", "this", "let", "do", "if", "else", "while", "return"];
    let symbols = vec!["{", "}", "(", ")", "[", "]", ".", ",", ";", "+", "-", "*", "/", "&", "|", "<", ">", "=", "~"];

    if data.lexicons.len() <= data.iter {
        panic!("syntax error: code 01-token_matching: irregular end of program: in {} line {}", data.filename, data.lexicons[data.iter - 1].row_number);
    } else {
        let keyword_case = keywords.contains(&token) && data.lexicons[data.iter].token_type == "keyword".to_string() && data.lexicons[data.iter].token == token.to_string();
        let symbol_case = symbols.contains(&token) && data.lexicons[data.iter].token_type == "symbol".to_string() && data.lexicons[data.iter].token == token.to_string();
        return keyword_case || symbol_case;
    }
}

fn tokens_matching(tokens: Vec<&str>, data: &mut Data) -> bool {
    let mut tmp = false;
    for token in tokens {
        tmp = tmp || token_matching(token, data);
    }
    return tmp;
}

fn token_type_matching(token_type: &str, data: &mut Data) -> bool {
    if data.lexicons.len() <= data.iter {
        panic!("syntax error: code 02-token_type_matching: irregular end of program: in {} line {}", data.filename, data.lexicons[data.iter - 1].row_number);
    } else {
        return data.lexicons[data.iter].token_type == token_type.to_string();
    }
}

fn token_checker(token: &str, data: &mut Data) {
    if token_matching(token, data) {
        data.iter += 1;
    } else {
        print_error(format!("code 03-token_checker: invalid token '{}'", token), data);
    }
}

fn tokens_checker(tokens: Vec<&str>, data: &mut Data) -> String {
    for token in tokens {
        if token_matching(token, data) {
            data.iter += 1;
            return token.to_string();
        }
    }
    return print_error(format!("code 04-tokens_checker: invalid token '{}'", data.lexicons[data.iter].token), data);
}

fn identifier_register(data: &mut Data, category: &str, var_type: &String) -> String {
    if data.lexicons.len() <= data.iter {
        panic!("syntax error: code 05-identifier_register: irregular end of program: in {} line {}", data.filename, data.lexicons[data.iter - 1].row_number);
    } else if data.lexicons[data.iter].token_type == "identifier".to_string() {
        let index = data.symbol_table.define(&data.lexicons[data.iter].token, category, var_type, &data.filename, data.lexicons[data.iter].row_number);
        data.iter += 1;
        return index;
    } else {
        return print_error("code 06-identifier_register: invalid kind of identifier", data);
    }
}

fn identifier_use(data: &mut Data) -> String {
    if data.lexicons.len() <= data.iter {
        panic!("syntax error: code 07-identifier_use: irregular end of program: in {} line {}", data.filename, data.lexicons[data.iter - 1].row_number);
    } else if data.lexicons[data.iter].token_type == "identifier".to_string() {
        let symbol_info = match data.symbol_table.find(&data.lexicons[data.iter].token) {
            Some(info) => info,
            None => panic!("syntax error: code 08-identifier_use: variable is not defined: in {} line {}", data.filename, data.lexicons[data.iter].row_number),
        };
        data.iter += 1;
        return symbol_info.index.clone();
    } else {
        return print_error("code 09-identifier_use: invalid kind of identifier", data);
    }
}

struct SymbolInfo {
    category: String,
    var_type: String,
    index: String,
}

impl SymbolInfo {
    fn new(category: &String, var_type: &String, index: &String) -> Self {
        SymbolInfo {category: category.to_string(), var_type: var_type.to_string(), index: index.to_string()}
    }
}

struct SymbolTable {
    global: HashMap<String, SymbolInfo>,
    local: HashMap<String, SymbolInfo>,
    static_counter: usize,
    field_counter: usize,
    var_counter: usize,
    arg_counter: usize,
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            global: HashMap::new(),
            local: HashMap::new(),
            static_counter: 0,
            field_counter: 0,
            var_counter: 0,
            arg_counter: 0,
        }
    }

    fn define(&mut self, name: &String, category: &str, var_type: &String, filename: &String, row_number: usize) -> String {
        if category == "class" || category == "subroutine" {
            if let Some(_name) = self.global.get(name) {
                panic!("syntax error: code s0-SymbolTable_define: identifier '{}' is already used: in {} line {}", name, filename, row_number);
            } else {
                let symbol_table = SymbolInfo::new(&category.to_string(), var_type, &"none".to_string());
                self.global.insert(name.to_string(), symbol_table);
                return name.to_string();
            }
        } else if category == "static" {
            if let Some(_name) = self.global.get(name) {
                panic!("syntax error: code s1-SymbolTable_define: identifier '{}' is already used: in {} line {}", name, filename, row_number);
            } else {
                let index = format!("static {}", self.static_counter);
                self.static_counter += 1; 
                let symbol_table = SymbolInfo::new(&category.to_string(), var_type, &index);
                self.global.insert(name.to_string(), symbol_table);
                return index;
            }
        } else if category == "field" {
            if let Some(_name) = self.global.get(name) {
                panic!("syntax error: code s2-SymbolTable_define: identifier '{}' is already used: in {} line {}", name, filename, row_number);
            } else {
                let index = format!("this {}", self.field_counter);
                self.field_counter += 1; 
                let symbol_table = SymbolInfo::new(&category.to_string(), var_type, &index);
                self.global.insert(name.to_string(), symbol_table);
                return index;
            }
        } else if category == "var" {
            if let Some(_name) = self.local.get(name) {
                panic!("syntax error: code s3-SymbolTable_define: identifier '{}' is already used: in {} line {}", name, filename, row_number);
            } else {
                let index = format!("local {}", self.var_counter);
                self.var_counter += 1; 
                let symbol_table = SymbolInfo::new(&category.to_string(), var_type, &index);
                self.local.insert(name.to_string(), symbol_table);
                return index;
            }
        } else if category == "arg" {
            if let Some(_name) = self.local.get(name) {
                panic!("syntax error: code s4-SymbolTable_define: identifier '{}' is already used: in {} line {}", name, filename, row_number);
            } else {
                let index = format!("argument {}", self.arg_counter);
                self.arg_counter += 1; 
                let symbol_table = SymbolInfo::new(&category.to_string(), var_type, &index);
                self.local.insert(name.to_string(), symbol_table);
                return index;
            }
        } else {
            panic!("syntax error: code s5-SymbolTable_define: invalid indentifier category '{}': in {} line {}", category, filename, row_number);
        }
    }

    fn find(&self, name: &String) -> Option<&SymbolInfo> {
        if let Some(info) = self.local.get(name) {
            return Some(info);
        } else if let Some(info) = self.global.get(name) {
            return Some(info);
        } else {
            return None;
        }
    }

    fn clear_local_table(&mut self) {
        self.local.clear();
        self.var_counter = 0;
        self.arg_counter = 0;
    }
}