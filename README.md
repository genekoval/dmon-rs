# dmon

A library for building daemon processes.

In addition to forking the process, `dmon` allows for configuring common aspects
of daemons, such as dropping privileges and redirecting standard streams.

## Example

```rust
use std::path::PathBuf;

#[derive(Default)]
struct Config {
    daemon: bool,
    user: Option<String>,
    pidfile: Option<PathBuf>,
}

impl Config {
    fn parse() -> Self {
        // Read command-line arguments or a config file...
        Default::default()
    }
}

fn main() {
    let config = Config::parse();

    let mut parent = if config.daemon {
        dmon::options()
            .user(config.user.as_deref())
            .pidfile(config.pidfile.as_deref())
            .working_directory(Some(
                format!("/var/lib/{}", env!("CARGO_PKG_NAME"))
            ))
            .stdout(Some("stdout.log"))
            .stderr(Some("stderr.log"))
            .daemonize();
    } else {
        Default::default()
    };

    // Perform additional setup such as starting an async runtime,
    // listening on a port, or creating some files.

    // Tell the original process and the user that the daemon
    // started successfully.
    parent.success().unwrap();
}
```

