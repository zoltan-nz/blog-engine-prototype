use specta::ts::{BigIntExportBehavior, ExportConfiguration};

fn main() {
    // TypeId::of generates a runtime call, pulling in the backend lib's object
    // files and triggering specta's ctor type registrations before main() runs.
    let _ = std::any::TypeId::of::<backend::types::WsEnvelope>();

    let out = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../frontend/src/lib/types/bindings.ts"
    );

    let cfg = ExportConfiguration::new().bigint(BigIntExportBehavior::Number);

    specta::export::ts_with_cfg(out, &cfg).expect("failed to export TypeScript bindings");

    println!("exported bindings → {out}");
}
