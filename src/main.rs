use std::time::SystemTime;

use quicli::prelude::*;
use structopt::StructOpt;

use i3ipc::reply::Node;
use i3ipc::I3Connection;

#[cfg(feature = "notifications")]
use notify_rust::Notification;

/// Replacement for i3wm's built-in 'kill' command, with the ability to protect windows
#[derive(StructOpt)]
struct Cli {
    // Positional argument
    /// Operation to run: kill, lock, unlock, toggle
    cmd: String,
}

fn main() -> CliResult {
    let args = Cli::from_args();

    let mut i3 = I3Connection::connect()?;

    let mark = get_mark(&mut i3);
    let cmd = args.cmd.as_ref();

    match mark {
        None => match cmd {
            "lock" | "toggle" => {
                i3.run_command(&format!("mark --add j4-persist_{}", get_nonce()))
                    .expect("Failed to set mark");

                #[cfg(feature = "notifications")]
                Notification::new()
                    .summary("Window protected")
                    .icon("changes-prevent-symbolic.symbolic")
                    .show()?;
            }
            "kill" => {
                i3.run_command("kill").expect("Failed to kill window");
            }
            _ => return Ok(()),
        },
        Some(m) => match cmd {
            "unlock" | "toggle" => {
                i3.run_command(&format!("unmark {}", m))
                    .expect("Failed to unset mark");

                #[cfg(feature = "notifications")]
                Notification::new()
                    .summary("Window unprotected")
                    .icon("changes-allow-symbolic.symbolic")
                    .show()?;
            }
            "kill" => {
                #[cfg(feature = "notifications")]
                Notification::new()
                    .summary("Window is protected")
                    .icon("changes-prevent-symbolic.symbolic")
                    .show()?;
            }
            _ => return Ok(()),
        },
    };

    Ok(())
}

// Helper functions
fn get_nonce() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(time) => return time.as_millis(),
        Err(_) => panic!("Failed to get current time"),
    };
}

fn get_mark(i3: &mut I3Connection) -> Option<String> {
    let tree = i3.get_tree().unwrap();
    let focused = get_focused(tree.nodes);

    match focused {
        Some(node) => match node.marks.iter().find(|m| m.starts_with("j4-persist_")) {
            Some(m) => return Some(m.to_owned()),
            None => return None,
        },
        None => panic!("Failed to get focused window"),
    }
}

/// Recursively iterate through nodes
fn get_focused(nodes: Vec<Node>) -> Option<Node> {
    // Loop through the nodes of this container
    for node in nodes {
        if node.focused {
            // If we've found the focused window, return it
            return Some(node);
        } else if node.focus.len() > 0 {
            // Only iterate the nodes if there is focus in this container
            let sub_node = get_focused(node.nodes);
            match sub_node {
                Some(_) => return sub_node,
                None => continue,
            }
        }
    }

    // If nothing is found, return None
    // I don't think this should be reached this shouldn't be reached
    return None;
}
