// Data is pulled from https://covid.ourworldindata.org/data/owid-covid-data.json
use plotters::prelude::*;
use std::fs::File;
use std::io::BufReader;

use chrono::NaiveDate;
use std::str::FromStr;

#[derive(serde_derive::Deserialize)]
struct DailyData {
    date: String,
    #[serde(default)]
    new_cases_smoothed_per_million: f64,
    #[serde(default)]
    total_cases_per_million: f64,
}

#[derive(serde_derive::Deserialize)]
struct CountryData {
    data: Vec<DailyData>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root =
        BitMapBackend::gif("/tmp/tick_control.gif", (800, 600), 100)?.into_drawing_area();

    for a in 0..200 {

        root.fill(&WHITE)?;


        let mut chart = ChartBuilder::on(&root)
            .set_label_area_size(LabelAreaPosition::Left, (8).percent())
            .set_label_area_size(LabelAreaPosition::Bottom, (6).percent())
            .margin((1).percent())
            .build_cartesian_3d(
                (20u32..10_0000u32)
                    .log_scale()
                    .with_key_points(vec![50, 100, 200, 500, 1000, 10000]),
                (0u32..1000u32)
                    .log_scale()
                    .with_key_points(vec![2, 5, 10, 20, 50, 100, 200]),
                NaiveDate::from_ymd(2020, 1, 1)..NaiveDate::from_ymd(2020, 9, 5),
            )?;

        chart.with_projection(|mut pb| {
            pb.yaw = (1.57 - a as f64 / 100.0 * 1.57).abs();
            pb.into_matrix()
        });

        chart
            .configure_axes()
            .draw()?;

        let data: std::collections::HashMap<String, CountryData> = serde_json::from_reader(
            BufReader::new(File::open("plotters-doc-data/covid-data.json")?),
        )?;

        for (idx, &series) in ["USA", "CHN"]
            .iter()
            .enumerate()
        {
            let color = Palette99::pick(idx).mix(1.0);
            chart
                .draw_series(LineSeries::new(
                    data[series].data.iter().map(
                        |DailyData {
                             date,
                             new_cases_smoothed_per_million,
                             total_cases_per_million,
                             ..
                         }| (*total_cases_per_million as u32, *new_cases_smoothed_per_million as u32, chrono::NaiveDate::from_str(date).unwrap(),),
                    ),
                    color.stroke_width(1),
                ))?
                .label(series)
                .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
        }

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw()?;

        root.present()?;
    }

    Ok(())
}
