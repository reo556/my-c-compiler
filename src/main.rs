use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("使い方: cargo run -- <数値>");
        std::process::exit(1);
    }

    let number: i32 = args[1].parse().expect("数値を指定してください");

    // Intel表記のアセンブリコードを動的に生成
    let assembly = format!(
        "\t.intel_syntax noprefix\n\
\t.globl main\n\
main:\n\
\tmov eax, {}\n\
\tret\n",
        number
    );

    let mut file = File::create("out.s").expect("ファイルの作成に失敗しました");
    file.write_all(assembly.as_bytes()).expect("ファイルの書き込みに失敗しました");

    println!("コンパイル成功 (Intel表記): {} を返す out.s を生成しました", number);
}
