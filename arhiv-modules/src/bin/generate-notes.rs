use arhiv::entities::Document;
use arhiv::Arhiv;
use arhiv_modules::generator::*;
use arhiv_modules::modules::DocumentDataManager;

#[tokio::main]
async fn main() {
    env_logger::init();

    let arhiv = Arhiv::must_open();
    let attachments = create_attachments();
    let generator = Generator::new(&attachments);
    let manager = DocumentDataManager::new();

    for _ in 0..30 {
        let mut data = manager.create("note".to_string()).unwrap();
        manager.gen_data(&mut data, &generator).unwrap();

        let mut document = Document::new(data.into());
        manager.update_refs(&mut document).unwrap();

        arhiv
            .stage_document(document, attachments.clone())
            .expect("must be able to save document");
    }

    arhiv.sync().await.expect("must be able to sync");

    log::info!("Generated 30 notes!");
}
