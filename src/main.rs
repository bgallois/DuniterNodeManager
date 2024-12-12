use cstr::cstr;
use qmetaobject::prelude::*;
use qmetaobject::qtcore::core_application::QCoreApplication;
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;

mod resources_qml;

#[macro_export]
macro_rules! session {
    ($self:ident, $params:expr, $body:expr) => {
        match $self.open_session($params) {
            Err(e) => {
                $self.output = format!("Failed to open session: {:?}", e).into();
                $self.output_changed();
            }
            Ok(sess) => {
                $body(sess);
            }
        }
    };
}

#[derive(QObject, Default)]
struct Main {
    base: qt_base_class!(trait QObject),
    connect_ssh: qt_method!(fn(&mut self, params: String)),
    see_logs: qt_method!(fn(&mut self, params: String, node_string: String)),
    start_node: qt_method!(fn(&mut self, params: String, node_string: String)),
    stop_node: qt_method!(fn(&mut self, params: String, node_string: String)),
    start_oracle: qt_method!(fn(&mut self, params: String)),
    stop_oracle: qt_method!(fn(&mut self, params: String)),
    get_config: qt_method!(fn(&mut self, params: String)),
    write_config: qt_method!(fn(&mut self, params: String, config: String)),
    config: qt_property!(QString; NOTIFY config_changed),
    config_changed: qt_signal!(),
    output: qt_property!(QString; NOTIFY output_changed),
    output_changed: qt_signal!(),
}

impl Main {
    fn open_session(&self, params: String) -> Result<Session, ssh2::Error> {
        let args: Vec<&str> = params.split("@").collect();
        let tcp = TcpStream::connect(args[1]).map_err(|_| ssh2::Error::eof())?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_password(args[0], args[2])?;
        Ok(sess)
    }

    pub fn connect_ssh(&mut self, params: String) {
        session!(self, params, |sess: Session| {
            let mut channel = sess.channel_session().unwrap();
            channel.exec("command -v duniter2").unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            match s.as_ref() {
                "" => self.output = "No duniter on the system".into(),
                _ => self.output = format!("Duniter detected at {}", s).into(),
            }

            let mut channel = sess.channel_session().unwrap();
            channel
                .exec("systemctl list-unit-files | grep -i -E 'duniter|distance-oracle'")
                .unwrap();
            channel.read_to_string(&mut s).unwrap();
            match s.as_ref() {
                "" => self.output += "\nNo duniter services on the system".into(),
                _ => self.output += format!("\nDuniter services detected at {}", s).into(),
            }

            self.output_changed();
            channel.wait_close().unwrap();
        });
    }

    pub fn see_logs(&mut self, params: String, node_type: String) {
        session!(self, params, |sess: Session| {
            let mut channel = sess.channel_session().unwrap();
            channel
                .exec(&format!(
                    "journalctl -ru duniter-{}.service -u distance-oracle -n 100",
                    node_type
                ))
                .unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            self.output = format!("\n {}", s).into();
            self.output_changed();
            channel.wait_close().unwrap();
        });
    }

    fn command(&mut self, params: String, cmd: String) {
        session!(self, params, |sess: Session| {
            let mut channel = sess.channel_session().unwrap();
            channel.exec(&cmd).unwrap();
            channel.wait_eof().unwrap();
            channel.wait_close().unwrap();
        });
    }

    pub fn start_node(&mut self, params: String, node_type: String) {
        self.command(
            params.clone(),
            format!("sudo systemctl start duniter-{}.service", node_type),
        );
        self.see_logs(params, node_type);
    }

    pub fn stop_node(&mut self, params: String, node_type: String) {
        self.command(
            params.clone(),
            format!("sudo systemctl stop duniter-{}.service", node_type),
        );
        self.see_logs(params, node_type);
    }

    pub fn start_oracle(&mut self, params: String) {
        self.command(
            params.clone(),
            "sudo systemctl start distance-oracle.timer".into(),
        );
        session!(self, params, |sess: Session| {
            let mut channel = sess.channel_session().unwrap();
            channel
                .exec("journalctl -ru distance-oracle -n 100")
                .unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            self.output = format!("\n {}", s).into();
            self.output_changed();
            channel.wait_close().unwrap();
        });
    }

    pub fn stop_oracle(&mut self, params: String) {
        self.command(
            params.clone(),
            "sudo systemctl stop distance-oracle.timer".into(),
        );
        session!(self, params, |sess: Session| {
            let mut channel = sess.channel_session().unwrap();
            channel
                .exec("journalctl -ru distance-oracle -n 100")
                .unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            self.output = format!("\n {}", s).into();
            self.output_changed();
            channel.wait_close().unwrap();
        });
    }

    pub fn get_config(&mut self, params: String) {
        session!(self, params, |sess: Session| {
            let mut channel = sess.channel_session().unwrap();
            channel.exec("sudo cat /etc/duniter/env_file").unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            self.config = s.trim().into();
            self.config_changed();
            channel.wait_close().unwrap();
        });
    }

    pub fn write_config(&mut self, params: String, config: String) {
        self.command(
            params.clone(),
            format!(
                "echo \"{}\" | sudo tee /etc/duniter/env_file",
                config.trim()
            ),
        );
    }
}

fn main() {
    qml_register_type::<Main>(cstr!("Main"), 1, 0, cstr!("Main"));
    QCoreApplication::set_organization_name("Analyzable".into());
    QCoreApplication::set_organization_domain("gallois.cc".into());
    QCoreApplication::set_application_name("DuniterNodeManager".into());
    resources_qml::init_resources();
    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/qml/main.qml".into());
    engine.exec();
}
