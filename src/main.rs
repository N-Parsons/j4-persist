use std::time::SystemTime;

use clap::{App, AppSettings, SubCommand};
use failure;

use i3ipc::reply::Node;
use i3ipc::I3Connection;

#[cfg(feature = "notifications")]
use notify_rust::Notification;

/// Replacement for i3wm's built-in 'kill' command, with the ability to protect windows
fn main() -> Result<(), failure::Error> {
    // Set up the command line interface
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("kill").about("Kill the active container (if unlocked)"))
        .subcommand(SubCommand::with_name("lock").about("Lock the active container"))
        .subcommand(SubCommand::with_name("unlock").about("Unlock the active container"))
        .subcommand(
            SubCommand::with_name("toggle").about("Toggle the lock on the active container"),
        )
        .get_matches();

    // Get the subcommand name
    let cmd = matches.subcommand_name().unwrap();

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
            "unlock" => {
                #[cfg(feature = "notifications")]
                Notification::new()
                    .summary("Window unprotected")
                    .icon("changes-allow-symbolic.symbolic")
                    .show()?;
            }
            _ => unreachable!(),
        },
        Some(m) => match cmd {
            "unlock" | "toggle" => {
                i3.run_command(&format!("unmark {}", m))?;

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
            "lock" => {
                #[cfg(feature = "notifications")]
                Notification::new()
                    .summary("Window protected")
                    .icon("changes-prevent-symbolic.symbolic")
                    .show()?;
            }
            _ => unreachable!(),
        },
    };

    Ok(())
}

// Helper functions
fn get_nonce() -> Result<u128, failure::Error> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis())
}

fn get_mark(node: &Node) -> Option<String> {
    node.marks
        .iter()
        .find(|m| m.starts_with("j4-persist_"))
        .map(|m| m.to_owned())
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
            if let Ok(sub_node) = get_focused(node.nodes) {
                return Ok(sub_node);
            }

            // Then iterate floating_nodes
            if let Ok(floating) = get_focused(node.floating_nodes) {
                return Ok(floating);
            }
        }
    }

    // If nothing is found, return an Err
    Err(failure::err_msg("Focused node not found"))
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
        }
    }

    Ok(())
}
