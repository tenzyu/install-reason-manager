# moree
> **more e**xplicit package management

## Proposal
Tired of forgetting why you installed certain packages?  `moree` helps you explicitly manage your package installations by recording the reasons behind them.  This brings clarity to your system's software and reduces future decision fatigue when reviewing or updating your packages.  Take control of your package management and understand the "why" behind every install.

## Motivation
- Gain a clear understanding of the purpose of each installed package.
- Simplify future package reviews and updates by having readily available installation reasons.
- Distinguish between explicitly installed packages and dependencies.
- Easily maintain and clean up your system by identifying and removing unnecessary software.

## Recommendation
Set an alias as `moe` if you don't have another `moe` command.

```zsh
# .zshrc
alias moe="moree"
```

## Usage
```
moe [command] [flags]
```

Running `moe` without a command will display the help.


### Commands

* **`add [packages...]`**
    * With `packages`: Interactively confirms the explicit installation status of the specified, already installed package(s).  If a package isn't installed, an error is displayed.
    * Without `packages`: Interactively reviews all installed packages on your system, prompting you to mark them as explicitly installed or dependencies. Includes a "Quit" option to stop the process and optionally save changes.

* **`apply`**  Reinstalls packages based on the installation reasons managed by `moree`.
    * `--with-install`: Installs packages marked as explicitly installed by `moree` that aren't currently on your system.
    * `--with-uninstall`: Uninstalls packages present on your system but not marked as explicitly installed by `moree`.
    * `--sync`:  A shorthand for `--with-install --with-uninstall`, synchronizing your system with the `moree` state.

* **`unmanaged`**  Lists packages installed on your system that aren't managed by `moree`.

* **`diff [--all]`**
    * Compares the `moree` managed state with your currently installed packages (explicitly and as dependencies), highlighting discrepancies with '+' and '-'.
    * `--all`:  Extends the diff to show packages managed by `moree` but not installed, and unmanaged packages installed explicitly. Note: Unmanaged dependencies are not shown.

* **`query`** Retrieves package information.
    * `--explicit` or `-e`: Lists all explicitly installed packages managed by `moree`.
    * `--deps` or `-d`: Lists all packages marked as dependencies by `moree`.
    * `--information` or `-i`: Prints detailed information (explicit status, memo) for all managed packages. `-e` and `-d` flags can be combined with this flag.

* **`edit <package>`** Interactively edits the explicit status and memo for the specified package.


### Flags

* **`--data <path>`** Specifies the path to the `moree` state file. Defaults to `$XDG_DATA_HOME/moree/state.json`.


## Use Cases

### Basic Package Management
```
moe add        # Review and categorize installed packages.
moe apply      # Reinstall packages according to their explicit status.
```

### Querying Package Information
```
moe query -e > .config/tenzyu/packages#arch  # Save a list of explicitly installed packages to a file.
```

### Reviewing Installation Reasons (Example)
```
moe query -i # Review detailed information about all managed packages, including installation reasons.
```

Thank you for discovering this somewhat hidden repository! If you find `moree` useful or interesting, please consider starring it. Your support motivates further development.
