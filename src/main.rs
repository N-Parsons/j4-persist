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
    let cmd = args.cmd.as_ref();

    let mut i3 = I3Connection::connect()?;
    let tree = i3.get_tree().unwrap();
    let focused = get_focused(tree.nodes).expect("Focused container not found");

    let mark = get_mark(&focused);

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
            "kill" => safe_kill(focused, &mut i3),
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

fn get_mark(node: &Node) -> Option<String> {
    match node.marks.iter().find(|m| m.starts_with("j4-persist_")) {
        Some(m) => return Some(m.to_owned()),
        None => return None,
    };
}

/// Recursively iterate through nodes
fn get_focused(nodes: Vec<Node>) -> Option<Node> {
    // Loop through the nodes of this container
    for node in nodes {
        if node.focused {
            // If we've found the focused container, return it
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
    // I don't think this should be reached normally
    return None;
}

/// Safely kill only unprotected windows in the container
fn safe_kill(node: Node, mut i3: &mut I3Connection) {
    // Only perform the kill if no mark is set and there are no sub-nodes
    if node.nodes.len() == 0 && get_mark(&node).is_none() {
        let command = format!("[con_id={}] kill", node.id).to_owned();
        i3.run_command(&command).expect("Failed to kill container");
    } else {
        for container in node.nodes {
            safe_kill(container, &mut i3);
        }
    }
}
