# DuniterNodeManager

**Intro:** A minimal graphical node manager. Connect to the server (only SSH password authentication is supported at the moment), edit configurations, start/stop the node and oracle, and view logs.

### Configuring Passwordless `sudo` for `systemctl` Commands

To run certain `systemctl` commands (e.g., starting and stopping services) without a password prompt, you can configure `sudo` for passwordless access. Follow the steps below:


#### 1. **Edit the `sudoers` File for Passwordless Commands**

Next, modify the `sudoers` file to allow a user (e.g., `benjamin`) to run specific `systemctl` commands without a password prompt.

1. Open the `sudoers` file with `visudo`:

    ```bash
    sudo visudo
    ```

2. **Add the following lines at the end of the `sudoers` file**. This will allow the user `benjamin` to run `systemctl` commands for starting and stopping the `duniter-smith`, `duniter-mirror`, and `distance-oracle.timer` services, as well as manage the Duniter node, without entering a password. Make sure to replace `/usr/bin/systemctl` with the exact path returned by `which systemctl` (if different):

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

3. **Save the changes and exit `visudo`**. The changes should take effect immediately. By adding these lines at the end of the file, you avoid been overwritten any previous configurations.

