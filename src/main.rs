use std::fs;


use std::path::{Path};
use clap::Parser;
use globset::{Glob, GlobMatcher};
use colored::*;
use regex::Regex;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

#[derive(Parser, Debug)]
struct GrepArgs {
    content: String,
    file_name: String,
}

#[derive(Debug, Default)]
struct GrepResult {
    row: usize,
    col: Vec<(usize, usize)>,
    text: String,
}

#[derive(Debug, Default)]
struct MatchResult {
    file_name: String,
    gr: Option<Vec<GrepResult>>,
}

impl MatchResult {
    fn new(file_name: String) -> Self {
        Self {
            file_name: file_name,
            ..Default::default()
        }
    }
}

trait Grep {
    fn search(&mut self, str: &str) -> &Self;
    fn print_result(&self);
}

impl Grep for MatchResult {
    fn search(&mut self, str: &str) -> &Self {
        let regex = Regex::new(str).unwrap();
        let content = fs::read_to_string(&self.file_name).unwrap();
        for (index, x) in content.lines().enumerate() {
            for mat in regex.find_iter(x) {
                match self.gr {
                    Some(ref mut gr) => {
                        let mut gresult: GrepResult = Default::default();
                        gresult.row = index + 1;
                        gresult.text = x.to_string();
                        gresult.col.push((mat.start(), mat.end()));
                        gr.push(gresult);
                    }
                    None => {
                        self.gr = Some(vec![]);
                        let mut gresult: GrepResult = Default::default();
                        gresult.row = index + 1;
                        gresult.text = x.to_string();
                        gresult.col.push((mat.start(), mat.end()));
                        self.gr.as_mut().unwrap().push(gresult);
                    }
                }
            }
        }
        self
    }

    fn print_result(&self) {
        if let Some(ref v) = self.gr {
            println!("file_name:{}", self.file_name);
            for x in v {
                for c in &x.col {
                    println!("  {}:{} {}{}{}", format!("{}", x.row).red(), format!("{}", c.0).purple(),
                             x.text[..c.0].white(), x.text[c.0..c.1].green(), x.text[c.1..].white());
                }
            }
        }
    }
}

//单线程版本
fn read_dirs(p: &Path, glob: &GlobMatcher, match_str: &str,tx: UnboundedSender<MatchResult>) {
    if p.is_dir() {
        for entry in fs::read_dir(p).unwrap() {
            read_dirs(&entry.unwrap().path(), glob, match_str,tx.clone());
        }
    } else {
        if glob.is_match(p) {
            let file_name = p.to_str().unwrap();
            let file_name = file_name.to_string();
            let match_str = match_str.to_string();
            task::spawn(async move{
                let mut x = MatchResult::new(file_name);
                x.search(&match_str);
                tx.send(x).unwrap();
            });
        }
    }
}

#[tokio::main]
async fn main() {
    let args = GrepArgs::parse();
    let glob = Glob::new(format!("./{}", args.file_name).as_str()).expect("input file_name parse err").compile_matcher();
    let (tx, mut rx) = mpsc::unbounded_channel();
    read_dirs(Path::new("./"), &glob, &args.content,tx.clone());
    while let Some(v) = rx.recv().await{
        v.print_result();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use globset::Glob;

    #[test]
    fn testMf() {
        let glob = Glob::new("./*.txt").unwrap().compile_matcher();
        println!("{}", glob.is_match(Path::new("./aa.txt")));
    }
}
