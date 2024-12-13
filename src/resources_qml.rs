use qmetaobject::qrc;

pub fn init_resources() {
    qrc!(resources_qml,
        "src/qml/" as "qml/" {
            "main.qml",
        }
    );
    resources_qml();
    qrc!(resources_assets,
        "src/assets/" as "assets/" {
            "duniternodemanager.png",
        }
    );
    resources_assets();
}
