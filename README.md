# Dev Tool Installer
`dti` is a command-line app that installs editor/code tools, such as language servers, linters and formatters.

All packages are installed in your home directory at `~/.devtoolinstaller/`, no need to clutter the system.

The binaries are put into `~/.devtoolinstaller/bin/`, so make sure you add that too your `PATH` environment variable.

## Usage
The CLI can be explored by just running `dti`:
```sh
$ dti
...
```

### Installing Packages
The most import sub-command is probably `install`:
```sh
$ dti install <package>
```

You can list available packages with
```sh
$ dti list --all # or shorter: dti ls -a
```

`dti` will try to resolve the latest release (on GitHub) and install that version.

## Configuration
An optional configuration file can be used to configure some stuff: `~/.devtoolinstaller/config.toml`.

The following block is an example of a configuration file.

```toml
# Optional. Allows you to specify a GitHub OAuth app
# that can be used in authentication. This is useful
# if you reach the API rate limit.
[auth]
client-id = "string"
client-secret = "string"

# Optional. Additional configuration for packages.
[packages]
# Ensure that this list of packages is installed.
# These will be installed whenever running the `install` command.
ensure-installed = [
  "gopls",
  "lazygit",
]
```

### Root
You can specify the root directory for `dti` using the `DTI_ROOT` environment variable.

If this isn't specified it defaults to `~/.devtoolinstaller`.

## Todo
- Additional configuration for packages in configuration:
  - Lock version: set a fix version for a package
- Add support for other platforms
  - It currently only support linux binaries in e.g. assets
  - https://doc.rust-lang.org/reference/conditional-compilation.html
- Support packages that do not have releases (e.g. bash-language-server)
