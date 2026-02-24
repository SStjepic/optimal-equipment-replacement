use csv::Reader;
use plotters::prelude::*;
use serde::Deserialize;
use std::env;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    Language: String,
    Mode: String,
    Cores: u32,
    Mean_Time_s: f64,
}

fn create_plot(
    file_name: &str,
    title: &str,
    data: &[(f32, f32)],
    is_strong: bool,
    s_val: f64,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(file_name, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_cores = 8.0f32;
    let max_speedup = 8.5f32;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 40).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(1f32..max_cores, 0f32..max_speedup)?;

    chart.configure_mesh()
        .x_desc("Broj jezgara (P)")
        .y_desc("Ubrzanje (Speedup)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        (1..=8).map(|x| (x as f32, x as f32)),
        GREEN.stroke_width(2),
    ))?
    .label("Idealno (Linear)")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

    chart.draw_series(LineSeries::new(
        (1..=80).map(|p| {
            let x = p as f64 / 10.0;
            let y = if is_strong {
                1.0 / (s_val + (1.0 - s_val) / x)
            } else {
                s_val + (1.0 - s_val) * x
            };
            (x as f32, y as f32)
        }),
        BLUE.stroke_width(2),
    ))?
    .label(format!("Teorijski (s={:.1}%)", s_val * 100.0))
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart.draw_series(LineSeries::new(data.iter().cloned(), RED.stroke_width(4)))?
        .label("Izmereno")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    chart.draw_series(data.iter().map(|(x, y)| Circle::new((*x, *y), 5, RED.filled())))?;

    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cargo run <language> <mode>");
        return Ok(());
    }

    let lang_param = args[1].to_lowercase();
    let mode_param = args[2].to_lowercase();
    let is_strong = mode_param == "strong";
    
    let file_path = format!("../../python/report/scaling_results_{}.csv", lang_param);
    let mut rdr = Reader::from_path(&file_path)?;
    
    let mut filtered_records: Vec<Record> = Vec::new();
    for result in rdr.deserialize() {
        let rec: Record = result?;
        if rec.Mode.to_lowercase() == mode_param {
            filtered_records.push(rec);
        }
    }

    if filtered_records.is_empty() {
        return Err("Nema podataka za izabrane parametre.".into());
    }

    let t1 = filtered_records[0].Mean_Time_s;
    let last_rec = filtered_records.last().unwrap();
    let last_p = last_rec.Cores as f64;
    let last_t = last_rec.Mean_Time_s;

    let plot_data: Vec<(f32, f32)> = filtered_records.iter().map(|r| {
        let val = if is_strong {
            t1 / r.Mean_Time_s
        } else {
            (r.Cores as f64 * t1) / r.Mean_Time_s
        };
        (r.Cores as f32, val as f32)
    }).collect();

    let speedup_actual = if is_strong { t1 / last_t } else { (t1 * last_p) / last_t };

    let s_est = if is_strong {
        ((1.0 / speedup_actual) - (1.0 / last_p)) / (1.0 - (1.0 / last_p))
    } else {
        (last_p - speedup_actual) / (last_p - 1.0)
    }.max(0.0);

    let output_file = format!("speedup_{}_{}.png", lang_param, mode_param);
    let title = format!("{} {} Scaling Analysis", lang_param.to_uppercase(), mode_param.to_uppercase());

    create_plot(&output_file, &title, &plot_data, is_strong, s_est)?;

    println!("Jezik: {}, Mod: {}", lang_param, mode_param);
    println!("Izracunat serijski deo (s): {:.4}", s_est);
    println!("Grafik sacuvan kao: {}", output_file);

    Ok(())
}