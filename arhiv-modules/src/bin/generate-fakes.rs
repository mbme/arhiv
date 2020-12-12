use arhiv::Arhiv;
use arhiv_modules::generator::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let arhiv = Arhiv::must_open();

    let faker = Faker::new();

    faker.create_fakes("note", &arhiv);
    faker.create_fakes("project", &arhiv);

    arhiv.sync().await.expect("must be able to sync");

    println!("DONE");
}
