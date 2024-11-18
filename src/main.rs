use std::{error::Error, fmt::Display, path::PathBuf};

fn main() {
    let documents_dir = dirs::document_dir().unwrap();
    let documents_dir = documents_dir.join("input.hawk");

    fn tokenize(file_path: PathBuf) -> Tokens {
        Tokens::new(std::fs::read_to_string(file_path).unwrap())
    }

    let mut tokens = tokenize(documents_dir);

    match tokens.program() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    }
}

struct Tokens {
    tokens: String,
    cursor: usize,
    line_number: usize,
}

impl Tokens {
    fn new(tokens: String) -> Tokens {
        Tokens {
            tokens,
            cursor: 0,
            line_number: 0,
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

    fn debug(&self) {
        let rest_of_input = self.tokens.get(self.cursor..).unwrap();
        println!("rest of input: {}", rest_of_input);
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

    fn syntax_error(&self, message: String) -> Result<(), SyntaxError> {
        Err(SyntaxError::new(message, self.line_number))
    }

    fn save_state(&self) -> TokensState {
        TokensState {
            cursor: self.cursor,
            line_number: self.line_number,
        }
    }

    fn restore_state(&mut self, state: TokensState) {
        self.cursor = state.cursor;
        self.line_number = state.line_number;
    }
}

#[derive(Debug)]
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
        write!(
            f,
            "Syntax Error: {} at line number {}.",
            self.message, self.line_number
        )
    }
}

impl Error for SyntaxError {}

#[derive(Debug)]
struct TokensState {
    cursor: usize,
    line_number: usize,
}

impl Tokens {
    // Rule 01: PROGRAM -> program DECL_SEC begin STMT_SEC end; | program begin STMT_SEC end;
    fn program(&mut self) -> Result<(), SyntaxError> {
        println!("PROGRAM");
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
        println!("DECL_SEC");
        self.decl()?;
        let state = self.save_state();
        if self.decl_sec().is_err() {
            // if this errors, we have reached the end of the decl_sec. ignore the error.
            self.restore_state(state);
        }
        Ok(())
    }

    // Rule 03: DECL -> ID_LIST : TYPE ;
    fn decl(&mut self) -> Result<(), SyntaxError> {
        println!("DECL");
        self.id_list()?;
        self.next(":")?;
        self._type()?;
        self.next(";")?;
        Ok(())
    }

    // Rule 04: ID_LIST -> ID | ID , ID_LIST
    fn id_list(&mut self) -> Result<(), SyntaxError> {
        println!("ID_LIST");
        self.id()?;
        if self.peek(",") {
            self.next(",")?;
            self.id_list()?;
        }
        Ok(())
    }

    // Rule 05: ID -> (_ | a | b | ... | z | A | ... | Z) (_ | a | b | ... | z | A | ... | Z | 0 | 1 | ... | 9)*
    fn id(&mut self) -> Result<(), SyntaxError> {
        println!("ID");
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

        println!("identifier: \"{}\"", identifier);

        Ok(())
    }

    // Rule 06: STMT_SEC -> STMT | STMT STMT_SEC
    fn stmt_sec(&mut self) -> Result<(), SyntaxError> {
        println!("STMT_SEC");
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
        println!("STMT");
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
        println!("ASSIGN");
        self.id()?;
        self.next(":=")?;
        self.expr()?;
        self.next(";")?;
        Ok(())
    }

    // Rule 09: IFSTMT -> if COMP then STMT_SEC end if ; | if COMP then STMT_SEC else STMT_SEC end if ;
    fn if_stmt(&mut self) -> Result<(), SyntaxError> {
        println!("IFSTMT");
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
        println!("WHILESTMT");
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
        println!("INPUT");
        self.next("input")?;
        self.id_list()?;
        self.next(";")?;
        Ok(())
    }

    // Rule 12: OUTPUT -> output ID_LIST | output NUM;
    fn output(&mut self) -> Result<(), SyntaxError> {
        println!("OUTPUT");
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
        println!("EXPR");
        self.factor()?;
        self.debug();
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
        println!("FACTOR");
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
        println!("OPERAND");
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
        println!("NUM");
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
        println!("COMP");
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
        println!("TYPE");
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
