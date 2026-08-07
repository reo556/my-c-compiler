use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    // 実行時の引数を取得（例: cargo run -- main.s）
    let args: Vec<String> = env::args().collect();
    let output_filename = if args.len() > 1 {
        &args[1]
    } else {
        "out.s"
    };

    // 出力するアセンブリ言語のコードを組み立てる
    // (return 42; に相当するx86_64 AT&T記法のアセンブリ)
    let assembly = "\
\t.globl main
main:
\tmovl $42, %eax
\tret
";

    // アセンブリファイル (.s) を書き出し
    let mut file = File::create(output_filename).expect("ファイルの作成に失敗しました");
    file.write_all(assembly.as_bytes()).expect("ファイルの書き込みに失敗しました");

    println!("コンパイル成功: {} を生成しました", output_filename);
}
