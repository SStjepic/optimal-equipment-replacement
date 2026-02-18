use rand::Rng;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};

#[derive(Debug, Deserialize, Clone)]
struct MachineInput {
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

struct EquipmentReplacementSim {
    machines: Vec<MachineInput>,
    horizon_years: usize,
    simulations: usize,
    market_growth_rate: f64,
}

impl EquipmentReplacementSim {
    fn new(input_file: &str, horizon_years: usize, simulations: usize) -> Result<Self, Box<dyn Error>> {
        let file = File::open(input_file)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut machines = Vec::new();

        for result in rdr.deserialize() {
            machines.push(result?);
        }

        Ok(Self {
            machines,
            horizon_years,
            simulations,
            market_growth_rate: 0.01,
        })
    }

    fn get_revenue(&self, age: i32, machine: &MachineInput, year: usize) -> f64 {
        let market_factor = (1.0 + self.market_growth_rate).powi(year as i32);
        (machine.base_profit * (1.0 - machine.profit_decay).powi(age)) * market_factor
    }

    fn get_maint_cost(&self, age: i32, machine: &MachineInput) -> f64 {
        machine.maint_base * machine.maint_growth.powi(age)
    }

    fn get_resale_value(&self, age: i32, machine: &MachineInput) -> f64 {
        machine.resale_base * machine.resale_decay.powi(age)
    }

    fn get_breakdown_prob(&self, age: i32) -> f64 {
        (0.03 * age as f64).min(0.6)
    }

    fn run_monte_carlo(&self) -> (Vec<String>, Vec<Vec<f64>>) {
        let mut all_replacement_events = Vec::new();
        let mut machine_profits = Vec::new();
        let mut rng = rand::thread_rng();

        for m in &self.machines {
            let mut accumulated_profits = vec![0.0; self.horizon_years];
            let mut replacement_counts: BTreeMap<usize, usize> = BTreeMap::new();

            for _ in 0..self.simulations {
                let mut current_age = m.initial_age;

                for year in 0..self.horizon_years {
                    let revenue = self.get_revenue(current_age, m, year);
                    let mut maint = self.get_maint_cost(current_age, m);

                    if rng.gen_range(0.0..1.0) < self.get_breakdown_prob(current_age) {
                        maint += m.repair_cost;
                    }

                    let mut yearly_profit = revenue - maint;
                    let threshold = m.base_profit * 0.4;

                    if yearly_profit < threshold || maint > (revenue * 0.6) {
                        let net_cost = m.buy_price - self.get_resale_value(current_age, m);
                        yearly_profit -= net_cost;
                        current_age = 0;
                        *replacement_counts.entry(year + 1).or_insert(0) += 1;
                    } else {
                        current_age += 1;
                    }

                    accumulated_profits[year] += yearly_profit;
                }
            }

            let typical_years: Vec<String> = replacement_counts
                .iter()
                .filter(|&(_, &count)| count > (self.simulations / 5))
                .map(|(year, _)| year.to_string())
                .collect();

            all_replacement_events.push(typical_years.join(", "));
            
            let avg_profits: Vec<f64> = accumulated_profits
                .into_iter()
                .map(|p| (p / self.simulations as f64 * 100.0).round() / 100.0)
                .collect();
            machine_profits.push(avg_profits);
        }

        (all_replacement_events, machine_profits)
    }

    fn save_results(&self, logs: Vec<String>, profits: Vec<Vec<f64>>) -> Result<(), Box<dyn Error>> {
        let path = "../data";
        fs::create_dir_all(path)?;

        let mut file_path = format!("{}/machine_profit_sequential.csv", path);
        let mut p_writer = csv::Writer::from_path(file_path)?;
        let mut headers = vec!["machine_id".to_string()];
        for i in 1..=self.horizon_years {
            headers.push(format!("year_{}_profit", i));
        }
        p_writer.write_record(&headers)?;

        let mut system_total = vec![0.0; self.horizon_years];

        for (idx, m_profit) in profits.iter().enumerate() {
            let mut row = vec![self.machines[idx].id.to_string()];
            for (year, &val) in m_profit.iter().enumerate() {
                row.push(val.to_string());
                system_total[year] += val;
            }
            p_writer.write_record(&row)?;
        }

        let mut total_row = vec!["SYSTEM_TOTAL".to_string()];
        for val in system_total {
            total_row.push(format!("{:.2}", val));
        }
        p_writer.write_record(&total_row)?;
        p_writer.flush()?;

        file_path = format!("{}/machine_replacements_sequential.csv", path);
        let mut l_writer = csv::Writer::from_path(file_path)?;
        l_writer.write_record(&["machine_id", "replacement_years"])?;
        for (idx, log) in logs.iter().enumerate() {
            l_writer.write_record(&[self.machines[idx].id.to_string(), log.clone()])?;
        }
        l_writer.flush()?;

        println!("Results saved to data/ folder.");
        Ok(())
    }
}

fn main() {
    match EquipmentReplacementSim::new("../data/machines_input.csv", 30, 10000) {
        Ok(sim) => {
            let (logs, profits) = sim.run_monte_carlo();
            if let Err(e) = sim.save_results(logs, profits) {
                panic!("Error saving results: {}", e);
            }
        },
        Err(e) => panic!("Error loading input file: {}. Make sure 'data/machines_input.csv' exists.", e),
    }
}