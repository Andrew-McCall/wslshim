//! wslshim: runs the WSL tool named after this executable. Arguments that are
//! Windows drive paths (`C:\foo`, `c:/foo`, or `--opt=C:\foo`) are rewritten to
//! WSL form (`/mnt/c/foo`), unless the first argument is `--no-translate`
//! (which is consumed). Copy the built exe to `<tool>.exe`.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

fn is_drive_path(s: &str) -> bool {
    let b = s.as_bytes();

    b.len() >= 3
        && b[0].is_ascii_alphabetic()
        && b[1] == b':'
        && matches!(b[2], b'\\' | b'/')
}

fn drive_path_into(out: &mut String, s: &str) {
    let b = s.as_bytes();

    out.push_str("/mnt/");
    out.push(b[0].to_ascii_lowercase() as char);

    for c in s[2..].chars() {
        out.push(if c == '\\' { '/' } else { c });
    }
}

fn translate(arg: OsString) -> OsString {
    let Some(s) = arg.to_str() else {
        return arg;
    };

    // C:\foo -> /mnt/c/foo
    if is_drive_path(s) {
        let mut out = String::with_capacity(s.len() + 4);
        drive_path_into(&mut out, s);
        return out.into();
    }

    // --foo=C:\bar -> --foo=/mnt/c/bar
    if s.starts_with('-') {
        if let Some((opt, val)) = s.split_once('=')
            && is_drive_path(val)
        {
            let mut out = String::with_capacity(s.len() + 4);
            out.push_str(opt);
            out.push('=');
            drive_path_into(&mut out, val);
            return out.into();
        }
    }

    arg
}

fn main() {
    let mut args = env::args_os();

    let exe = PathBuf::from(args.next().unwrap_or_default());
    let name = exe
        .file_stem()
        .map(|s| s.to_os_string())
        .unwrap_or_default();

    // `--no-translate` as the first argument is consumed and disables path rewriting.
    let mut args = args.peekable();
    let translate_args = args.next_if(|a| a == "--no-translate").is_none();

    let wsl = Path::new(&env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()))
        .join("System32")
        .join("wsl.exe");

    match Command::new(wsl)
        .arg("-e")
        .arg(name)
        .args(args.map(|a| if translate_args { translate(a) } else { a }))
        .status()
    {
        Ok(status) => exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("wslshim: {e}");
            exit(127);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> String {
        translate(s.into()).into_string().unwrap()
    }

    #[test]
    fn converts_drive_paths() {
        assert_eq!(t(r"C:\Users\Andrew\x.txt"), "/mnt/c/Users/Andrew/x.txt");
        assert_eq!(t("d:/data/y"), "/mnt/d/data/y");
        assert_eq!(t(r"C:\"), "/mnt/c/");
        assert_eq!(t(r"--file=C:\x\y"), "--file=/mnt/c/x/y");
        assert_eq!(t("--file=d:/x/y"), "--file=/mnt/d/x/y");
    }

    #[test]
    fn leaves_others_alone() {
        for s in [
            "-la",
            "D:",
            "user@host:/p",
            r"foo\bar",
            r"x=C:\a",
            "relative/path",
            "",
        ] {
            assert_eq!(t(s), s);
        }
    }
}
