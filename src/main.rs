use std::{collections::HashMap, error::Error, fmt::Display, path::PathBuf};

fn main() {
    let documents_dir = dirs::document_dir().unwrap();
    let documents_dir = documents_dir.join("input.hawk");

    fn tokenize(file_path: PathBuf) -> Tokens {
        Tokens::new(std::fs::read_to_string(file_path).unwrap())
    }

    let mut tokens = tokenize(documents_dir);
    let result = tokens.program();

    for output in &tokens.outputs {
        println!("{}", output);
    }

    match tokens.check_consisitency() {
        Ok(_) => match result {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        },
        Err(e) => eprintln!("{}", e),
    };
}

struct Tokens {
    tokens: String,
    cursor: usize,
    line_number: usize,
    outputs: Vec<&'static str>,
    decleration_section: bool,
    declared_variables: HashMap<String, bool>,
    critical_error: Result<(), SyntaxError>,
}

impl Tokens {
    fn new(tokens: String) -> Tokens {
        Tokens {
            tokens,
            cursor: 0,
            line_number: 1,
            outputs: Vec::new(),
            decleration_section: false,
            declared_variables: HashMap::new(),
            critical_error: Ok(()),
        }
    }

    // skips whitespace and advances the line number.
    fn skip_whitespace(&mut self) {
        for c in self.tokens.chars().skip(self.cursor) {
            if c == '\n' {
                self.line_number += 1;
                self.cursor += 1;
            } else if c.is_whitespace() {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    #[allow(dead_code)]
    fn debug(&self) {
        let rest_of_input = self.tokens.get(self.cursor..).unwrap();
        println!("{}", rest_of_input);
    }

    // consumes the next token. errors if the token does not match the search query.
    fn next(&mut self, search_query: &str) -> Result<(), SyntaxError> {
        if self.peek(search_query) {
            self.cursor += search_query.len();
            Ok(())
        } else {
            self.syntax_error(format!("unexpected token: \"{}\"", search_query))
        }
    }

    // previews the next token matches the search query without consuming it.
    fn peek(&mut self, search_query: &str) -> bool {
        self.skip_whitespace(); // skip any whitespace before the token

        let token = self
            .tokens
            .get(self.cursor..self.cursor + search_query.len());

        match token {
            Some(token) => token == search_query,
            None => false,
        }
    }

    fn check_consisitency(&self) -> Result<(), SyntaxError> {
        match &self.critical_error {
            Err(e) => Err(e.clone()),
            Ok(_) => Ok(()),
        }
    }

    fn syntax_error(&self, message: String) -> Result<(), SyntaxError> {
        Err(SyntaxError::new(message, self.line_number))
    }

    fn push(&mut self, output: &'static str) {
        self.outputs.push(output);
    }

    fn save_state(&self) -> Self {
        Self {
            cursor: self.cursor,
            line_number: self.line_number,
            outputs: self.outputs.clone(),
            decleration_section: self.decleration_section,
            declared_variables: self.declared_variables.clone(),
            tokens: self.tokens.clone(),
            critical_error: self.critical_error.clone(),
        }
    }

    fn restore_state(&mut self, state: Self) {
        if self.critical_error.is_err() {
            return;
        }

        self.cursor = state.cursor;
        self.line_number = state.line_number;
        self.outputs = state.outputs;
        self.decleration_section = state.decleration_section;
        self.declared_variables = state.declared_variables;
    }
}

#[derive(Debug, Clone)]
struct SyntaxError {
    message: String,
    line_number: usize,
}

impl SyntaxError {
    fn new(message: String, line_number: usize) -> SyntaxError {
        SyntaxError {
            message,
            line_number,
        }
    }
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR !! {} in Line {}.", self.message, self.line_number)
    }
}

impl Error for SyntaxError {}

impl Tokens {
    // Rule 01: PROGRAM -> program DECL_SEC begin STMT_SEC end; | program begin STMT_SEC end;
    fn program(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("PROGRAM");
        self.next("program")?;
        if self.peek("begin") {
            self.next("begin")?;
        } else {
            self.decl_sec()?;
            self.next("begin")?;
        }
        self.stmt_sec()?;
        self.next("end")?;
        self.next(";")?;
        Ok(())
    }

    // Rule 02: DECL_SEC -> DECL | DECL DECL_SEC
    fn decl_sec(&mut self) -> Result<(), SyntaxError> {
        self.decleration_section = true;

        self.push("DECL_SEC");
        self.decl()?;
        let state = self.save_state();
        if self.decl_sec().is_err() {
            // if this errors, we have reached the end of the decl_sec. ignore the error.
            self.restore_state(state);
        }

        self.decleration_section = false;
        Ok(())
    }

    // Rule 03: DECL -> ID_LIST : TYPE ;
    fn decl(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("DECL");
        self.id_list()?;
        self.next(":")?;
        self._type()?;
        self.next(";")?;
        Ok(())
    }

    // Rule 04: ID_LIST -> ID | ID , ID_LIST
    fn id_list(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("ID_LIST");
        self.id()?;
        if self.peek(",") {
            self.next(",")?;
            self.id_list()?;
        }
        Ok(())
    }

    // Rule 05: ID -> (_ | a | b | ... | z | A | ... | Z) (_ | a | b | ... | z | A | ... | Z | 0 | 1 | ... | 9)*
    fn id(&mut self) -> Result<(), SyntaxError> {
        self.skip_whitespace();
        let mut identifier = String::new();
        let mut first_char = true;
        for c in self.tokens.chars().skip(self.cursor) {
            if c.is_alphabetic() || c == '_' {
                self.cursor += 1;
                identifier.push(c);
                first_char = false;
            } else if c.is_ascii_digit() && !first_char {
                self.cursor += 1;
                identifier.push(c);
            } else {
                break;
            }
        }

        if first_char {
            return self.syntax_error("expected an identifier".to_string());
        }

        if matches!(
            identifier.as_str(),
            "program"
                | "begin"
                | "end"
                | "if"
                | "then"
                | "else"
                | "while"
                | "loop"
                | "input"
                | "output"
                | "int"
                | "float"
                | "double"
        ) {
            return self.syntax_error(format!("{identifier} is a reserved keyword"));
        }

        if self.decleration_section {
            self.declared_variables.insert(identifier, true);
        } else if !self.declared_variables.contains_key(&identifier) && self.critical_error.is_ok()
        {
            self.critical_error = self.syntax_error("identifier not declared".to_string());
        }

        Ok(())
    }

    // Rule 06: STMT_SEC -> STMT | STMT STMT_SEC
    fn stmt_sec(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("STMT_SEC");
        self.stmt()?;
        let state = self.save_state();
        if self.stmt_sec().is_err() {
            // if this errors, we have reached the end of the stmt_sec. ignore the error.
            self.restore_state(state);
        }
        Ok(())
    }

    // Rule 07: STMT -> ASSIGN | IFSTMT | WHILESTMT | INPUT | OUTPUT
    fn stmt(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("STMT");
        let state = self.save_state();
        if self.assign().is_ok() {
            return Ok(());
        }
        self.restore_state(state);
        let state = self.save_state();
        if self.if_stmt().is_ok() {
            return Ok(());
        }
        self.restore_state(state);
        let state = self.save_state();
        if self.while_stmt().is_ok() {
            return Ok(());
        }
        self.restore_state(state);
        let state = self.save_state();
        if self.input().is_ok() {
            return Ok(());
        }
        self.restore_state(state);
        let state = self.save_state();
        if self.output().is_ok() {
            return Ok(());
        }
        self.restore_state(state);
        self.syntax_error("expected a statement".to_string())
    }

    // Rule 08: ASSIGN -> ID := EXPR ;
    fn assign(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("ASSIGN");
        self.id()?;
        self.next(":=")?;
        self.expr()?;
        self.next(";")?;
        Ok(())
    }

    // Rule 09: IFSTMT -> if COMP then STMT_SEC end if ; | if COMP then STMT_SEC else STMT_SEC end if ;
    fn if_stmt(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("IF_STMT");
        self.next("if")?;
        self.comp()?;
        self.next("then")?;
        self.stmt_sec()?;
        if self.peek("else") {
            self.next("else")?;
            self.stmt_sec()?;
        }
        self.next("end")?;
        self.next("if")?;
        self.next(";")?;
        Ok(())
    }

    // Rule 10: WHILESTMT -> while COMP loop STMT_SEC end loop ;
    fn while_stmt(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("WHILE_STMT");
        self.next("while")?;
        self.comp()?;
        self.next("loop")?;
        self.stmt_sec()?;
        self.next("end")?;
        self.next("loop")?;
        self.next(";")?;
        Ok(())
    }

    // Rule 11: INPUT -> input ID_LIST;
    fn input(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("INPUT");
        self.next("input")?;
        self.id_list()?;
        self.next(";")?;
        Ok(())
    }

    // Rule 12: OUTPUT -> output ID_LIST | output NUM;
    fn output(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("OUTPUT");
        self.next("output")?;
        let state = self.save_state();
        if self.id_list().is_ok() {
            self.next(";")?;
            return Ok(());
        }
        self.restore_state(state);
        self.num()?;
        self.next(";")?;
        Ok(())
    }

    // Rule 13: EXPR -> FACTOR | FACTOR + EXPR | FACTOR - EXPR
    fn expr(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("EXPR");
        self.factor()?;
        if self.peek("+") {
            self.next("+")?;
            self.expr()?;
        } else if self.peek("-") {
            self.next("-")?;
            self.expr()?;
        }
        Ok(())
    }

    // Rule 14: FACTOR -> OPERAND | OPERAND * FACTOR | OPERAND / FACTOR
    fn factor(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("FACTOR");
        self.operand()?;
        if self.peek("*") {
            self.next("*")?;
            self.factor()?;
        } else if self.peek("/") {
            self.next("/")?;
            self.factor()?;
        }
        Ok(())
    }

    // Rule 15: OPERAND -> NUM | ID | ( EXPR )
    fn operand(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("OPERAND");
        let state = self.save_state();
        if self.num().is_ok() {
            return Ok(());
        }
        self.restore_state(state);
        let state = self.save_state();
        if self.id().is_ok() {
            return Ok(());
        }
        self.restore_state(state);
        self.next("(")?;
        self.expr()?;
        self.next(")")?;
        Ok(())
    }

    // Rule 16: NUM -> (0 | 1 | ... | 9)+ [.(0 | 1 | ... | 9)+]
    fn num(&mut self) -> Result<(), SyntaxError> {
        let mut decimel = false;
        let mut seen_any = false;

        self.skip_whitespace();
        for c in self.tokens.chars().skip(self.cursor) {
            if c.is_ascii_digit() {
                seen_any = true;
                self.cursor += 1;
            } else if c == '.' {
                if decimel {
                    break;
                }
                decimel = true;
                seen_any = false;
                self.cursor += 1;
            } else {
                break;
            }
        }

        if !seen_any {
            return self.syntax_error("expected a number".to_string());
        }
        Ok(())
    }

    // Rule 17: COMP -> ( OPERAND = OPERAND ) | ( OPERAND <> OPERAND ) | ( OPERAND > OPERAND ) | ( OPERAND < OPERAND )
    fn comp(&mut self) -> Result<(), SyntaxError> {
        self.check_consisitency()?;
        self.push("COMP");
        self.next("(")?;
        self.operand()?;
        if self.peek("=") {
            self.next("=")?;
            self.operand()?;
            self.next(")")?;
            Ok(())
        } else if self.peek("<>") {
            self.next("<>")?;
            self.operand()?;
            self.next(")")?;
            Ok(())
        } else if self.peek(">") {
            self.next(">")?;
            self.operand()?;
            self.next(")")?;
            Ok(())
        } else if self.peek("<") {
            self.next("<")?;
            self.operand()?;
            self.next(")")?;
            Ok(())
        } else {
            self.syntax_error("expected a comparison operator".to_string())
        }
    }

    // Rule 18: TYPE -> int | float | double
    fn _type(&mut self) -> Result<(), SyntaxError> {
        if self.peek("int") {
            Ok(self.next("int")?)
        } else if self.peek("float") {
            Ok(self.next("float")?)
        } else if self.peek("double") {
            Ok(self.next("double")?)
        } else {
            self.syntax_error(
                "all declarations must have a type of int, float, or double".to_string(),
            )
        }
    }
}
