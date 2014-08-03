use std::str;
use std::string::String;
use std::vec::Vec;
use std::os;
use std::path::Path;
use std::io::File;
use std::io::process::Command;
use std::io::process::ProcessOutput;
use std::io::process::ExitStatus;

fn git_status(staged: bool) -> Option<String> {
    let mut git = Command::new("git");
    let cmd = if staged {
        git.args(["diff", "--shortstat", "--ignore-submodules"])
    } else {
        git.args(["diff", "--shortstat", "--ignore-submodules", "--staged"])
    };

    match cmd.output() {
        Ok(ProcessOutput {
            status: ExitStatus(0),
            output: stat_staged,
            error: _
        }) => if stat_staged.len() > 0 {
            str::from_utf8(stat_staged.as_slice()).map(|s| {
                let s: Vec<char> = s.trim_left().chars().take_while(|c| c.is_digit()).collect();
                String::from_chars(s.as_slice())
            })
        } else {
            None
        },
        _ => None
    }

}

fn main() {
    let mut prompt = String::from_str("\\[\x1b[1;34m\\]\\w");
    let mut term_title = String::from_str("\\w");

    let pwd = Path::new(os::getenv("PWD").unwrap());
    let mut repo = pwd.clone();
    let mut git_branch = None;
    let mut parent_repo = None;

    loop {
        let git = repo.join(".git");
        if git.exists() {
            if git.is_file() {
                let mut prepo = repo.join(
                    File::open(&git).unwrap().read_to_string().unwrap().as_slice().slice_from(8).trim().to_string()
                );
                let gitdir = prepo.clone();
                loop {
                    // prepo.rev_str_components
                    if prepo.ends_with_path(&Path::new(".git")) {
                        prepo.pop();
                        parent_repo = Some(prepo);
                        break;
                    }
                    if !prepo.pop() {
                        break;
                    }
                }

                git_branch = File::open(&gitdir.join("HEAD")).unwrap().read_to_string().ok();
            }
            else if git.is_dir() {
                git_branch = File::open(&git.join("HEAD")).unwrap().read_to_string().ok();
            }
            break;
        }
        if !repo.pop() {
            break;
        }
    }

    match git_branch {
        Some(b) => {
            let mut branch = b.as_slice().trim();
            if branch.starts_with("ref:") {
                if branch.starts_with("ref: refs/heads/") {
                    branch = branch.slice_from(16);
                }
                else {
                    branch = branch.slice_from(4).trim_left();
                }
            }
            else {
                branch = branch.slice_to(7);
            }

            prompt = "\\[\x1b[1m\\]".to_string();
            term_title = String::new();
            match parent_repo {
                Some(prepo) => match prepo.filename_str() {
                    Some(prepo_name) => {
                        prompt.push_str(prepo_name);
                        prompt.push_str("\\[\x1b[m\\]/\\[\x1b[1m\\]");
                    }
                    None => ()
                },
                None => ()
            }
            match repo.filename_str() {
                Some(repo_name) => {
                    prompt.push_str(repo_name);
                    term_title.push_str(repo_name);
                }
                None => ()
            }

            if branch != "master" {
                prompt.push_str("\\[\x1b[m\\]:\\[\x1b[31m\\]");
                prompt.push_str(branch);
                term_title.push_char(':');
                term_title.push_str(branch);
            }

            match git_status(false) {
                Some(staged) => prompt.push_str(format!("\\[\x1b[1;32m\\]+{}", staged).as_slice()),
                None => match git_status(true) {
                    Some(changed) => {
                        prompt.push_str("\\[\x1b[1;31m\\]*");
                        prompt.push_str(changed.as_slice());
                    }
                    None => ()
                }
            }

            if repo != pwd {
                prompt.push_str("\\[\x1b[1;34m\\]/");
                prompt.push_str(pwd.path_relative_from(&repo).unwrap().as_str().unwrap());
            }
        }
        None => ()
    }

    match os::getenv("RUBY_VERSION") {
        Some(rv) => {
            prompt.push_str(" \\[\x1b[0;37m\\]");
            prompt.push_str(rv.as_slice());
        }
        _ => ()
    }

    prompt.push_str(" \\[\x1b[0m\\]");

    match os::homedir() {
        Some(home) => if home != pwd {
            prompt.push_str("$ ");
        },
        _ => ()
    }

    match os::getenv("TERM") {
        Some(v) => if v.as_slice().starts_with("xterm") {
            print!("\\[\x1b]0;{}\x07\\]", term_title);
        },
        None => ()
    }

    print!("{}", prompt);
}
