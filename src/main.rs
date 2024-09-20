use std::{error::Error, fmt::Display, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let documents_dir = get_documents_dir();
    let documents_dir = documents_dir.join("input.hawk");
    let mut tokens = tokenize(documents_dir);
    tokens.program()?;

    Ok(())
}

fn get_documents_dir() -> PathBuf {
    dirs::document_dir().unwrap()
}

struct Tokens {
    tokens: String,
    cursor: isize,
}

impl Tokens {
    fn new(tokens: String, cursor: isize) -> Tokens {
        Tokens { tokens, cursor }
    }

    fn next(&mut self) -> Result<&str, SyntaxError> {
        let next = &self.tokens.get(self.cursor as usize);
        self.cursor += 1;
        match next {
            Some(token) => Ok(token),
            None => {
                let err = SyntaxError::new("unexpected end of file");
                Err(err)
            }
        }
    }

    fn peek(&self) -> Result<&str, SyntaxError> {
        match &self.tokens.get(self.cursor as usize) {
            Some(token) => Ok(token),
            None => {
                let err = SyntaxError::new("unexpected end of file");
                Err(err)
            }
        }
    }
}

#[derive(Debug)]
struct SyntaxError {
    message: &'static str,
}

impl SyntaxError {
    fn new(message: &'static str) -> SyntaxError {
        SyntaxError { message }
    }
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Syntax Error: {}", self.message)
    }
}

impl Error for SyntaxError {}

impl Tokens {
    fn program(&mut self) -> Result<(), SyntaxError> {
        println!("PROGRAM");
        if self.next()? != "program" {
            let err = SyntaxError::new("all programs must start with the program keyword");
            return Err(err);
        }

        if self.peek()? == "begin" {
            self.next()?;
        } else {
            self.decl_sec()?;
            if self.next()? != "begin" {
                let err = SyntaxError::new("all programs must start with the begin keyword");
                return Err(err);
            }
        }

        self.stmt_sec()?;
        
        if self.next()? != "end;" {
            let err = SyntaxError::new("all programs must end with the end; keyword");
            return Err(err);
        }

        Ok(())
    }

    fn decl_sec(&mut self) -> Result<(), SyntaxError> {
        println!("DECL_SEC");
        self.decl()?;
        let _ = self.decl_sec(); // if this errors, we have reached the end of the decl_sec. ignore the error.
        Ok(())
    }

    fn decl(&mut self) -> Result<(), SyntaxError> {
        println!("DECL");
        self.id_list()?;
        if self.next()? != ":" {
            let err = SyntaxError::new("all declarations must end with a colon");
            return Err(err);
        }
        self._type()?;
        if self.next()? != ";" {
            let err = SyntaxError::new("all declarations must end with a semicolon");
            return Err(err);
        }
        Ok(())
    }

    fn id_list(&mut self) -> Result<(), SyntaxError> {
        println!("ID_LIST");
        self.id()?;
        if self.peek()? == "," {
            self.next()?;
            self.id_list()?;
        }
        Ok(())
    }

    fn id(&mut self) -> Result<(), SyntaxError> {
        println!("ID");
        let identifier = self.next()?;
        println!("{identifier}");
        let first_letter = match identifier.chars().next() {
            Some(c) => c,
            None => {
                let err = SyntaxError::new("identifiers must have at least one character");
                return Err(err);
            }
        };
        if !first_letter.is_alphabetic() && first_letter != '_' {
            let err = SyntaxError::new("identifiers must start with a letter or underscore");
            return Err(err);
        }
        for c in identifier.chars().skip(1) {
            if !c.is_alphanumeric() && c != '_' {
                let err = SyntaxError::new("identifiers must only contain alphanumeric characters and underscores");
                return Err(err);
            }
        }
        Ok(())
    }

    fn _type(&mut self) -> Result<(), SyntaxError> {
        println!("TYPE");
        match self.next()? {
            "int" | "float" | "double" => Ok(()),
            _ => {
                let err = SyntaxError::new("all declarations must have a type of int, float, or double");
                Err(err)
            }
        }
    }

    fn stmt_sec(&mut self) -> Result<(), SyntaxError> {
        println!("STMT_SEC");
        if self.assign().is_ok() { return Ok(()); }
        if self.if_stmt().is_ok() { return Ok(()); }
        if self.while_stmt().is_ok() { return Ok(()); }
        if self.input().is_ok() { return Ok(()); }
        if self.output().is_ok() { return Ok(()); }
        let err = SyntaxError::new("expected a statement");
        Err(err)
    }

    fn assign(&mut self) -> Result<(), SyntaxError> {
        println!("ASSIGN");
        self.id()?;
        if self.next()? != ":=" {
            let err = SyntaxError::new("all assignments must have a colon followed by an equal sign");
            return Err(err);
        }
        self.expr()?;
        if self.next()? != ";" {
            let err = SyntaxError::new("all assignments must end with a semicolon");
            return Err(err);
        }
        Ok(())
    }

    fn if_stmt(&mut self) -> Result<(), SyntaxError> {
        println!("IFSTMT");
        if self.next()? != "if" {
            let err = SyntaxError::new("all if statements must start with the if keyword");
            return Err(err);
        }
        self.comp()?;
        if self.next()? != "then" {
            let err = SyntaxError::new("all if statements must start with the if keyword");
            return Err(err);
        }
        self.stmt_sec()?;
        if self.peek()? == "else" {
            self.next()?;
            self.stmt_sec()?;
        }
        if self.next()? != "end" {
            let err = SyntaxError::new("all if statements must end with the end keyword");
            return Err(err);
        }
        if self.peek()? == "if;" {
            self.next()?;
            return Ok(())
        }
        if self.next()? != "if" {
            let err = SyntaxError::new("all if statements must end with the if keyword");
            return Err(err);
        }
        Ok(())
    }

    fn while_stmt(&mut self) -> Result<(), SyntaxError> {
        println!("WHILESTMT");
        if self.next()? != "while" {
            let err = SyntaxError::new("all while statements must start with the while keyword");
            return Err(err);
        }
        self.comp()?;
        if self.next()? != "loop" {
            let err = SyntaxError::new("all loop statements must start with the loop keyword");
            return Err(err);
        }
        self.stmt_sec()?;
        if self.next()? != "end" {
            let err = SyntaxError::new("all while statements must end with the end keyword");
            return Err(err);
        }
        if self.peek()? == "loop;" {
            self.next()?;
            return Ok(())
        }
        if self.next()? != "loop" {
            let err = SyntaxError::new("all loop statements must end with the loop keyword");
            return Err(err);
        }
        if self.next()? != ";" {
            let err = SyntaxError::new("all loop statements must end with a semicolon");
            return Err(err);
        }
        Ok(())
    }

    fn comp(&mut self) -> Result<(), SyntaxError> {
        println!("COMP");
        if self.next()? != "(" {
            let err = SyntaxError::new("expected an operand");
            return Err(err);
        }
        self.operand()?;
        match self.next()? {
            "=" | "<>" | ">" | "<" => (),
            _ => {
                let err = SyntaxError::new("expected a comparison operator");
                return Err(err);
            }
        }
        self.operand()?;
        if self.next()? != ")" {
            let err = SyntaxError::new("expected a closing parenthesis");
            return Err(err);
        }
        Ok(())
    }

    fn input(&mut self) -> Result<(), SyntaxError> {
        println!("INPUT");
        if self.next()? != "input" {
            let err = SyntaxError::new("all input statements must start with the input keyword");
            return Err(err);
        }
        self.id_list()?;
        if self.next()? != ";" {
            let err = SyntaxError::new("all input statements must end with a semicolon");
            return Err(err);
        }
        Ok(())
    }

    fn output(&mut self) -> Result<(), SyntaxError> {
        println!("OUTPUT");
        if self.next()? != "output" {
            let err = SyntaxError::new("all output statements must start with the output keyword");
            return Err(err);
        }
        if self.id_list().is_ok() { return Ok(()); }
        self.num()?;
        if self.next()? != ";" {
            let err = SyntaxError::new("all output statements must end with a semicolon");
            return Err(err);
        }
        Ok(())
    }

    fn expr(&mut self) -> Result<(), SyntaxError> {
        println!("EXPR");
        self.factor()?;
        if let Ok(operator) = self.peek() {
            if operator == "+" || operator == "-" {
                self.next()?;
                self.expr()?;
            }
        }
        Ok(())
    }

    fn factor(&mut self) -> Result<(), SyntaxError> {
        println!("FACTOR");
        self.operand()?;
        if let Ok(operator) = self.peek() {
            if operator == "*" || operator == "/" {
                self.next()?;
                self.expr()?;
            }
        }
        Ok(())
    }

    fn operand(&mut self) -> Result<(), SyntaxError> {
        println!("OPERAND");
        if self.num().is_ok() { return Ok(()); }
        if self.id().is_ok() { return Ok(()); }
        if self.next()? != "(" {
            let err = SyntaxError::new("expected an operand");
            return Err(err);
        }
        self.expr()?;
        if self.next()? != ")" {
            let err = SyntaxError::new("expected a closing parenthesis");
            return Err(err);
        }
        Ok(())
    }

    fn num(&mut self) -> Result<(), SyntaxError> {
        println!("NUM");
        let number = self.next()?;
        if number.parse::<f64>().is_err() {
            let err = SyntaxError::new("expected a number");
            return Err(err);
        }
        Ok(())
    }
}

fn tokenize(file_path: PathBuf) -> Tokens {
    let file_contents = std::fs::read_to_string(file_path).unwrap();
    Tokens::new(file_contents, 0)
}