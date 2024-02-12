use std::collections::HashMap;
use crate::emulator::Insn;

fn parse_text(line: &str, section: &mut Section) -> Result<(), ParseError>{
    for (i, c) in line.iter().enumerate() {
        match c {
            ' ' | '\t' => continue,
            '.' => return parse_text_directive(line[c..]),
            '#' => return Ok(()),
            c if c.is_alphabetic() => parse_text_alpha(line[c..]),
            c => return Err(ParseError::InvalidChar(c))
        }
    }
    Ok(())
}

fn parse_text_directive(line: &str) -> Result<(), ParseError> {
    Ok(())
}

fn parse_text_alpha(line: &str) -> Result<(), ParseError> {
    for (i, c) in line.iter().enumerate() {
        match c {
            c
        }
    }
}

#[derive(thiserror::Error)]
enum ParseError {
    #[error("invalid char '{0}'")]
    InvalidChar(char)
}

struct Ir {
    insns: Vec<SymbolicInsn>,
    labels: HashMap<String, Option<usize>>
}

struct SymbolicInsn {
    insn: Insn,
    label: Option<String>
}

enum Section {
    Text,
    Data
}
