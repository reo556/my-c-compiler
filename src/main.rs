use std::env;
use std::fs::File;
use std::io::Write;

// --- 1. 字句解析（トークナイザ）---
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(i32),
    Ident(String),
    StringLiteral(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Semicolon,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '-' => { tokens.push(Token::Minus); chars.next(); }
            '*' => { tokens.push(Token::Star); chars.next(); }
            '/' => { tokens.push(Token::Slash); chars.next(); }
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            ';' => { tokens.push(Token::Semicolon); chars.next(); }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        break;
                    }
                    s.push(ch);
                    chars.next();
                }
                chars.next();
                tokens.push(Token::StringLiteral(s));
            }
            '0'..='9' => {
                let mut num_str = String::new();
                while let Some(&digit) = chars.peek() {
                    if digit.is_ascii_digit() {
                        num_str.push(digit);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Num(num_str.parse().unwrap()));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident_str));
            }
            _ => {
                eprintln!("トークナイズエラー: 未知の文字 '{}'", c);
                std::process::exit(1);
            }
        }
    }
    tokens
}

// --- 2. 構文解析（パーサ）---
#[derive(Debug)]
enum Node {
    Num(i32),
    StringLiteral(usize, String),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    FuncCall(String, Vec<Node>),
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    string_literals: Vec<(usize, String)>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0, string_literals: Vec::new() }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn parse(&mut self) -> Node {
        let node = self.expr();
        if let Some(Token::Semicolon) = self.peek() {
            self.next();
        }
        node
    }

    fn expr(&mut self) -> Node {
        let mut node = self.mul();
        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.next();
                    let rhs = self.mul();
                    node = Node::Add(Box::new(node), Box::new(rhs));
                }
                Token::Minus => {
                    self.next();
                    let rhs = self.mul();
                    node = Node::Sub(Box::new(node), Box::new(rhs));
                }
                _ => break,
            }
        }
        node
    }

    fn mul(&mut self) -> Node {
        let mut node = self.primary();
        while let Some(tok) = self.peek() {
            match tok {
                Token::Star => {
                    self.next();
                    let rhs = self.primary();
                    node = Node::Mul(Box::new(node), Box::new(rhs));
                }
                Token::Slash => {
                    self.next();
                    let rhs = self.primary();
                    node = Node::Div(Box::new(node), Box::new(rhs));
                }
                _ => break,
            }
        }
        node
    }

    fn primary(&mut self) -> Node {
        match self.next().cloned() {
            Some(Token::Num(val)) => Node::Num(val),
            Some(Token::StringLiteral(s)) => {
                let id = self.string_literals.len();
                self.string_literals.push((id, s.clone()));
                Node::StringLiteral(id, s)
            }
            Some(Token::Ident(name)) => {
                if let Some(Token::LParen) = self.peek() {
                    self.next();
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        args.push(self.expr());
                    }
                    if self.next() != Some(&Token::RParen) {
                        eprintln!("構文エラー: ')' がありません");
                        std::process::exit(1);
                    }
                    Node::FuncCall(name, args)
                } else {
                    eprintln!("未知の識別子です: {}", name);
                    std::process::exit(1);
                }
            }
            Some(Token::LParen) => {
                let node = self.expr();
                if self.next() != Some(&Token::RParen) {
                    eprintln!("構文エラー: ')' が見つかりません");
                    std::process::exit(1);
                }
                node
            }
            tok => {
                eprintln!("構文エラー: 予期しないトークン {:?}", tok);
                std::process::exit(1);
            }
        }
    }
}

// --- 3. コード生成 ---
fn generate(node: &Node, asm: &mut String) {
    match node {
        Node::Num(val) => {
            asm.push_str(&format!("\tpush {}\n", val));
        }
        Node::StringLiteral(id, _) => {
            asm.push_str(&format!("\tlea rax, .LC{}[rip]\n\tpush rax\n", id));
        }
        Node::Add(lhs, rhs) => {
            generate(lhs, asm);
            generate(rhs, asm);
            asm.push_str("\tpop rdi\n\tpop rax\n\tadd rax, rdi\n\tpush rax\n");
        }
        Node::Sub(lhs, rhs) => {
            generate(lhs, asm);
            generate(rhs, asm);
            asm.push_str("\tpop rdi\n\tpop rax\n\tsub rax, rdi\n\tpush rax\n");
        }
        Node::Mul(lhs, rhs) => {
            generate(lhs, asm);
            generate(rhs, asm);
            asm.push_str("\tpop rdi\n\tpop rax\n\timul rax, rdi\n\tpush rax\n");
        }
        Node::Div(lhs, rhs) => {
            generate(lhs, asm);
            generate(rhs, asm);
            asm.push_str("\tpop rdi\n\tpop rax\n\tcqo\n\tidiv rdi\n\tpush rax\n");
        }
        Node::FuncCall(name, args) => {
            if !args.is_empty() {
                generate(&args[0], asm);
                asm.push_str("\tpop rdi\n");
            }
            asm.push_str("\tmov eax, 0\n");
            asm.push_str(&format!("\tcall {}\n", name));
            asm.push_str("\tpush rax\n");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使い方: cargo run -- \"<C言語コード>\"");
        std::process::exit(1);
    }

    let input = &args[1];
    let tokens = tokenize(input);

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    let mut assembly = String::new();

    if !parser.string_literals.is_empty() {
        assembly.push_str("\t.section .rodata\n");
        for (id, s) in &parser.string_literals {
            assembly.push_str(&format!(".LC{}:\n\t.string \"{}\"\n", id, s));
        }
    }

    assembly.push_str(
        "\t.text\n\
\t.intel_syntax noprefix\n\
\t.globl main\n\
main:\n",
    );

    generate(&ast, &mut assembly);

    assembly.push_str("\tpop rax\n\tret\n");

    let mut file = File::create("out.s").expect("ファイルの作成に失敗しました");
    file.write_all(assembly.as_bytes()).expect("ファイルの書き込みに失敗しました");

    println!("コンパイル成功: 式 \"{}\" から out.s を生成しました", input);
}
