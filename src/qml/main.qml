import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15
import QtCore
import QtQuick.Dialogs
import Main

ApplicationWindow {
    visible: true
    width: 600
    height: 400
    title: "Duniter Node Manager"
    id: window
    font.family: "Roboto"

    property string doc: "
        Server Configuration
        ----------------------

        To run certain `systemctl` commands (e.g., starting and stopping services) without a password prompt,
        you can configure `sudo` for passwordless access. Follow the steps below:

        1. Edit the `sudoers` File for Passwordless Commands
        ------------------------------------------------------

        Next, modify the `sudoers` file to allow a user (e.g., `benjamin`) to run specific `systemctl` commands
        without a password prompt.

        - Open the `sudoers` file with `visudo`:

            ```bash
            sudo visudo
            ```

        - Add the following lines at the end of the `sudoers` file. This will allow the user `benjamin` to run
          `systemctl` commands for starting and stopping the `duniter-smith`, `duniter-mirror`, and 
          `distance-oracle.timer` services, as well as manage the Duniter node, without entering a password:

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

        - Save the changes and exit `visudo`. The changes should take effect immediately.
    "

    Settings {
        property alias x: window.x
        property alias y: window.y
        property alias width: window.width
        property alias height: window.height
    }

    Material.theme: Material.Light
    Material.primary: "#6200EE"
    Material.accent: "#FF5722"

    Main {
        id: main
    }

    GridLayout {
        anchors.centerIn: parent
        anchors.margins: 16
        columns: 5
        rowSpacing: 16
        columnSpacing: 16
        width: parent.width * 0.9
        height: parent.height * 0.9
        Layout.fillWidth: true

        ColumnLayout {
            spacing: 12
            Layout.fillWidth: true

            RowLayout {
                spacing: 12
                Layout.fillWidth: true

                Label {
                    text: "SSH Address:"
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: 80
                }

                TextField {
                    id: sshInput
                    placeholderText: "e.g., user@host:port"
                    Layout.fillWidth: true

                    Settings {
                        property alias sshInput: sshInput.text
                    }
                }
            }

            RowLayout {
                spacing: 12
                Layout.fillWidth: true

                Label {
                    text: "SSH Password:"
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: 80
                }

                TextField {
                    id: sshPass
                    placeholderText: "e.g., pass"
                    echoMode: TextInput.Password
                    Layout.fillWidth: true
                }
            }

            RowLayout {
                spacing: 12
                Layout.fillWidth: true

                Label {
                    text: "Node Type:"
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: 80
                }

                ComboBox {
                    id: nodeType
                    Layout.fillWidth: true
                    model: ["Mirror", "Smith"]
                }
            }
        }

        GridLayout {
            columns: 1
            rowSpacing: 12
            Layout.fillWidth: true
            Layout.preferredWidth: 100

            Button {
                text: "Check Installation"
                Layout.fillWidth: true
                onClicked: main.connect_ssh(sshInput.text + "@" + sshPass.text)
            }

            Button {
                text: "See Logs"
                Layout.fillWidth: true
                onClicked: main.see_logs(sshInput.text + "@" + sshPass.text, nodeType.currentText.toLowerCase())
            }
        }

        GridLayout {
            columns: 1
            rowSpacing: 12
            Layout.fillWidth: true
            Layout.preferredWidth: 100

            Button {
                text: "Start Node"
                Layout.fillWidth: true
                onClicked: main.start_node(sshInput.text + "@" + sshPass.text, nodeType.currentText.toLowerCase())
            }

            Button {
                text: "Stop Node"
                Layout.fillWidth: true
                onClicked: confirmationDialog.open()
                MessageDialog {
                    id: confirmationDialog
                    title: "Confirm Stopping"
                    text: "Are you sure you want to stop the node? If you are a Smith you need to go offline before stopping a node."
                    onAccepted: {
                        main.stop_node(sshInput.text + "@" + sshPass.text, nodeType.currentText.toLowerCase())
                    }
                    buttons: MessageDialog.Yes | MessageDialog.No
                }
            }
        }

        GridLayout {
            columns: 1
            rowSpacing: 12
            Layout.fillWidth: true
            Layout.preferredWidth: 100

            Button {
                text: "Start Oracle"
                Layout.fillWidth: true
                onClicked: main.start_oracle(sshInput.text + "@" + sshPass.text)
            }

            Button {
                text: "Stop Oracle"
                Layout.fillWidth: true
                onClicked: main.start_oracle(sshInput.text + "@" + sshPass.text)
            }
        }

        GridLayout {
            columns: 1
            rowSpacing: 12
            Layout.fillWidth: true
            Layout.preferredWidth: 100

            Button {
                text: "Edit"
                Layout.fillWidth: true
                onClicked: main.get_config(sshInput.text + "@" + sshPass.text)
            }

            Button {
                text: "Telemetry"
                Layout.fillWidth: true
                onClicked: {
                    Qt.openUrlExternally("https://telemetry.polkadot.io/#/0xc184c4ccde8e771483bba7a01533d007a3e19a66d3537c7fd59c5d9e3550b6c3")
                }
    }
        }

        ScrollView {
            Layout.columnSpan: 5
            Layout.fillWidth: true
            Layout.fillHeight: true
            TextArea {
                id: commandOutput
                readOnly: true
                font.pointSize: 12
                placeholderText: "Command output will appear here..."
                text: main.output !== "" ? main.output : doc
                wrapMode: TextArea.Wrap
            }
        }
    }
    Dialog {
        id: configEditor
        title: "Edit Configuration. A reboot of the node/oracle will be necessary to apply changes!"
        standardButtons: Dialog.Ok | Dialog.Cancel
        visible: main.config !== ""

        width: parent.width
        height: parent.height

        ScrollView {
            anchors.fill: parent

            TextArea {
                id: textEditor
                wrapMode: TextEdit.Wrap
                text: main.config
                font.pointSize: 12
                focus: true
            }
        }
        onAccepted: {
            main.write_config(sshInput.text + "@" + sshPass.text, textEditor.text)
            configEditor.close()
        }

        onRejected: {
            configEditor.close()
        }
    }
}
