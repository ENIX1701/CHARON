# Functionality

CHARON is divided into four main operational contexts, accessible via tabs.

## Dashboard

// PIC

Main view. Provides a real-time table of all GHOSTs that have checked in with the server.

- **Monitoring**: ID, hostname, OS and last seen timer
- **Selection**:  
- **Entrance to other modules**: select GHOSTs to use in Terminal and Config views :3

## Terminal

// PIC

Allows direct interaction with the selected GHOST.

- **Command history**: scrolling log of sent tasks and their results (*almost* like a real terminal!)
- **Input mode**: pressing `i` enters input mode. You can then type your command and see the results (...when the GHOST sends them back...)
- **Commands**:
    - **Implicit**: not typing an explicit command defaults to `EXEC`. For example, if you type `whoami`, CHARON will send `EXEC whoami` to SHADOW
    - **Explicit**:
        - `EXEC <command>` - execute a shell command 
        - `IMPACT` - triggers the configured [`Impact`](https://github.com/ENIX1701/GHOST/blob/main/docs/FUNCTIONALITY.md#impact) module (be **VERY CAREFUL** with this one)
        - `STOP_HAUNT` - kills the GHOST. You can also do that on the dashboard panel, by pressing `x`, selecting the *kill GHOST* option and pressing `enter`

## Config

// PIC

This tab allows, well, you wouldn't guess, configuration! ...of the GHOSTs.

It allows you to change:
- **Sleep interval**: useful for keeping a low profile if your red teaming activity might've been detected by the blue team
- **Jitter**: as above, helps obfuscate the traffic and blend in

There is a big button that lets you submit those to SHADOW (which, in turn, passes these to the selected GHOST). Press it if you want to submit. Good job :3 

## Builder

// PIC

You can build GHOSTs with it. More extensive documentation can be found [here](https://github.com/ENIX1701/GHOST/docs/FUNCTIONALITY.md). Also, [usage guide](MANUAL.md#builder).

You can control which modules, and which tactics in these modules are enabled. Full control. Always.
