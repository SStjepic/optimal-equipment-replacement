use clap::Parser;
use plotters::prelude::*;
use std::collections::HashMap;
use std::error::Error;

#[derive(Parser)]
struct Args {
    language: String,
    execution: String,
    graph: String,
}

type MachineProfits = HashMap<i32, Vec<(i32, f64)>>;
type MachineReplacements = HashMap<i32, Vec<i32>>;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let base_path = match args.language.as_str() {
        "python" => "../../python/",
        "rust" => "../data/",
        _ => panic!("Language must be 'python' or 'rust'"),
    };

    let profit_path = format!(
        "{}machine_profit_{}.csv",
        base_path, args.execution
    );

    let replacement_path = format!(
        "{}machine_replacements_{}.csv",
        base_path, args.execution
    );

    println!("Loading: {}", profit_path);
    println!("Loading: {}", replacement_path);

    let profits = load_machine_profits(&profit_path)?;
    let replacements = load_replacements(&replacement_path)?;

    match args.graph.as_str() {
        "profit_per_year" => draw_profit_per_year(&profits, &args.language, &args.execution)?,
        "cumulative_profit" => draw_cumulative_profit(&profits, &args.language, &args.execution)?,
        "machine_profit" => draw_machine_profit(&profits, &replacements, &args.language, &args.execution)?,
        "replacements_per_year" => draw_replacements_per_year(&replacements, &args.language, &args.execution)?,
        "optimal_replacement" => draw_optimal_replacement(&replacements, &args.language, &args.execution)?,
        _ => println!("Unknown graph type"),
    }

    Ok(())
}

fn load_machine_profits(path: &str) -> Result<MachineProfits, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    let headers = rdr.headers()?.clone();
    let mut data: MachineProfits = HashMap::new();

    for result in rdr.records() {
        let record = result?;

        let machine_id: i32 = match record.get(0).unwrap().trim().parse() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let mut yearly_data = Vec::new();

        for (i, header) in headers.iter().enumerate().skip(1) {
            let year_str = header
                .replace("year_", "")
                .replace("_profit", "")
                .trim()
                .to_string();

            let year: i32 = match year_str.parse() {
                Ok(y) => y,
                Err(_) => continue,
            };

            let profit: f64 = record.get(i)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0.0);

            yearly_data.push((year, profit));
        }

        data.insert(machine_id, yearly_data);
    }

    Ok(data)
}

fn load_replacements(path: &str) -> Result<MachineReplacements, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    let mut data = HashMap::new();

    for result in rdr.records() {
        let record = result?;

        let machine_id: i32 = match record.get(0).unwrap().trim().parse() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let years: Vec<i32> = record.get(1)
            .unwrap_or("")
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        data.insert(machine_id, years);
    }

    Ok(data)
}

fn draw_profit_per_year(profits: &MachineProfits, lang: &str, exec: &str) -> Result<(), Box<dyn Error>> {
    let mut totals: HashMap<i32, f64> = HashMap::new();

    for machine in profits.values() {
        for (year, profit) in machine {
            *totals.entry(*year).or_insert(0.0) += profit;
        }
    }

    let mut data: Vec<(i32, f64)> = totals.into_iter().collect();
    data.sort_by_key(|(year, _)| *year);

    let max_year = data.last().unwrap().0;
    let max_profit = data.iter().map(|(_, p)| *p).fold(0.0, f64::max);

    let file_name = format!("{}_{}_profit_per_year.png", lang, exec);
    let root = BitMapBackend::new(&file_name, (1200, 700))
        .into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Ukupan profit po godinama({} - {})", lang, exec), ("sans-serif", 35))
        .margin(30)
        .x_label_area_size(50)
        .y_label_area_size(120)
        .build_cartesian_2d(0..max_year, 0f64..max_profit)?;

    chart
        .configure_mesh()
        .x_desc("Godina")
        .y_desc("Ukupan profit")
        .y_label_formatter(&|x| format!("{:.0} €", x))
        .axis_desc_style(("sans-serif", 25))
        .label_style(("sans-serif", 18))
        .draw()?;

    chart.draw_series(LineSeries::new(data, &BLUE))?;

    root.present()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", &file_name])
        .spawn()?;

    println!("Grafik sačuvan kao {}", &file_name);

    Ok(())
}

fn draw_cumulative_profit(profits: &MachineProfits, lang: &str, exec: &str) -> Result<(), Box<dyn Error>> {
    let mut totals: HashMap<i32, f64> = HashMap::new();

    for machine in profits.values() {
        for (year, profit) in machine {
            *totals.entry(*year).or_insert(0.0) += profit;
        }
    }

    let mut data: Vec<(i32, f64)> = totals.into_iter().collect();
    data.sort_by_key(|(year, _)| *year);

    let mut cumulative = 0.0;
    let mut cumulative_data = Vec::new();

    for (year, profit) in data {
        cumulative += profit;
        cumulative_data.push((year, cumulative));
    }

    let max_year = cumulative_data.last().map(|d| d.0).unwrap_or(0);
    let max_profit = cumulative_data.iter().map(|(_, p)| *p).fold(0.0, f64::max);

    let file_name = format!("{}_{}_cumulative_profit.png", lang, exec);
    let root = BitMapBackend::new(&file_name, (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Kumulativni profit ({} - {})", lang, exec), ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(50) 
        .y_label_area_size(120)
        .build_cartesian_2d(0..max_year, 0f64..max_profit)?;

    chart.configure_mesh()
        .x_desc("Godina")
        .y_desc("Kumulativni profit")
        .y_label_formatter(&|y| format!("{:.0} €", y))
        .axis_desc_style(("sans-serif", 20))
        .label_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(LineSeries::new(cumulative_data, &GREEN))?;

    root.present()?;
    
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", &file_name])
        .spawn()?;
        
    println!("Grafik sačuvan kao {}", &file_name);

    Ok(())
}

fn draw_machine_profit(profits: &MachineProfits, replacements: &MachineReplacements, lang: &str, exec: &str) -> Result<(), Box<dyn Error>> {
    let file_name = format!("{}_{}_machine_profit.png", lang, exec);
    let root = BitMapBackend::new(&file_name, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let (chart_area, _legend_area) = root.split_horizontally(1000);

    let max_year = profits.values().flat_map(|v| v.iter().map(|(y, _)| *y)).max().unwrap_or(0);
    let max_profit = profits.values().flat_map(|v| v.iter().map(|(_, p)| *p)).fold(0.0, f64::max);

    let mut chart = ChartBuilder::on(&chart_area)
        .caption(format!("Profit po mašini ({} - {})", lang, exec), ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(100)
        .build_cartesian_2d(0..max_year, 0f64..max_profit)?;

    chart.configure_mesh()
        .y_label_formatter(&|y| format!("{:.0} €", y))
        .draw()?;

    for (machine_id, data) in profits {
        let machine_id_usize = *machine_id as usize;
        let color = Palette99::pick(machine_id_usize);
        
        chart.draw_series(LineSeries::new(data.clone(), &color))?
            .label(format!("Mašina {}", machine_id))
            .legend(move |(x, y)| {
                let color = Palette99::pick(machine_id_usize);
                PathElement::new(vec![(x, y), (x + 20, y)], color)
            });

        if let Some(repl_years) = replacements.get(machine_id) {
            for year in repl_years {
                if let Some((_, profit)) = data.iter().find(|(y, _)| y == year) {
                    chart.draw_series(std::iter::once(
                        Circle::new((*year, *profit), 5, RED.filled())
                    ))?;
                }
            }
        }
    }

    chart.configure_series_labels()
        .border_style(&BLACK)
        .background_style(WHITE.mix(0.8)) 
        .draw()?;

    root.present()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", &file_name])
        .spawn()?;
    println!("Grafik sačuvan kao {}", &file_name);
    Ok(())
}

fn draw_replacements_per_year(replacements: &MachineReplacements, lang: &str, exec: &str) -> Result<(), Box<dyn Error>> {
    let mut counts: HashMap<i32, i32> = HashMap::new();

    for years in replacements.values() {
        for year in years {
            *counts.entry(*year).or_insert(0) += 1;
        }
    }

    let mut data: Vec<(i32, i32)> = counts.into_iter().collect();
    data.sort_by_key(|(year, _)| *year);

    let max_year = data.last().map(|d| d.0).unwrap_or(0);
    let min_year = data.first().map(|d| d.0).unwrap_or(0);
    let max_count = data.iter().map(|(_, c)| *c).max().unwrap_or(0);

    let file_name = format!("{}_{}_replacements_per_year.png", lang, exec);
    let root = BitMapBackend::new(&file_name, (1200, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Zamene po godinama ({} - {})", lang, exec), ("sans-serif", 35))
        .margin(30)
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d((min_year as f64 - 0.5)..(max_year as f64 + 0.5), 0f64..(max_count as f64 + 1.0))?;

    chart.configure_mesh()
        .x_desc("Godina")
        .y_desc("Broj zamena")
        .x_labels((max_year - min_year + 1) as usize) 
        .light_line_style(TRANSPARENT)
        .axis_desc_style(("sans-serif", 25))
        .label_style(("sans-serif", 15))
        .x_label_formatter(&|x| format!("{:.0}", x.round()))
        .draw()?;

    let bar_width = 0.5;

    chart.draw_series(
        data.iter()
        .map(|(year, count)| {
            let x_center = *year as f64;
            let x0 = x_center - (bar_width / 2.0);
            let x1 = x_center + (bar_width / 2.0);
            Rectangle::new([(x0, 0.0), (x1, *count as f64)], BLUE.filled())
        }),
    )?;

    root.present()?;
    
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/C", "start", &file_name]).spawn()?;
        
    println!("Grafik sačuvan kao {}", &file_name);
    Ok(())
}

fn draw_optimal_replacement(replacements: &MachineReplacements, lang: &str, exec: &str) -> Result<(), Box<dyn Error>> {
    let file_name = format!("{}_{}_optimal_replacement.png", lang, exec);
    let root = BitMapBackend::new(&file_name, (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_machine = replacements.keys().max().cloned().unwrap_or(0);
    let max_year = replacements
        .values()
        .flat_map(|v| v.iter())
        .max()
        .cloned()
        .unwrap_or(1);

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Optimalna godina zamene ({} - {})", lang, exec), ("sans-serif", 35))
        .margin(30)
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(-0.5..(max_machine as f64 + 0.5), 0f64..(max_year as f64 + 2.0))?;

    chart.configure_mesh()
        .x_desc("ID Mašine")
        .y_desc("Godina zamene")
        .y_labels(max_year as usize + 1)
        .light_line_style(TRANSPARENT)
        .axis_desc_style(("sans-serif", 25))
        .label_style(("sans-serif", 18))
        .x_label_formatter(&|x| format!("{:.0}", x.round()))
        .y_label_formatter(&|y| format!("{:.0}", y))
        .draw()?;

    let bar_width = 0.6;

    for (machine_id, years) in replacements {
        if let Some(first_year) = years.first() {
            let x_center = *machine_id as f64;
            let x0 = x_center - (bar_width / 2.0);
            let x1 = x_center + (bar_width / 2.0);
            
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(x0, 0.0), (x1, *first_year as f64)],
                    GREEN.mix(0.8).filled(),
                ),
            ))?;
        }
    }

    root.present()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", &file_name])
        .spawn()?;
    println!("Grafik sačuvan kao {}", &file_name);
    Ok(())
}