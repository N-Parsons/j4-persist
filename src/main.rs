use std::time::SystemTime;

use failure;
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

    // Initialise the connection to i3
    let mut i3 = I3Connection::connect()?;

    // Get the tree
    let tree = i3.get_tree()?;

    // Extract the nodes and floating_nodes
    let mut nodes = tree.nodes;
    nodes.extend(tree.floating_nodes.iter().cloned());

    // Find the focused container
    let focused = get_focused(nodes)?;

    // Get the mark set on the focused container
    let mark = get_mark(&focused);

    match mark {
        None => match cmd {
            "lock" | "toggle" => {
                i3.run_command(&format!("mark --add j4-persist_{}", get_nonce()?))?;

                #[cfg(feature = "notifications")]
                Notification::new()
                    .summary("Window protected")
                    .icon("changes-prevent-symbolic.symbolic")
                    .show()?;
            }
            "kill" => safe_kill(focused, &mut i3)?,
            _ => return Ok(Err(failure::err_msg("Unknown command. Valid commands are: kill, lock, unlock, and toggle."))?),
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
            _ => return Ok(Err(failure::err_msg("Unknown command. Valid commands are: kill, lock, unlock, and toggle."))?),
        },
    };

    return Ok(())
}

// Helper functions
fn get_nonce() -> Result<u128, failure::Error> {
    return Ok(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis());
}

fn get_mark(node: &Node) -> Option<String> {
    match node.marks.iter().find(|m| m.starts_with("j4-persist_")) {
        Some(m) => return Some(m.to_owned()),
        None => return None,
    };
}

/// Recursively iterate through nodes
fn get_focused(nodes: Vec<Node>) -> Result<Node, failure::Error> {
    // Loop through the nodes of this container
    for node in nodes {
        if node.focused {
            // If we've found the focused container, return it
            return Ok(node);
        } else if node.focus.len() > 0 {
            // Only iterate the nodes if there is focus in this container
            // Iterate the non-floating nodes first - these are most likely
            let sub_node = get_focused(node.nodes);
            if sub_node.is_ok() {
                return sub_node;
            }

            // Then iterate floating_nodes
            let floating_node = get_focused(node.floating_nodes);
            if floating_node.is_ok() {
                return floating_node;
            }
        }
    }

    // If nothing is found, return an Err
    return Err(failure::err_msg("Focused node not found"));
}

/// Safely kill only unprotected windows in the container
fn safe_kill(node: Node, mut i3: &mut I3Connection) -> Result<(), failure::Error> {
    // Only perform the kill if no mark is set and there are no sub-nodes
    if node.nodes.len() == 0 && get_mark(&node).is_none() {
        let command = format!("[con_id={}] kill", node.id).to_owned();
        i3.run_command(&command)?;
    } else {
        for container in node.nodes {
            safe_kill(container, &mut i3)?;
        };
    }

    return Ok(());
}
