// parser package
// parsing jack code via LL(1) analysis
// コンピュータシステムの理論と実装 §10

use crate::lexical_analysis::Lexicon;

pub fn parse_class(lex_vec: &Vec<Lexicon>, filename: &str) -> String {
    if lex_vec.len() < 4 {
        panic!("syntax error: code 0-parse_class: invalid class: in {}", filename);
    }
    let mut iter = 0;
    let indent = "\t".to_string();

    let token_class = token_processer("class".to_string(), lex_vec, &mut iter, &indent, filename);
    let class_name = parse_class_name(lex_vec, &mut iter, &indent, filename);
    let token_bracket1 = token_processer("{".to_string(), lex_vec, &mut iter, &indent, filename);
    let mut class_contents = "".to_string();
    loop {
        if lex_vec.len() <= iter {
            panic!("syntax error: code 1-parse_class: invalid class definition: in {} line {}", filename, lex_vec[iter-1].row_number);
        } else if lex_vec[iter].token == "static".to_string() || lex_vec[iter].token == "field".to_string() {
            class_contents += &parse_class_var_dec(lex_vec, &mut iter, &indent, filename);
        } else if lex_vec[iter].token == "constructor".to_string() || lex_vec[iter].token == "function".to_string() || lex_vec[iter].token == "method".to_string() {
            class_contents += &parse_subroutine_dec(lex_vec, &mut iter, &indent, filename);
        } else {
            break;
        }
    } 
    let token_bracket2 = token_processer("}".to_string(), lex_vec, &mut iter, &indent, filename);
    return format!("<class>\n{}{}{}{}{}</class>", token_class, class_name, token_bracket1, class_contents, token_bracket2);
}

// parse structure of the program
fn parse_class_name(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 2-parse_class_name: invalid class name: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type == "identifier".to_string() {
        let xml = format!("{}<identifier> {} </identifier>\n", indent, lex_vec[*iter].token);
        *iter += 1;
        return xml;
    } else {
        panic!("syntax error: code 3-parse_class_name: invalid class name: in {} line {}", filename, lex_vec[*iter].row_number);
    }
}

fn parse_class_var_dec(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);
    
    // check "static" or "field"
    let mut varfield = new_indent.clone();
    if lex_vec[*iter].token_type != "keyword".to_string() {
        panic!("syntax error: code 4-parse_class_var_dec: invalid variable: in {} line {}", filename, lex_vec[*iter].row_number);
    } else if lex_vec[*iter].token == "field".to_string() {
        varfield += "<keyword> field </keyword>\n";
    } else if lex_vec[*iter].token == "static".to_string() {
        varfield += "<keyword> static </keyword>\n";
    } else {
        panic!("syntax error: code 5-parse_class_var_dec: invalid variable: in {} line {}", filename, lex_vec[*iter].row_number);
    }
    *iter += 1;

    let vartype = parse_type(lex_vec, iter, &new_indent, filename);
    let varname = parse_var_name(lex_vec, iter, &new_indent, filename);
    let mut vars = "".to_string();
    loop {
        if lex_vec.len() <= *iter {
            panic!("syntax error: code 6-parse_class_var_dec: invalid variable definition: in {} line {}", filename, lex_vec[*iter-1].row_number);
        } else if lex_vec[*iter].token == ",".to_string() && lex_vec[*iter].token_type == "symbol".to_string() {
            vars = format!("{}{}<symbol> , </symbol>\n", vars, new_indent);
            *iter += 1;
            vars += &parse_var_name(lex_vec, iter, &new_indent, filename);
        } else {
            break;
        }
    }
    let semicolon = token_processer(";".to_string(), lex_vec, iter, &new_indent, filename);
    return format!("{}<classVarDec>\n{}{}{}{}{}{}</classVarDec>\n", indent, varfield, vartype, varname, vars, semicolon, indent);
}

fn parse_type(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let mut xml = indent.clone();
    if *iter >= lex_vec.len() {
        panic!("syntax error: code 7-parse_type: invalid type: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type == "keyword".to_string() {
        if lex_vec[*iter].token == "int".to_string() {
            xml += "<keyword> int </keyword>\n";
        } else if lex_vec[*iter].token == "char".to_string() {
            xml += "<keyword> char </keyword>\n";
        } else if lex_vec[*iter].token == "boolean".to_string() {
            xml += "<keyword> boolean </keyword>\n";
        } else {
            panic!("syntax error: code 8-parse_type: invalid type: in {} line {}", filename, lex_vec[*iter].row_number);
        }
        *iter += 1;
    } else if lex_vec[*iter].token_type == "identifier".to_string() {
        xml = parse_class_name(lex_vec, iter, indent, filename);
    } else {
        panic!("syntax error: code 9-parse_type: invalid variable type: in {} line {}", filename, lex_vec[*iter].row_number);
    }
    return xml;
}

fn parse_var_name(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 10-parse_var_name: invalid variable name: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type == "identifier".to_string() {
        let xml = format!("{}<identifier> {} </identifier>\n", indent, lex_vec[*iter].token);
        *iter += 1;
        return xml;
    } else {
        panic!("syntax error: code 11-parse_var_name: invalid variable name: in {} line {}", filename, lex_vec[*iter].row_number);
    }
}

fn parse_subroutine_dec(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);
    
    // check "constructor", "function" or "method"
    let mut routine_type = new_indent.clone();
    if lex_vec[*iter].token_type != "keyword".to_string() {
        panic!("syntax error: code 12-parse_subroutine_dec: invalid function: in {} line {}", filename, lex_vec[*iter].row_number);
    } else if lex_vec[*iter].token == "function".to_string() {
        routine_type += "<keyword> function </keyword>\n";
    } else if lex_vec[*iter].token == "method".to_string() {
        routine_type += "<keyword> method </keyword>\n";
    } else if lex_vec[*iter].token == "constructor".to_string() {
        routine_type += "<keyword> constructor </keyword>\n";
    } else {
        panic!("syntax error: code 13-parse_subroutine_dec: invalid function: in {} line {}", filename, lex_vec[*iter].row_number);
    }
    *iter += 1;

    // check "void" or type of return value
    let mut return_type = new_indent.clone();
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 14-parse_subroutine_dec: invalid type: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type == "keyword".to_string() {
        if lex_vec[*iter].token == "void".to_string() {
            return_type += "<keyword> void </keyword>\n";
            *iter += 1;
        } else {
            panic!("syntax error: code 15-parse_subroutine_dec: invalid type: in {} line {}", filename, lex_vec[*iter].row_number);
        }
    } else if lex_vec[*iter].token_type == "identifier".to_string() {
        return_type = parse_type(lex_vec, iter, &new_indent, filename);
    } else {
        panic!("syntax error: code 16-parse_subroutine_dec: invalid type: in {} line {}", filename, lex_vec[*iter].row_number);
    }

    let function_name = parse_subroutine_name(lex_vec, iter, &new_indent, filename);
    let token_bracket1 = token_processer("(".to_string(), lex_vec, iter, &new_indent, filename);
    let parameter_list = parse_parameter_list(lex_vec, iter, &new_indent, filename);
    let token_bracket2 = token_processer(")".to_string(), lex_vec, iter, &new_indent, filename);
    let subroutine_body = parse_subroutine_body(lex_vec, iter, &new_indent, filename);
    
    return format!("{}<subroutineDec>\n{}{}{}{}{}{}{}{}</subroutineDec>\n", indent, routine_type, return_type, function_name, token_bracket1, parameter_list, token_bracket2, subroutine_body, indent);
}

fn parse_subroutine_name(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 17-parse_subroutine_name: invalid function name: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type == "identifier".to_string() {
        let xml = format!("{}<identifier> {} </identifier>\n", indent, lex_vec[*iter].token);
        *iter += 1;
        return xml;
    } else {
        panic!("syntax error: code 18-parse_subroutine_name: invalid function name: in {} line {}", filename, lex_vec[*iter].row_number);
    }
}

fn parse_parameter_list(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let mut list = "".to_string();
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 19-parse_parameter_list: invalid function definition: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token != ")".to_string() {
        let vartype = parse_type(lex_vec, iter, &new_indent, filename);
        let varname = parse_var_name(lex_vec, iter, &new_indent, filename);
        list = format!("{}{}", vartype, varname);

        loop {
            if lex_vec.len() <= *iter {
                panic!("syntax error: code 20-parse_parameter_list: invalid function definition: in {} line {}", filename, lex_vec[*iter-1].row_number);
            } else if lex_vec[*iter].token == ",".to_string() && lex_vec[*iter].token_type == "symbol".to_string() {
                list = format!("{}{}<symbol> , </symbol>\n", list, new_indent);
                *iter += 1;
                list += &parse_type(lex_vec, iter, &new_indent, filename);
                list += &parse_var_name(lex_vec, iter, &new_indent, filename);
            } else {
                break;
            }
        }
    }
    return format!("{}<parameterList>\n{}{}</parameterList>\n", indent, list, indent);
}

fn parse_subroutine_body(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);
    let token_bracket1 = token_processer("{".to_string(), lex_vec, iter, &new_indent, filename);

    let mut body = "".to_string();
    loop {
        if lex_vec.len() <= *iter {
            panic!("syntax error: code 21-parse_subroutine_body: invalid subroutine: in {} line {}", filename, lex_vec[*iter-1].row_number);
        } else if lex_vec[*iter].token == "var".to_string() && lex_vec[*iter].token_type == "keyword".to_string() {
            body += &parse_var_dec(lex_vec, iter, &new_indent, filename);
        } else {
            break;
        }
    }

    let statements = parse_statements(lex_vec, iter, &new_indent, filename);
    let token_bracket2 = token_processer("}".to_string(), lex_vec, iter, &new_indent, filename);
    return format!("{}<subroutineBody>\n{}{}{}{}{}</subroutineBody>\n", indent, token_bracket1, body, statements, token_bracket2, indent);
}

fn parse_var_dec(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let token_var = token_processer("var".to_string(), lex_vec, iter, &new_indent, filename);
    let var_type = parse_type(lex_vec, iter, &new_indent, filename);
    let var_name = parse_var_name(lex_vec, iter, &new_indent, filename);

    let mut vars = "".to_string();
    loop {
        if lex_vec.len() <= *iter {
            panic!("syntax error: code 22-parse_var_dec: invalid subroutine: in {} line {}", filename, lex_vec[*iter-1].row_number);
        } else if lex_vec[*iter].token == ",".to_string() && lex_vec[*iter].token_type == "symbol".to_string() {
            vars += &new_indent;
            vars += "<symbol> , </symbol>\n";
            *iter += 1;
            vars += &parse_var_name(lex_vec, iter, &new_indent, filename);
        } else {
            break;
        }
    }
    let semicolon = token_processer(";".to_string(), lex_vec, iter, &new_indent, filename);
    return format!("{}<varDec>\n{}{}{}{}{}{}</varDec>\n", indent, token_var, var_type, var_name, vars, semicolon, indent);
}

// parse statements
fn parse_statements(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let mut statement = "".to_string();
    let statement_type = vec!["let".to_string(), "if".to_string(), "while".to_string(), "do".to_string(), "return".to_string()];
    loop {
        if lex_vec.len() <= *iter {
            panic!("syntax error: code 23-parse_statements: invalid statements: in {} line {}", filename, lex_vec[*iter-1].row_number);
        } else if lex_vec[*iter].token_type == "keyword".to_string() && statement_type.contains(&lex_vec[*iter].token) {
            statement += &parse_statement(lex_vec, iter, indent, filename);
        } else {
            break;
        }
    }
    return format!("{}<statements>\n{}{}</statements>\n", indent, statement, indent);
}

fn parse_statement(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    if lex_vec.len() <= *iter {
        panic!("syntax error: code 24-parse_statement: invalid statement: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type != "keyword".to_string() {
        panic!("syntax error: code 25-parse_statement: invalid statement: in {} line {}", filename, lex_vec[*iter].row_number);
    } else if lex_vec[*iter].token == "let".to_string() {
        return parse_let_statement(lex_vec, iter, &new_indent, filename);
    } else if lex_vec[*iter].token == "if".to_string() {
        return parse_if_statement(lex_vec, iter, &new_indent, filename);
    } else if lex_vec[*iter].token == "while".to_string() {
        return parse_while_statement(lex_vec, iter, &new_indent, filename);
    } else if lex_vec[*iter].token == "do".to_string() {
        return parse_do_statement(lex_vec, iter, &new_indent, filename);
    } else if lex_vec[*iter].token == "return".to_string() {
        return parse_return_statement(lex_vec, iter, &new_indent, filename);
    } else {
        panic!("syntax error: code 26-parse_statement: invalid statement: in {} line {}", filename, lex_vec[*iter].row_number);
    }
}

fn parse_let_statement(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let let_token = token_processer("let".to_string(), lex_vec, iter, &new_indent, filename);
    let var_name = parse_var_name(lex_vec, iter, &new_indent, filename);
    let mut array_bracket = "".to_string();
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 27-parse_let_statement: invalid let statement: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type != "symbol".to_string() {
        panic!("syntax error: code 28-parse_let_statement: invalid let statement: in {} line {}", filename, lex_vec[*iter].row_number);
    } else if lex_vec[*iter].token == "[".to_string() {
        array_bracket += &token_processer("[".to_string(), lex_vec, iter, &new_indent, filename);
        array_bracket += &parse_expression(lex_vec, iter, &new_indent, filename);
        array_bracket += &token_processer("]".to_string(), lex_vec, iter, &new_indent, filename);
    }
    let token_eq = token_processer("=".to_string(), lex_vec, iter, &new_indent, filename);
    let expression = parse_expression(lex_vec, iter, &new_indent, filename);
    let semicolon = token_processer(";".to_string(), lex_vec, iter, &new_indent, filename);
    return format!("{}<letStatement>\n{}{}{}{}{}{}{}</letStatement>\n", indent, let_token, var_name, array_bracket, token_eq, expression, semicolon, indent);
}

fn parse_if_statement(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let if_token = token_processer("if".to_string(), lex_vec, iter, &new_indent, filename);
    let token_bracket1 = token_processer("(".to_string(), lex_vec, iter, &new_indent, filename);
    let condition = parse_expression(lex_vec, iter, &new_indent, filename);
    let token_bracket2 = token_processer(")".to_string(), lex_vec, iter, &new_indent, filename);
    let token_bracket3 = token_processer("{".to_string(), lex_vec, iter, &new_indent, filename);
    let branch = parse_statements(lex_vec, iter, &new_indent, filename);
    let token_bracket4 = token_processer("}".to_string(), lex_vec, iter, &new_indent, filename);

    let mut else_statements = "".to_string();
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 29-parse_if_statement: invalid if-else statement: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token == "else".to_string() && lex_vec[*iter].token_type == "keyword".to_string() {
        else_statements += &token_processer("else".to_string(), lex_vec, iter, &new_indent, filename);
        else_statements += &token_processer("{".to_string(), lex_vec, iter, &new_indent, filename);
        else_statements += &parse_statements(lex_vec, iter, &new_indent, filename);
        else_statements += &token_processer("}".to_string(), lex_vec, iter, &new_indent, filename);
    }
    return format!("{}<ifStatement>\n{}{}{}{}{}{}{}{}{}</ifStatement>\n", indent, if_token, token_bracket1, condition, token_bracket2, token_bracket3, branch, token_bracket4, else_statements, indent);
}

fn parse_while_statement(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let while_token = token_processer("while".to_string(), lex_vec, iter, &new_indent, filename);
    let token_bracket1 = token_processer("(".to_string(), lex_vec, iter, &new_indent, filename);
    let condition = parse_expression(lex_vec, iter, &new_indent, filename);
    let token_bracket2 = token_processer(")".to_string(), lex_vec, iter, &new_indent, filename);
    let token_bracket3 = token_processer("{".to_string(), lex_vec, iter, &new_indent, filename);
    let statements = parse_statements(lex_vec, iter, &new_indent, filename);
    let token_bracket4 = token_processer("}".to_string(), lex_vec, iter, &new_indent, filename);
    return format!("{}<whileStatement>\n{}{}{}{}{}{}{}{}</whileStatement>\n", indent, while_token, token_bracket1, condition, token_bracket2, token_bracket3, statements, token_bracket4, indent);
}

fn parse_do_statement(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let do_token = token_processer("do".to_string(), lex_vec, iter, &new_indent, filename);
    let subroutine = parse_subroutine_call(lex_vec, iter, &new_indent, filename);
    let semicolon = token_processer(";".to_string(), lex_vec, iter, &new_indent, filename);
    return format!("{}<doStatement>\n{}{}{}{}</doStatement>\n", indent, do_token, subroutine, semicolon, indent);
}

fn parse_return_statement(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let return_token = token_processer("return".to_string(), lex_vec, iter, &new_indent, filename);
    let mut expression = "".to_string();
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 30-parse_return_statement: invalid return statement: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token != ";".to_string() {
        expression += &parse_expression(lex_vec, iter, &new_indent, filename);
    }
    let semicolon = token_processer(";".to_string(), lex_vec, iter, &new_indent, filename);
    return format!("{}<returnStatement>\n{}{}{}{}</returnStatement>\n", indent, return_token, expression, semicolon, indent);
}

// parse expression
fn parse_expression(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);
    let operator_type = vec!["+".to_string(), "-".to_string(), "*".to_string(), "/".to_string(), "&".to_string(), "|".to_string(), "<".to_string(), ">".to_string(), "=".to_string()];

    let term = parse_term(lex_vec, iter, &new_indent, filename);
    let mut binary_operators_and_terms = "".to_string();
    loop {
        if lex_vec.len() <= *iter {
            panic!("syntax error: code 31-parse_expression: invalid expression: in {} line {}", filename, lex_vec[*iter-1].row_number);
        } else if lex_vec[*iter].token_type == "symbol".to_string() && operator_type.contains(&lex_vec[*iter].token) {
            binary_operators_and_terms += &parse_operator(lex_vec, iter, &new_indent, filename);
            binary_operators_and_terms += &parse_term(lex_vec, iter, &new_indent, filename);
        } else {
            break;
        }
    }
    return format!("{}<expression>\n{}{}{}</expression>\n", indent, term, binary_operators_and_terms, indent);
}

fn parse_term(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let mut term = "".to_string();
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 32-parse_term: invalid term: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type == "integer_constant".to_string() {
        term = format!("\t{}<integerConstant> {} </integerConstant>\n", indent, lex_vec[*iter].token);
        *iter += 1;
    } else if lex_vec[*iter].token_type == "string_constant".to_string() {
        term = format!("\t{}<stringConstant> {} </stringConstant>\n", indent, lex_vec[*iter].token);
        *iter += 1;
    } else if lex_vec[*iter].token_type == "keyword".to_string() {
        term = parse_keyword_constant(lex_vec, iter, &new_indent, filename);
    } else if lex_vec[*iter].token_type == "symbol".to_string() {
        if lex_vec[*iter].token == "-".to_string() || lex_vec[*iter].token == "~".to_string() { 
            term += &parse_unary_operator(lex_vec, iter, &new_indent, filename);
            term += &parse_term(lex_vec, iter, &new_indent, filename);
        } else if lex_vec[*iter].token == "(".to_string() {
            term += &token_processer("(".to_string(), lex_vec, iter, &new_indent, filename);
            term += &parse_expression(lex_vec, iter, &new_indent, filename);
            term += &token_processer(")".to_string(), lex_vec, iter, &new_indent, filename);
        }
    } else if lex_vec[*iter].token_type == "identifier".to_string() {
        if lex_vec.len() <= *iter + 1 {
            panic!("syntax error: code 33-parse_term: invalid term: in {} line {}", filename, lex_vec[*iter].row_number);
        } else if lex_vec[*iter + 1].token == "[".to_string() && lex_vec[*iter + 1].token_type == "symbol".to_string() {
            term += &parse_var_name(lex_vec, iter, &new_indent, filename);
            term += &token_processer("[".to_string(), lex_vec, iter, &new_indent, filename);
            term += &parse_expression(lex_vec, iter, &new_indent, filename);
            term += &token_processer("]".to_string(), lex_vec, iter, &new_indent, filename);
        } else if (lex_vec[*iter + 1].token == "(".to_string() || lex_vec[*iter + 1].token == ".".to_string()) && lex_vec[*iter + 1].token_type == "symbol".to_string() {
            term = parse_subroutine_call(lex_vec, iter, indent, filename);
        } else {
            term = parse_var_name(lex_vec, iter, &new_indent, filename);
        }
    }
    return format!("{}<term>\n{}{}</term>\n", indent, term, indent);
}

fn parse_keyword_constant(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let keyword_constant = vec!["true".to_string(), "false".to_string(), "null".to_string(), "this".to_string()];

    if lex_vec.len() <= *iter {
        panic!("syntax error: code 34-parse_keyword_constant: invalid keyword constant: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type != "keyword".to_string() {
        panic!("syntax error: code 35-parse_keyword_constant: invalid keyword constant: in {} line {}", filename, lex_vec[*iter].row_number);
    } else if keyword_constant.contains(&lex_vec[*iter].token) {
        let xml = format!("{}<keyword> {} </keyword>\n", indent, lex_vec[*iter].token);
        *iter += 1;
        return xml;
    } else {
        panic!("syntax error: invalid keyword constant: in {} line {}", filename, lex_vec[*iter].row_number);
    }
}

fn parse_unary_operator(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 36-parse_unary_operator: invalid operator: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if lex_vec[*iter].token_type != "symbol".to_string() {
        panic!("syntax error: code 37-parse_unary_operator: invalid operator: in {} line {}", filename, lex_vec[*iter].row_number);
    } else if lex_vec[*iter].token == "-".to_string() {
        let xml = format!("{}<symbol> - </symbol>\n", indent);
        *iter += 1;
        return xml;
    } else if lex_vec[*iter].token == "~".to_string() {
        let xml = format!("{}<symbol> ~ </symbol>\n", indent);
        *iter += 1;
        return xml;
    } else {
        panic!("syntax error: code 38-parse_unary_operator: invalid operator: in {} line {}", filename, lex_vec[*iter].row_number);
    }
}

fn parse_subroutine_call(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let mut subroutine = "".to_string();
    if lex_vec.len() <= *iter + 1 {
        panic!("syntax error: code 39-parse_subroutine_call: invalid function call: in {} line {}", filename, lex_vec[*iter].row_number);
    } else if lex_vec[*iter + 1].token_type != "symbol".to_string() {
        panic!("syntax error: code 40-parse_subroutine_call: invalid function call: in {} line {}", filename, lex_vec[*iter].row_number);
    } else if lex_vec[*iter + 1].token == "(".to_string() {
        subroutine += &parse_subroutine_name(lex_vec, iter, &new_indent, filename);
        subroutine += &token_processer("(".to_string(), lex_vec, iter, &new_indent, filename);
        subroutine += &parse_expression_list(lex_vec, iter, &new_indent, filename);
        subroutine += &token_processer(")".to_string(), lex_vec, iter, &new_indent, filename);
    } else if lex_vec[*iter + 1].token == ".".to_string() {
        // class_name or var_name
        subroutine += &parse_class_name(lex_vec, iter, &new_indent, filename);
        subroutine += &token_processer(".".to_string(), lex_vec, iter, &new_indent, filename);
        subroutine += &parse_subroutine_name(lex_vec, iter, &new_indent, filename);
        subroutine += &token_processer("(".to_string(), lex_vec, iter, &new_indent, filename);
        subroutine += &parse_expression_list(lex_vec, iter, &new_indent, filename);
        subroutine += &token_processer(")".to_string(), lex_vec, iter, &new_indent, filename);
    } else {
        panic!("syntax error: code 41-parse_subroutine_call: invalid function call: in {} line {}", filename, lex_vec[*iter].row_number);
    }
    return subroutine;
}

fn parse_expression_list(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let new_indent = format!("{}\t", indent);

    let mut list = "".to_string();
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 42-parse_expression_list: invalid argument: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if !(lex_vec[*iter].token_type == "symbol".to_string() && lex_vec[*iter].token == ")".to_string()) {
        list += &parse_expression(lex_vec, iter, &new_indent, filename);
        loop {
            if lex_vec.len() <= *iter {
                panic!("syntax error: code 43-parse_expression_list: invalid argument: in {} line {}", filename, lex_vec[*iter-1].row_number);
            } else if lex_vec[*iter].token_type == "symbol".to_string() && lex_vec[*iter].token == ",".to_string() {
                list += &token_processer(",".to_string(), lex_vec, iter, &new_indent, filename);
                list += &parse_expression(lex_vec, iter, &new_indent, filename);
            } else {
                break;
            }
        }
    }
    return format!("{}<expressionList>\n{}{}</expressionList>\n", indent, list, indent);
}

fn parse_operator(lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    let operator_type = vec!["+".to_string(), "-".to_string(), "*".to_string(), "/".to_string(), "&".to_string(), "|".to_string(), "<".to_string(), ">".to_string(), "=".to_string()];
    if lex_vec.len() <= *iter {
        panic!("syntax error: code 44-parse_operator: invalid operator: in {} line {}", filename, lex_vec[*iter-1].row_number);
    } else if !(operator_type.contains(&lex_vec[*iter].token)) {
        panic!("syntax error: code 45-parse_operator: invalid operator: in {} line {}", filename, lex_vec[*iter].row_number);
    } else {
        let mut xml = format!("{}<symbol> {} </symbol>\n", indent, lex_vec[*iter].token);
        if lex_vec[*iter].token == "<".to_string() {
            xml = format!("{}<symbol> &lt; </symbol>\n", indent);
        } else if lex_vec[*iter].token == ">".to_string() {
            xml = format!("{}<symbol> &gt; </symbol>\n", indent);
        } else if lex_vec[*iter].token == "&".to_string() {
            xml = format!("{}<symbol> &amp; </symbol>\n", indent);
        }
        *iter += 1;
        return xml;
    }
}

// subroutine
fn token_processer(token: String, lex_vec: &Vec<Lexicon>, iter: &mut usize, indent: &String, filename: &str) -> String {
    if *iter >= lex_vec.len() {
        panic!("syntax error: code 01: invalid '{}': in {} line {}", token, filename, lex_vec[*iter-1].row_number);
    } else if !(lex_vec[*iter].token_type == "keyword".to_string() || lex_vec[*iter].token_type == "symbol".to_string()) {
        panic!("syntax error: code 02: invalid '{}': in {} line {}", token, filename, lex_vec[*iter].row_number);
    } else if token == lex_vec[*iter].token {
        let xml = format!("{}<{}> {} </{}>\n", indent, lex_vec[*iter].token_type, token, lex_vec[*iter].token_type);
        *iter += 1;
        return xml;
    } else {
        panic!("syntax error: code 03: invalid '{}': in {} line {}", token, filename, lex_vec[*iter].row_number);
    }
}