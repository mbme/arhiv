use arhiv::Arhiv;
use arhiv_modules::generator::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let arhiv = Arhiv::must_open();

    let mut faker = Faker::new();

    faker.quantity_limits.insert("project".to_string(), 10);
    faker
        .field_size_limits
        .insert(("task".to_string(), "description".to_string()), (0, 1));

    faker.create_fakes("note", &arhiv);
    faker.create_fakes("project", &arhiv);

    arhiv.sync().await.expect("must be able to sync");

    println!("DONE");
}
