use std::env;
use std::fs::File;
use std::io::Write;

// --- 1. 字句解析（トークナイザ）---
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(i32),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
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
            _ => {
                eprintln!("トークナイズエラー: 未知の文字 '{}'", c);
                std::process::exit(1);
            }
        }
    }
    tokens
}

// --- 2. 構文解析（パーサ & 構文木 AST）---
#[derive(Debug)]
enum Node {
    Num(i32),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
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

    // expr = mul ("+" mul | "-" mul)*
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

    // mul = primary ("*" primary | "/" primary)*
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

    // primary = num | "(" expr ")"
    fn primary(&mut self) -> Node {
        match self.next() {
            Some(Token::Num(val)) => Node::Num(*val),
            Some(Token::LParen) => {
                let node = self.expr();
                match self.next() {
                    Some(Token::RParen) => node,
                    _ => {
                        eprintln!("構文エラー: ')' が見つかりません");
                        std::process::exit(1);
                    }
                }
            }
            tok => {
                eprintln!("構文エラー: 予期しないトークン {:?}", tok);
                std::process::exit(1);
            }
        }
    }
}

// --- 3. コード生成（スタックマシン）---
fn generate(node: &Node, asm: &mut String) {
    match node {
        Node::Num(val) => {
            asm.push_str(&format!("\tpush {}\n", val));
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
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使い方: cargo run -- \"<式>\"");
        std::process::exit(1);
    }

    let input = &args[1];
    let tokens = tokenize(input);

    let mut parser = Parser::new(tokens);
    let ast = parser.expr();

    let mut assembly = String::from(
        "\t.intel_syntax noprefix\n\
\t.globl main\n\
main:\n",
    );

    generate(&ast, &mut assembly);

    // 最終結果を rax に取り出してリターン
    assembly.push_str("\tpop rax\n\tret\n");

    let mut file = File::create("out.s").expect("ファイルの作成に失敗しました");
    file.write_all(assembly.as_bytes()).expect("ファイルの書き込みに失敗しました");

    println!("コンパイル成功: 式 \"{}\" から out.s を生成しました", input);
}
