#![recursion_limit = "4096"]
#![windows_subsystem = "windows"]
#![feature(addr_parse_ascii)]

use cpp::cpp;
use cstr::cstr;
use qmetaobject::prelude::*;
use qmetaobject::qtcore::core_application::QCoreApplication;
use qmetaobject::QStringList;
use ssh2::PublicKey;
use ssh2::Session;
use std::ffi::CString;
use std::io::Read;
use std::net::TcpStream;

cpp! {{
    #include <QtGui/QGuiApplication>
    #include <QIcon>
}}

mod resources_qml;

#[macro_export]
macro_rules! session {
    ($self:ident, $params:expr, $pass:expr, $body:expr) => {
        match $self.open_session($params, $pass) {
            Err(e) => {
                $self.output = format!("Failed to open session: {:?}", e.message()).into();
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
    check_installation: qt_method!(fn(&mut self, params: String, pass: String)),
    see_logs: qt_method!(fn(&mut self, params: String, pass: String, node_string: String)),
    start_node: qt_method!(fn(&mut self, params: String, pass: String, node_string: String)),
    stop_node: qt_method!(fn(&mut self, params: String, pass: String, node_string: String)),
    start_oracle: qt_method!(fn(&mut self, params: String, pass: String)),
    stop_oracle: qt_method!(fn(&mut self, params: String, pass: String)),
    get_config: qt_method!(fn(&mut self, params: String, pass: String)),
    write_config: qt_method!(fn(&mut self, params: String, pass: String, config: String)),
    get_keys: qt_method!(fn(&self) -> QStringList),
    config: qt_property!(QString; NOTIFY config_changed),
    config_changed: qt_signal!(),
    output: qt_property!(QString; NOTIFY output_changed),
    output_changed: qt_signal!(),
}

impl Main {
    fn open_session(&self, params: String, pass: String) -> Result<Session, ssh2::Error> {
        let args: Vec<&str> = params.split("@").collect();
        let _: Result<(), String> = (args.len() == 2).then_some(Ok(())).ok_or_else(|| {
            ssh2::Error::new(
                ssh2::ErrorCode::Session(0),
                "Invalid input format: Expected 'username@ip:port'.",
            )
        })?;
        let address = std::net::SocketAddrV4::parse_ascii(args[1].as_bytes()).map_err(|e| {
            ssh2::Error::new(
                ssh2::ErrorCode::Session(0),
                Box::leak(format!("Connexion Error: {}", e).into_boxed_str()),
            )
        })?;

        let tcp = TcpStream::connect_timeout(
            &std::net::SocketAddr::V4(address),
            std::time::Duration::from_secs(2),
        )
        .map_err(|e| {
            ssh2::Error::new(
                ssh2::ErrorCode::Session(0),
                Box::leak(format!("{}", e).into_boxed_str()),
            )
        })?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        if pass.contains("🔑") {
            let mut agent = sess.agent()?;
            agent.connect()?;
            agent.list_identities()?;
            let identity: PublicKey = agent
                .identities()?
                .into_iter()
                .find(|i| {
                    pass.contains(
                        &(i.comment().to_owned()
                            + &i.blob()
                                .iter()
                                .map(|&byte| byte as u32)
                                .sum::<u32>()
                                .to_string()),
                    )
                })
                .ok_or(ssh2::Error::new(
                    ssh2::ErrorCode::Session(0),
                    "Identity not found",
                ))?;
            agent.userauth(args[0], &identity)?;
        } else {
            sess.userauth_password(args[0], &pass)?;
        }
        Ok(sess)
    }

    fn get_keys(&self) -> QStringList {
        self.try_get_keys().unwrap_or_default()
    }

    fn try_get_keys(&self) -> Result<QStringList, ssh2::Error> {
        let sess = Session::new()?;
        let mut agent = sess.agent()?;

        agent.connect()?;
        agent.list_identities()?;
        Ok(agent
            .identities()?
            .into_iter()
            .map(|i| {
                "🔑".to_string()
                    + i.comment()
                    + &i.blob()
                        .iter()
                        .map(|&byte| byte as u32)
                        .sum::<u32>()
                        .to_string()
            })
            .collect())
    }

    pub fn check_installation(&mut self, params: String, pass: String) {
        session!(self, params, pass, |sess: Session| {
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

    pub fn see_logs(&mut self, params: String, pass: String, node_type: String) {
        session!(self, params, pass, |sess: Session| {
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

    fn command(&mut self, params: String, pass: String, cmd: String) {
        session!(self, params, pass, |sess: Session| {
            let mut channel = sess.channel_session().unwrap();
            channel.exec(&cmd).unwrap();
            channel.wait_eof().unwrap();
            channel.wait_close().unwrap();
        });
    }

    pub fn start_node(&mut self, params: String, pass: String, node_type: String) {
        self.command(
            params.clone(),
            pass.clone(),
            format!("sudo systemctl start duniter-{}.service", node_type),
        );
        self.see_logs(params, pass, node_type);
    }

    pub fn stop_node(&mut self, params: String, pass: String, node_type: String) {
        self.command(
            params.clone(),
            pass.clone(),
            format!("sudo systemctl stop duniter-{}.service", node_type),
        );
        self.see_logs(params, pass, node_type);
    }

    pub fn start_oracle(&mut self, params: String, pass: String) {
        self.command(
            params.clone(),
            pass.clone(),
            "sudo systemctl start distance-oracle.service".into(),
        );
        session!(self, params, pass, |sess: Session| {
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

    pub fn stop_oracle(&mut self, params: String, pass: String) {
        self.command(
            params.clone(),
            pass.clone(),
            "sudo systemctl stop distance-oracle.service".into(),
        );
        session!(self, params, pass, |sess: Session| {
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

    pub fn get_config(&mut self, params: String, pass: String) {
        session!(self, params, pass, |sess: Session| {
            let mut channel = sess.channel_session().unwrap();
            channel.exec("sudo cat /etc/duniter/env_file").unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            self.config = s.trim().into();
            self.config_changed();
            channel.wait_close().unwrap();
        });
    }

    pub fn write_config(&mut self, params: String, pass: String, config: String) {
        self.command(
            params.clone(),
            pass.clone(),
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

    let icon_path = CString::new(":/assets/duniternodemanager.png").expect("CString::new failed");
    let icon_path_ptr = icon_path.as_ptr();
    cpp!(unsafe [icon_path_ptr as "const char *"] {
        QGuiApplication::setWindowIcon(QIcon(icon_path_ptr));
    });

    engine.exec();
}
