# Dev Tool Installer
`dti` is a command-line app that installs editor/code tools, such as language servers, linters and formatters.

All packages are installed in your home directory at `~/.devtoolinstaller/` by default (see section below on how to change), no need to clutter the system.

The binaries are put into `~/.devtoolinstaller/bin/`, so make sure you add that too your `PATH` environment variable.

## Installation
Using cargo from the [rust toolchain](https://rustup.rs/):

```sh
$ cargo install --locked --path .
```

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

`dti` will try to resolve the latest release (on GitHub) and install that version,
preferably from a binary that can run on your system.

If that's not the case, it may look for an alternative way to install a package.

For instance, if you try to install a package that `dti` doesn't know how
install on your platform but you have a native tool installed, it may prompt
you to use that instead.

By a _native tool_ i mean some of the following:
- `cargo`
- `go`
- `pip`
- `npm`

Although pre-built binaries are preferred, these exist as fallback for some packages;
in fact, some packages are only available through some of the listed tools.

## Configuration
An optional configuration file can be used to configure `dti`: `~/.devtoolinstaller/config.toml`.

```toml
# Optional. Additional configuration for packages.
[packages]
# Ensure that this list of packages is installed.
# These will be installed whenever running the `install` command.
ensure-installed = [
  "gopls",
  "lazygit",
]

# Optional. Allows you to specify a GitHub OAuth app
# that can be used in authentication. This is useful
# if you reach the API rate limit.
[auth]
client-id = "string"
client-secret = "string"
```

### Root
You can specify the root directory for `dti` using the `DTI_ROOT` environment variable.

If this isn't specified it defaults to `~/.devtoolinstaller`.
