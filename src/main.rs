#![cfg_attr(feature = "unstable", feature(test))]

#[cfg(feature = "unstable")]
extern crate test;

mod board;
mod error;

use std::fs::File;
use std::io::Read;

use board::Board;
use error::Error;

fn read(path: &str) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn boggle_main() -> Result<usize, Error> {
    let mut args = std::env::args();
    args.next().ok_or(Error::Usage)?;

    let raw_dict = {
        let dict_path = args.next().ok_or(Error::Usage)?;
        read(&dict_path)?
    };

    let raw_board = {
        let board_path = args.next().ok_or(Error::Usage)?;
        read(&board_path)?
    };

    let board = Board::parse(&raw_board)?;
    let dictionary = raw_dict.lines().filter(|l| l.len() > 3);
    Ok(board.solve(dictionary))
}

fn main() {
    match boggle_main() {
        Ok(s) => println!("Found {} matches!", s),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    }
}
