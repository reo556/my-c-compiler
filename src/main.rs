use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("使い方: cargo run -- \"<式>\"");
        std::process::exit(1);
    }

    let input = &args[1];
    let mut chars = input.chars().peekable();

    // 1. 最初のアセンブリヘッダーを出力
    let mut assembly = String::from(
        "\t.intel_syntax noprefix\n\
\t.globl main\n\
main:\n",
    );

    // 2. 最初の数値を読み込んで mov eax, <数値> を生成
    let first_num = parse_number(&mut chars);
    assembly.push_str(&format!("\tmov eax, {}\n", first_num));

    // 3. 残りの `+ 数値` や `- 数値` をループで処理
    while let Some(&ch) = chars.peek() {
        if ch == ' ' {
            chars.next();
            continue;
        }

        if ch == '+' {
            chars.next();
            let num = parse_number(&mut chars);
            assembly.push_str(&format!("\tadd eax, {}\n", num));
        } else if ch == '-' {
            chars.next();
            let num = parse_number(&mut chars);
            assembly.push_str(&format!("\tsub eax, {}\n", num));
        } else {
            eprintln!("予期しない文字です: {}", ch);
            std::process::exit(1);
        }
    }

    // 4. 関数から復帰
    assembly.push_str("\tret\n");

    // out.s に書き出し
    let mut file = File::create("out.s").expect("ファイルの作成に失敗しました");
    file.write_all(assembly.as_bytes()).expect("ファイルの書き込みに失敗しました");

    println!("コンパイル成功: 式 \"{}\" から out.s を生成しました", input);
}

// 文字列から連続する数字（例: "123"）を読み取る補助関数
fn parse_number<I>(chars: &mut std::iter::Peekable<I>) -> i32
where
    I: Iterator<Item = char>,
{
    let mut num_str = String::new();

    // 空白をスキップ
    while let Some(&ch) = chars.peek() {
        if ch == ' ' {
            chars.next();
        } else {
            break;
        }
    }

    // 数字を収集
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    if num_str.is_empty() {
        eprintln!("数値を期待していましたが、見つかりませんでした");
        std::process::exit(1);
    }

    num_str.parse().unwrap()
}
