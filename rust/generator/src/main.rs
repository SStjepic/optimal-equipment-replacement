use rand::Rng;
use serde::Serialize;
use std::error::Error;
use std::fs::{self, File};

#[derive(Serialize)]
struct Machine {
    id: i32,
    initial_age: i32,
    base_profit: f64,
    profit_decay: f64,
    maint_base: f64,
    maint_growth: f64,
    resale_base: f64,
    resale_decay: f64,
    buy_price: f64,
    repair_cost: f64,
}

impl Machine {
    fn new_random(id: i32, rng: &mut impl Rng) -> Self {
        let base_profit = (rng.gen_range(4000.0..7000.0) * 100.0_f64).round() / 100.0;
        let buy_price = (base_profit * 1.5 * 100.0_f64).round() / 100.0;

        Self {
            id,
            initial_age: rng.gen_range(0..7),
            base_profit,
            profit_decay: (rng.gen_range(0.08..0.12) * 1000.0_f64).round() / 1000.0,
            maint_base: (rng.gen_range(300.0..600.0) * 100.0_f64).round() / 100.0,
            maint_growth: (rng.gen_range(1.05..1.10) * 1000.0_f64).round() / 1000.0,
            resale_base: (buy_price * 0.6 * 100.0_f64).round() / 100.0,
            resale_decay: (rng.gen_range(0.75..0.85) * 1000.0_f64).round() / 1000.0,
            buy_price,
            repair_cost: (rng.gen_range(800.0..2000.0) * 100.0_f64).round() / 100.0,
        }
    }
}

fn generate_machines(num_machines: i32) -> Result<(), Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    
    let path = "../data";
    fs::create_dir_all(path)?;
    
    let file_path = format!("{}/machines_input.csv", path);
    let file = File::create(&file_path)?;
    let mut writer = csv::Writer::from_writer(file);

    for i in 0..num_machines {
        let machine = Machine::new_random(i, &mut rng);
        writer.serialize(machine)?;
    }

    writer.flush()?;
    println!("File machines_input.csv has been successfully generated.");
    Ok(())
}

fn main() {
    if let Err(e) = generate_machines(100) {
        panic!("Error during generation: {}", e);
    }
}