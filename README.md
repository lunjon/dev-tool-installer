# Dev Tool Installer
`dti` is a command-line app that installs editor/code tools, such as language servers, linters and formatters.

All packages are installed in your home directory at `~/.devtoolinstaller/`, no need to clutter the system.

The binaries are put into `~/.devtoolinstaller/bin/`, so make sure you add that too your `PATH` environment variable.

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

# Optional.
[packages]
ensure-installed = [
  "gopls",
  "lazygit",
]
```

## Todo
- Add support for configuring root, i.e. location on file system where everything is installed
  - Environment variable: `DTI_ROOT`
  - Configuration: `root = "string"`
- Additional configuration for packages in configuration:
  - Lock version: set a fix version for a package
- Add support for other platforms
  - It currently only support linux binaries in e.g. assets
  - https://doc.rust-lang.org/reference/conditional-compilation.html
- Support packages that do not have releases (e.g. bash-language-server)
