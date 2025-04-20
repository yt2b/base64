use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

const STRING: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn create_table() -> HashMap<u8, u8> {
    // 6ビットの値をキーにBase64の文字を返すハッシュマップを生成する
    STRING
        .chars()
        .enumerate()
        .map(|(i, ch)| (i as u8, ch as u8))
        .collect()
}

fn create_reversed_table() -> HashMap<u8, u8> {
    // Base64の文字をキーに6ビットの値を返すハッシュマップを生成する
    create_table().into_iter().map(|(i, ch)| (ch, i)).collect()
}

fn encode(input: Vec<u8>) -> Vec<u8> {
    let table = create_table();
    // 3バイトずつに分割して変換する
    let encoded = input
        .chunks(3)
        .map(|bytes| {
            // 複数のデータを1つの変数にまとめる
            let merged = bytes
                .iter()
                .enumerate()
                .fold(0u32, |merged, (i, x)| merged | (*x as u32) << (16 - 8 * i));
            // 6ビットずつに分割してBase64の文字に変換する
            let len = (8 * bytes.len()).div_ceil(6);
            (0..len)
                .map(|i| (merged >> (18 - 6 * i)) & 63)
                .filter_map(|i| table.get(&(i as u8)))
                .copied()
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<Vec<u8>>>()
        .concat();
    // 長さが4の倍数になるように調整
    let padding = (4 - encoded.len() % 4) % 4;
    [encoded, vec![b'='; padding]].concat()
}

fn decode(encoded: Vec<u8>) -> Vec<u8> {
    let reversed_table = create_reversed_table();
    // 4文字ずつ分割して変換する
    let decoded: Vec<u8> = encoded
        .chunks(4)
        .map(|bytes| {
            // 複数のデータを1つの変数にまとめる
            let merged: u32 = bytes
                .iter()
                .map(|b| reversed_table.get(b).unwrap_or(&0))
                .enumerate()
                .fold(0u32, |merged, (i, x)| merged | (*x as u32) << (18 - 6 * i));
            // 8ビットずつに分割して3バイトのバイナリに変換する
            (0..3)
                .map(|i| ((merged >> (16 - 8 * i)) & 255) as u8)
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<Vec<u8>>>()
        .concat();
    // パディングの数だけ末尾のデータを削除
    let len_padding = encoded.iter().rev().take_while(|b| **b == b'=').count();
    (decoded[..decoded.len() - len_padding]).to_vec()
}

fn read_stdin() -> Result<Vec<u8>> {
    let mut lock = io::stdin().lock();
    let mut input = Vec::new();
    loop {
        let mut buf = [0; 1024];
        let n = lock.read(&mut buf)?;
        if n == 0 {
            return Ok(input);
        } else {
            input.extend(&buf[..n]);
        }
    }
}

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    Encode,
    Decode,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = read_stdin()?;
    let bytes = match args.mode {
        Mode::Encode => encode(input),
        Mode::Decode => decode(input),
    };
    let mut lock = io::stdout().lock();
    lock.write_all(&bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{decode, encode};

    #[test]
    fn test_encode() {
        for (input, expected) in [
            ("", ""),
            ("A", "QQ=="),
            ("AB", "QUI="),
            ("ABC", "QUJD"),
            ("ABCD", "QUJDRA=="),
        ] {
            let encoded = encode(input.as_bytes().to_vec());
            assert_eq!(encoded, expected.as_bytes());
        }
    }

    #[test]
    fn test_decode() {
        for (input, expected) in [
            ("", ""),
            ("QQ==", "A"),
            ("QUI=", "AB"),
            ("QUJD", "ABC"),
            ("QUJDRA==", "ABCD"),
        ] {
            let decoded = decode(input.as_bytes().to_vec());
            assert_eq!(decoded, expected.as_bytes());
        }
    }
}
