use std::time::SystemTime;

use quicli::prelude::*;
use structopt::StructOpt;

use i3ipc::reply::Node;
use i3ipc::I3Connection;

/// Replacement for i3wm's built-in 'kill' command, with the ability to protect windows
#[derive(StructOpt)]
struct Cli {
    // Positional argument
    /// Operation to run: kill, lock, unlock, toggle
    cmd: String,
}

struct S(&str, Option<String>);

fn main() -> CliResult {
    let args = Cli::from_args();

    let mut i3 = I3Connection::connect()?;

    let mark = get_mark();
    let cmd = args.cmd.as_ref();

    match mark {
        None => match cmd {
            "lock" | "toggle" => i3
                .run_command(&format!("mark --add j4-persist_{}", get_nonce()))
                .expect("Failed to set mark"),
            "kill" => i3.run_command("kill").expect("Failed to kill window"),
            _ => return,
        },
        Some(m) => match cmd {
            "unlock" | "toggle" => i3
                .run_command(&format!("unmark {}", m))
                .expect("Failed to unset mark"),
            _ => return,
        },
    };

    Ok(())
}


// Helper methods
fn get_nonce() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(time) => return time.as_millis(),
        Err(_) => panic!("Failed to get current time"),
    };
}


fn get_mark(i3: &mut I3Connection) -> Option<&'static str> {
    //    let marks = i3.get_marks();
    //    let matching = marks.iter().filter(|m| m.starts_with("j4-persist_")).collect::<Vec<_>>();

    println!(
        "{:?}",
        i3.get_tree().unwrap().nodes[1].nodes[1].nodes[1].nodes[1].marks
    ); //.unwrap().nodes[1].nodes[1]); //.iter().filter(|n| n.focused));

    return Some("j4-persist");
}
