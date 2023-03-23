# archman

A CLI utility to help me manage my Arch Linux configuration. It reads a list of packages from a TOML
configuration file, then calls `pacman` to ensure the the list of installed packages matches the
configuration. It can also create symlinks and copy files listed in the configuration.
