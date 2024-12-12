use qmetaobject::qrc;

pub fn init_resources() {
    qrc!(resources_qml,
        "src/qml/" as "qml/" {
            "main.qml",
        }
    );
    resources_qml();
}
