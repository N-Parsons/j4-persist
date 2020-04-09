# j4-persist
j4-persist - extends i3 window management with persistent containers

## Description
`j4-persist` is a simple program that replaces the built-in `kill` command to allow you to protect containers from being closed.

## Dependencies
By default, the program displays helpful notifications via DBus to let you know if a window has been protected, unprotected, or if it can't be killed because it is protected. 

If DBus isn't installed, the program should still perform the action, but will exit with an error when the notification would otherwise be shown.

If you want to disable notifications, you can build the crate with the `--no-default-features` flag.

## Installation
If you have a Rust toolchain set-up, you can build and install the program directly from this repo using [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html), which will install it to `~/.cargo/bin` by default.

```sh
cargo install --git https://github.com/N-Parsons/j4-persist j4-persist
```

If you don't have Rust set up yet, check out [rustup](https://rustup.rs/).

### Binaries
Binaries for `x86_64` Linux are available on the releases tab.

These binaries are signed with my GPG key, which is available at [keybase.io/nparsons](https://keybase.io/nparsons) and can be imported trivially with
`curl https://keybase.io/nparsons/pgp_keys.asc | gpg --import`. You can then verify the binary with `gpg --verify j4-persist.gpg`.

## Usage
```
j4-persist <command>`

with <command>:
  - kill: kill the focused container (unless protected)
  - lock: protect the focused container
  - unlock: unprotect the focused container
  - toggle: toggle the protection of the focused container
```

In your i3 config you now set your bind to call `exec j4-persist kill`, and add binds for locking, unlocking, and toggling the protection on containers. The below snippet is what I use, but you're obviously free to rework it to suit your needs.

```
# kill focused window
bindsym $mod+Shift+q kill  # I don't normally use this shortcut, so this lets me kill protected windows
bindsym $mod+Delete exec j4-persist kill
bindsym $mod+Shift+Delete exec j4-persist toggle


# set states for j4-persist
bindsym $mod+Control+Delete mode "$j4_persist_states"
set $j4_persist_states (L) Lock, (U) Unlock, (T) Toggle

mode "$j4_persist_states" {
  bindsym l exec j4-persist lock, mode "default"
  bindsym u exec j4-persist unlock, mode "default"
  bindsym t exec j4-persist toggle, mode "default"

  bindsym Return mode "default"
  bindsym Escape mode "default"
}
```

If you've got a program that you want to automatically mark as persistent, the easiest way to do this is to just run the command when the window opens. For example, I lock my X2Go Agent windows:

```
for_window [class="X2GoAgent"] exec j4-persist lock
```

## Acknowledgements
This tool was inspired by [Igrom/i3-persist](https://github.com/Igrom/i3-persist) and some of the comments and suggestions made by users within issues and pull requests.
