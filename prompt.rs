use std::env;
use std::io::Read;
use std::str;
use std::string::String;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::process::Command;
use std::process;

fn git_status(staged: bool) -> Option<String> {
    let mut cmd = Command::new("git");
    let cmd = cmd.args(&["diff", "--shortstat", "--ignore-submodules"]);
    let cmd = if staged {
        cmd
    } else {
        cmd.arg("--staged")
    };

    match cmd.output() {
        Ok(process::Output {
            status,
            stdout: stat_staged,
            ..
        }) => if stat_staged.len() > 0 && status.success() {
            str::from_utf8(&stat_staged[..]).ok().map(|s| {
                s.trim_left().chars().take_while(|c| c.is_digit(10)).collect()
            })
        } else {
            None
        },
        _ => None
    }
}

fn relative_from<'a>(to_path: &'a Path, from_path: &'a Path) -> Option<&'a Path> {
    iter_after(to_path.components(), from_path.components()).map(|c| c.as_path())
}

fn iter_after<A, I, J>(mut iter: I, mut prefix: J) -> Option<I> where
    I: Iterator<Item=A> + Clone, J: Iterator<Item=A>, A: PartialEq
{
    loop {
        let mut iter_next = iter.clone();
        match (iter_next.next(), prefix.next()) {
            (Some(x), Some(y)) => {
                if x != y { return None }
            }
            (Some(_), None) => return Some(iter),
            (None, None) => return Some(iter),
            (None, Some(_)) => return None,
        }
        iter = iter_next;
    }
}

fn main() {
    let mut prompt = String::from("\x1b[1;34m\\w");

    let pwd_env = env::var("PWD").unwrap();
    let pwd = Path::new(&pwd_env);
    let mut repo = PathBuf::new().join(&pwd);
    let mut git_branch = None;
    let mut parent_repo = None;

    loop {
        let git = repo.join(".git");
        if let Ok(metadata) = fs::metadata(&git) {
            let mut head_file = if metadata.is_file() {
                let mut git_content = String::new();
                File::open(&git).unwrap().read_to_string(&mut git_content).unwrap();
                let mut prepo = repo.join(
                    git_content[8..].trim()
                );
                let gitdir = prepo.clone();
                loop {
                    // prepo.rev_str_components
                    if prepo.ends_with(".git") {
                        prepo.pop();
                        parent_repo = Some(prepo);
                        break;
                    }
                    if !prepo.pop() {
                        break;
                    }
                }

                File::open(&gitdir.join("HEAD")).unwrap()
            } else if metadata.is_dir() {
                File::open(&git.join("HEAD")).unwrap()
            } else {
                panic!();
            };
            git_branch = Some(String::new());
            head_file.read_to_string(git_branch.as_mut().unwrap()).unwrap();
        } else {
            if repo.pop() {
                continue;
            }
        }
        break;
    }

    match git_branch {
        Some(b) => {
            let mut branch = b[..].trim();
            if branch.starts_with("ref:") {
                if branch.starts_with("ref: refs/heads/") {
                    branch = &branch[16..];
                }
                else {
                    branch = &branch[4..].trim_left();
                }
            }
            else {
                branch = &branch[..7];
            }

            prompt = "\x1b[1m".to_string();
            match parent_repo {
                Some(prepo) => match prepo.file_name() {
                    Some(prepo_name) => {
                        prompt.push_str(prepo_name.to_str().unwrap());
                        prompt.push_str("\x1b[m/\x1b[1m");
                    }
                    None => ()
                },
                None => ()
            }
            match repo.file_name() {
                Some(repo_name) => {
                    let repo_name = repo_name.to_str().unwrap();
                    prompt.push_str(repo_name);
                }
                None => ()
            }

            if branch != "master" {
                prompt.push_str("\x1b[m:\x1b[31m");
                prompt.push_str(branch);
            }

            match git_status(false) {
                Some(staged) => {
                    prompt.push_str("\x1b[1;32m+");
                    prompt.push_str(&staged[..]);
                }
                None => match git_status(true) {
                    Some(changed) => {
                        prompt.push_str("\x1b[1;31m*");
                        prompt.push_str(&changed[..]);
                    }
                    None => ()
                }
            }

            if &*repo != pwd {
                prompt.push_str("\x1b[1;34m/");
                prompt.push_str(relative_from(&pwd, &*repo).unwrap().to_str().unwrap());
            }
        }
        None => ()
    }

    // Strip color codes.
    let mut term_title;
    {
        let mut iter = prompt.split("\x1b[");
        term_title = iter.next().unwrap_or("").to_string();
        for colored in iter {
            term_title.push_str(&colored[colored.find('m').unwrap() + 1 ..]);
        }
    };

    match env::var("RUBY_VERSION") {
        Ok(rv) => {
            prompt.push_str(" \x1b[0;37m");
            prompt.push_str(&rv[..]);
        }
        _ => ()
    }

    prompt.push_str(" \x1b[0m");

    match env::home_dir() {
        Some(home) => if &*home != pwd {
            prompt.push_str("$ ");
        },
        _ => ()
    }

    match env::var("TERM") {
        Ok(v) => if v.starts_with("xterm") {
            prompt.push_str(&format!("\x1b]0;{}\x07", term_title)[..]);
        },
        _ => ()
    }

    let mut final_prompt;
    if env::args().next().map(|s| s == "--delimit-non-printable") == Some(true) {
        let mut iter = prompt.split("\x1b[");

        final_prompt = iter.next().unwrap_or("").to_string();

        for colored in iter {
            final_prompt.push_str("\\[\x1b[");
            let text_start = colored.find('m').unwrap() + 1;
            final_prompt.push_str(&colored[..text_start]);
            final_prompt.push_str("\\]");
            final_prompt.push_str(&colored[text_start..]);
        }
    } else {
        final_prompt = prompt;
    }

    print!("{}", final_prompt);
}
