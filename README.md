# DuniterNodeManager

DuniterNodeManager is a minimal graphical tool to manage a Duniter node. With this application, you can:

- Connect to your server via SSH (password authentication only).
- Edit configuration files.
- Start/stop the Duniter node and the oracle.
- View logs directly within the interface.

## Features

- **Graphical Interface**: Manage your Duniter node through an easy-to-use GUI.
- **SSH Support**: Connect to your server securely (only password authentication supported).
- **Cross-Platform**: Available for Windows, MacOS, and Linux.

## Server Configuration
To run certain `systemctl` commands (e.g., starting and stopping services) without a password prompt, you can configure `sudo` for passwordless access. Follow the steps below:

#### 1. **Edit the `sudoers` File for Passwordless Commands**

Next, modify the `sudoers` file to allow a user (e.g., `benjamin`) to run specific `systemctl` commands without a password prompt.

1. Open the `sudoers` file with `visudo`:

    ```bash
    sudo visudo
    ```

2. **Add the following lines at the end of the `sudoers` file**. This will allow the user `benjamin` to run `systemctl` commands for starting and stopping the `duniter-smith`, `duniter-mirror`, and `distance-oracle.timer` services, as well as manage the Duniter node, without entering a password:

    ```bash
    benjamin ALL=NOPASSWD: /usr/bin/systemctl start duniter-smith.service
    benjamin ALL=NOPASSWD: /usr/bin/systemctl stop duniter-smith.service

    benjamin ALL=NOPASSWD: /usr/bin/systemctl start duniter-mirror.service
    benjamin ALL=NOPASSWD: /usr/bin/systemctl stop duniter-mirror.service

    benjamin ALL=NOPASSWD: /usr/bin/systemctl start distance-oracle.timer
    benjamin ALL=NOPASSWD: /usr/bin/systemctl stop distance-oracle.timer

    benjamin ALL=NOPASSWD: /usr/bin/cat /etc/duniter/env_file
    benjamin ALL=NOPASSWD: /usr/bin/tee /etc/duniter/env_file
    ```

3. **Save the changes and exit `visudo`**. The changes should take effect immediately.

## Installation

### Precompiled Binaries
Precompiled binaries for Windows, MacOS, and Linux are available on the releases page. Download the appropriate binary for your platform and run the application.

### Build from Source
If you prefer to build the application from source, follow these steps:

1. **Install Prerequisites**
   - Ensure [Qt6](https://www.qt.io/) is installed on your system.
   - Ensure [Rust](https://www.rust-lang.org/tools/install) and Cargo are installed.

2. **Clone the Repository**
3. **Build and Run**
   ```bash
   cargo run
   ```
