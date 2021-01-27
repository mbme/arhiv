use arhiv::Arhiv;
use arhiv_utils::generator::*;
use rs_utils::log::setup_logger;

#[tokio::main]
async fn main() {
    setup_logger();

    let arhiv = Arhiv::must_open();

    let mut faker = Faker::new();

    faker.quantity_limits.insert("project".to_string(), 10);
    faker
        .field_size_limits
        .insert(("project".to_string(), "description".to_string()), (0, 1));
    faker
        .field_size_limits
        .insert(("task".to_string(), "description".to_string()), (0, 1));
    faker
        .field_size_limits
        .insert(("project".to_string(), "title".to_string()), (1, 4));

    faker.create_fakes("note", &arhiv);
    faker.create_fakes("project", &arhiv);

    arhiv.sync().await.expect("must be able to sync");

    println!("DONE");
}
