use arhiv_core::Arhiv;
use arhiv_utils::generator::*;
use rs_utils::log::setup_logger;

#[tokio::main]
async fn main() {
    setup_logger();

    let arhiv = Arhiv::must_open();

    let mut faker = Faker::new(&arhiv).unwrap();

    faker.quantity_limits.insert("project".to_string(), 10);
    faker
        .field_size_limits
        .insert(("project".to_string(), "description".to_string()), (0, 1));
    faker
        .field_size_limits
        .insert(("task".to_string(), "description".to_string()), (0, 1));

    faker.create_fakes("note");
    faker.create_fakes("project");

    arhiv.sync().await.expect("must be able to sync");

    println!("DONE");
}
