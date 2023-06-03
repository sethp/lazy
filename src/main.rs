use nix::{fcntl::SpliceFFlags, unistd::Whence::SeekCur};
use std::{env, ffi::CString, os::unix::prelude::OsStringExt};

/* TODO signals? */
fn main() -> ! {
    let name: &str = &env::args()
        .next()
        .unwrap_or("lazy(missing arg0)".to_owned());

    if env::args_os().skip(1).count() < 1 {
        eprintln!("saw no arguments, at least one is required (use `true(1)`?)");
        eprintln!("usage: {name} COMMAND [...]");
        std::process::exit(2);
    }

    // TODO stdin is always open, which means at least one test is invalid
    // println!(
    //     "hello: {:?} I am {name}",
    //     nix::fcntl::fcntl(libc::STDIN_FILENO, F_GETFD)
    // );

    let pipe = nix::unistd::pipe2(nix::fcntl::OFlag::O_CLOEXEC);
    let (_, w) = if let Err(errno) = pipe {
        eprintln!("{name}: could not create pipe with `pipe2(2)`: {errno:?}");
        std::process::exit(1);
    } else {
        pipe.unwrap()
    };

    let tee = nix::fcntl::tee(
        libc::STDIN_FILENO,
        w,
        1, /* TODO param? would need to loop to ensure we got it all */
        SpliceFFlags::empty(),
    );

    'wait: {
        match tee {
            Err(nix::errno::Errno::EINVAL) => {
                match nix::unistd::lseek(libc::STDIN_FILENO, 0, SeekCur) {
                    Ok(_) => {
                        // now we know we can seek, we can read
                        let mut buf = [0u8; 1];

                        match nix::unistd::read(libc::STDIN_FILENO, &mut buf) {
                            Ok(0) => {
                                std::process::exit(3);
                            }
                            Ok(n) => {
                                // we read `n` bytes, hooray!
                                // let's clean up after ourselves...
                                match nix::unistd::lseek(libc::STDIN_FILENO, -(n as i64), SeekCur) {
                                    Ok(_) => {
                                        // we reset the file descriptor, hooray!
                                        // time to exec
                                        break 'wait;
                                    }
                                    Err(errno) => {
                                        eprintln!("{name}: could not lseek: {errno}");
                                    }
                                }
                            }
                            Err(errno) => {
                                eprintln!("{name}: could not lseek: {errno}");
                            }
                        }
                    }
                    Err(errno) => {
                        eprintln!("{name}: could not lseek: {errno}");
                    }
                }

                eprintln!("{name}: stdin not a pipe and not seekable");
                std::process::exit(1)
            }
            Ok(0) => {
                // No data to read, we exit
                std::process::exit(3);
            }
            Ok(_) => { /* continue below */ }

            Err(errno @ nix::errno::Errno::ENOMEM) => {
                eprintln!("{name}: `tee(2)` failed: {errno:?}");
                std::process::exit(1)
            }

            _ => unreachable!(),
        };
    }

    // TODO can we get the args as a slice?
    let argv: Vec<_> = env::args_os()
        .skip(1)
        .map(|arg| CString::new(arg.into_vec()))
        .collect::<Result<Vec<_>, _>>()
        .expect("interior nulls? in MY c string?");
    // println!("{argv:?}");
    match nix::unistd::execvp(&argv[0], argv.as_slice()) {
        Ok(_) => unreachable!(),
        Err(errno) => {
            eprintln!("{name}: `execv(3)` failed: {errno:?}");
            std::process::exit(1)
        }
    }
}

/// doesn't work, there's no way to tell "ready to read (nothing)" apart from "nothing to read"
/// too bad, because it'd be much more portable than `tee`
#[cfg(off)]
fn poll_magic() {
    use nix::{
        fcntl::FcntlArg::F_GETFD,
        ioctl_read_bad,
        poll::{PollFd, PollFlags},
    };
    const FIONREAD: u16 = 0x541B; //https://elixir.bootlin.com/linux/latest/source/include/uapi/asm-generic/ioctls.h#L46
    ioctl_read_bad!(get_bytes_ioctl, FIONREAD, i32);

    // kind of a bummer: &mut [ xxx ] copies instead of borrows
    let mut fds = [PollFd::new(libc::STDIN_FILENO, PollFlags::POLLIN)];

    // why does this only fire when I hit a newline from a tty ?
    // ah, canonical mode: https://viewsourcecode.org/snaptoken/kilo/02.enteringRawMode.html
    let r = nix::poll::poll(&mut fds, -1);

    let gfd = nix::fcntl::fcntl(libc::STDIN_FILENO, F_GETFD);
    let mut bytes = 0;
    let lol = unsafe { get_bytes_ioctl(fds[0].as_raw_fd(), &mut bytes) };

    println!("hooray {r:?} {:?} {gfd:?} {lol:?} {bytes}", fds);
}
