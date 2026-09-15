# notify-action

Execute a command for every desktop notification which fulfills certain matchers.

## Usage

There are no prebuilt binaries for this program, thus you need to compile it yourself.
For this you need to have `git` installed and a rust toolchain with `cargo` set up.

```bash
# First, clone the git repository using
git clone https://github.com/WhySoBad/notify-action.git

cd notify-action

# Compile a release binary
cargo build --release
```

The built binary is then available under `target/release/notify-action`.
If you want to install it directly using cargo, the following command can be used.
However, make sure the cargo binary directory is added to your path:

```bash
# Install the binary into your cargo binary directory
cargo install --path .
```

Once installed, the binary accepts a `--config` command line flag to specify the path to a config file. The default value is `config.toml`.
More about the config file format can be read in the following section.

This program binds to the `/org/freedesktop/Notifications` name. 
If you already have a notification service running, binding to this name will fail.
You thus need to kill any notification service before running this program.

## Config

The alerts with their matchers are defined in a TOML configuration file.
The following is an example configuration:

```toml
[alert.telegram]
# All of the specified matchers need to be fulfilled.
quantity = "all"
# A list of matchers.
matchers = [
    # The notification needs to originate from Telegram.
    { type = "app-name", value = "^Telegram Desktop$" },
    # The notification body needs to contain `hello` or `Hello`.
    { type = "body", value = "(hello|Hello)" }
]
# The command to run when the right amount of matchers are fulfilled.
action = "echo telegram > /tmp/match"
```

When `quantity` is set to `all`, all of the specified matchers of the alert need to be fulfilled in order for the action to be executed.
If you only want a single matcher to be fulfilled, `quantity` can be set to `any`.

Additionally, there are different types of watchers:

- `app-name`: The value specifies a regex against which the name of the app which sent to notification is matched
- `body`: The value specifies a regex against which the body of the notification is matched
- `summary`: The value specifies a regex against which the summary of the notification is matched
- `urgency`: The value contains an urgency (`low`, `normal`, `critical`). The matcher is only fulfilled when the notification was sent with the specified urgency.

> [!TIP]
> Using `RUST_LOG=debug` every incoming notification is logged. This can be useful for setting up a configuration.
