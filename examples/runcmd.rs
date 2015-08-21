extern crate unshare;
extern crate argparse;
extern crate libc;

use std::io::{stderr, Write, Read};
use std::process::exit;
use std::path::PathBuf;

use libc::{uid_t, gid_t};
use argparse::{ArgumentParser, Store, StoreOption, Collect, StoreTrue};
use argparse::{ParseOption};


fn main() {
    let mut command = "".to_string();
    let mut args: Vec<String> = Vec::new();
    let mut workdir = None::<String>;
    let mut verbose = false;
    let mut escape_stdout = false;
    let mut uid = None::<uid_t>;
    let mut gid = None::<gid_t>;
    let mut chroot = None::<PathBuf>;
    let mut groups = Vec::<gid_t>::new();
    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Run command with changed process state");
        ap.refer(&mut command)
            .add_argument("command", Store, "Command to run")
            .required();
        ap.refer(&mut args)
            .add_argument("arg", Collect, "Arguments for the command")
            .required();
        ap.refer(&mut workdir)
            .add_option(&["--work-dir"], StoreOption, "
                Set working directory of the command");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "
                Enable verbose mode (prints command, pid, exit status)");
        ap.refer(&mut escape_stdout)
            .add_option(&["--escape-stdout"], StoreTrue, "
                Read data written by the utility to stdout and print it back
                as a quoted string with binary data escaped");
        ap.refer(&mut uid)
            .add_option(&["-U", "--uid"], StoreOption, "
                Set user id for the target process");
        ap.refer(&mut gid)
            .add_option(&["-G", "--gid"], StoreOption, "
                Set group id for the target process");
        ap.refer(&mut groups)
            .add_option(&["--add-group"], Collect, "
                Add supplementary group id");
        ap.refer(&mut chroot)
            .add_option(&["--chroot"], ParseOption, "
                Chroot to directory before running command");
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    let mut cmd = unshare::Command::new(&command);
    cmd.args(&args[..]);
    workdir.map(|dir| cmd.current_dir(dir));
    gid.map(|gid| cmd.gid(gid));
    uid.map(|uid| cmd.uid(uid));
    chroot.map(|dir| cmd.chroot_dir(dir));
    if groups.len() > 0 { cmd.groups(groups); }
    if escape_stdout {
        cmd.stdout(unshare::Stdio::piped());
    }
    if verbose {
        // TODO(tailhook) implement display/debug in Command itself
        writeln!(&mut stderr(), "Command {} {:?}", command, args).ok();
    }
    let mut child = match cmd.spawn() {
        Ok(child) => { child }
        Err(e) => {
            writeln!(&mut stderr(), "Error: {}", e).ok();
            exit(127);
        }
    };
    if verbose {
        writeln!(&mut stderr(), "Child pid {}", child.id()).ok();
    }
    if escape_stdout {
        let mut buf = Vec::new();
        child.stdout.take().unwrap().read_to_end(&mut buf).unwrap();
        writeln!(&mut stderr(), "{:?}",
            String::from_utf8_lossy(&buf[..])).unwrap();
    }
    let res = child.wait().unwrap();
    if verbose {
        writeln!(&mut stderr(), "[pid {}] {}", child.id(), res).ok();
    }

}
