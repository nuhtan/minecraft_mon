# Minecraft Manager

What is this? This program runs a minecraft server in a child process, input and output to the server are handled in seperate threads. There is a seperate web server that gets launched along with the server that lets a user get information about the current state of the minecraft server and lets the user input commands to the server.

** Currently the server binds to the port 8000 on localhost **

## How to use this?
Linux
1. > git clone https://github.com/nuhtan/minecraft_mon.git && cd minecraft_mon

2. > mkdir server

This tool expects a 'server' directory to contain the server jar.

3. > *place a server .jar in the server folder

Any .jar should work, default, bukkit, spigot, paper, tuinity. It should be easy to modify the code to use a non jar server if you are using one. For all cases change the child args in [main.rs](src/main.rs). This process will change in the future to support both a configuration file and args.

4. > cargo build --release

5. > ./target/release/minecraft_mon

Windows
1. > git clone https://github.com/nuhtan/minecraft_mon.git && cd minecraft_mon

2. > mkdir server

This tool expects a 'server' directory to contain the server jar.

3. > *place a server .jar in the server folder

Any .jar should work, default, bukkit, spigot, paper, tuinity. It should be easy to modify the code to use a non jar server if you are using one. For all cases change the child args in [main.rs](src/main.rs). This process will change in the future to support both a configuration file and args.

4. > cargo build --release

5. > .\target\release\minecraft_mon.exe

## Features that are in progress
1. The current players does not update in anyway, I need to read console output to determine when a player joins or leaves. Issues arise as I think that server plugins can change the prefixes and suffixes for general chat so it might be possible that a player sending a message could be interpreted as a player leaving or joining as there is currently not any semblence of relation between different server outputs, ie. When a player joins a server there are typically three output messages but I need to verify that they are always together or potentially write a regex for the messages that references the current list of players.

## Planned changes for the future
1. Add systems for setting the child arguments without having to recompile the project. This will include a configuartion file and arguments, arguments will be taken with precedence over the configuration file.
2. Shutting down the server does not exit the application which is the intended use case.
3. There is no way to restart the server.
4. There is currently no ui in the web server.
5. Have releases for the project on github.
6. Refactor the data contained in Arc<Mutex<>>'s, the handle_connections() function is a mess, I think that using a struct would be the ideal solution but that introduces explicit runtimes of which I have no experience.
7. A verbose mode?
8. Have an option for what ip and port the web server are bound to, probably in the configuration file and arguments.
9. Output should be logged.