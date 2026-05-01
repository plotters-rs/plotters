//! Dark-theme line chart with on-plot annotations: data line + circle
//! markers, dashed linear-fit extrapolation, horizontal baseline, two
//! vertical reference lines (dashed + dotted), per-point value labels, a
//! bottom-left summary panel, and a built-in legend in the upper right.
//!
//! Uses the same `with_fonts` pattern as [`dynamic_font`](dynamic_font.rs) so
//! the example renders without depending on the host's installed fonts.

use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::sync::Arc;

const ROBOTO_REGULAR: &[u8] = include_bytes!("fonts/Roboto-Regular.ttf");
const ROBOTO_BOLD: &[u8] = include_bytes!("fonts/Roboto-Bold.ttf");

const OUT_FILE_NAME: &str = "plotters-doc-data/modern.png";
const FONT: &str = "Roboto";

const BG: RGBColor = RGBColor(28, 32, 48);
const PANEL: RGBColor = RGBColor(40, 44, 60);
const CYAN: RGBColor = RGBColor(102, 204, 255);
const GREEN: RGBColor = RGBColor(140, 250, 130);
const BASELINE: RGBColor = RGBColor(255, 130, 130);
const ORANGE: RGBColor = RGBColor(255, 200, 90);
const PURPLE: RGBColor = RGBColor(170, 140, 240);

const DATA: &[(i32, f64)] = &[
    (96, 7.3214),
    (224, 7.1289),
    (352, 6.8956),
    (480, 6.6943),
    (608, 6.4877),
    (736, 6.2731),
    (864, 6.0840),
];

const SLOPE: f64 = -0.001627;
const INTERCEPT: f64 = 7.4790;
const CURRENT_STEP: i32 = 960;
const TARGET_STEP: i32 = 1216;
const BASELINE_VALUE: f64 = 7.6;

fn fit(step: i32) -> f64 {
    INTERCEPT + SLOPE * step as f64
}

fn solid(color: &RGBColor, width: u32) -> ShapeStyle {
    ShapeStyle {
        color: color.mix(0.95).to_rgba(),
        filled: false,
        stroke_width: width,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(OUT_FILE_NAME, (1280, 720))
        .into_drawing_area()
        .with_fonts([
            (FONT, FontStyle::Normal, Arc::<[u8]>::from(ROBOTO_REGULAR)),
            (FONT, FontStyle::Bold, Arc::<[u8]>::from(ROBOTO_BOLD)),
        ]);
    root.fill(&BG)?;

    let title_style = (FONT, 26, FontStyle::Bold)
        .into_font()
        .color(&WHITE.mix(0.95));
    let root = root.titled(
        "Phase-2 Distillation Eval Loss - 9-Layer 1.4B SyntheticBook Run",
        title_style,
    )?;

    let label_color = WHITE.mix(0.78);
    let axis_label_style = (FONT, 14).into_font().color(&label_color);
    let axis_desc_style = (FONT, 16).into_font().color(&label_color);

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .margin_top(10)
        .set_label_area_size(LabelAreaPosition::Left, 70)
        .set_label_area_size(LabelAreaPosition::Bottom, 50)
        .build_cartesian_2d(40i32..1260i32, 5.30f64..7.70f64)?;

    chart
        .configure_mesh()
        .x_labels(13)
        .y_labels(6)
        .bold_line_style(WHITE.mix(0.12))
        .light_line_style(WHITE.mix(0.05))
        .axis_style(WHITE.mix(0.35))
        .x_desc("Training step")
        .y_desc("Eval cross-entropy nats, lower is better")
        .x_label_style(axis_label_style.clone())
        .y_label_style(axis_label_style)
        .axis_desc_style(axis_desc_style)
        .draw()?;

    let baseline_style = solid(&BASELINE, 2);
    chart
        .draw_series(DashedLineSeries::new(
            [(40, BASELINE_VALUE), (1260, BASELINE_VALUE)],
            2,
            5,
            baseline_style,
        ))?
        .label(format!("Untrained init baseline: {:.1}", BASELINE_VALUE))
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 24, y)], baseline_style));

    let fit_style = solid(&GREEN, 2);
    let fit_points: Vec<(i32, f64)> = (40..=1260).step_by(4).map(|x| (x, fit(x))).collect();
    chart
        .draw_series(DashedLineSeries::new(fit_points, 8, 6, fit_style))?
        .label("Linear fit (-0.208 nats / 128 steps)")
        .legend(move |(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 8, y), (x + 14, y), (x + 24, y)],
                fit_style,
            )
        });

    chart
        .draw_series(LineSeries::new(
            DATA.iter().copied(),
            CYAN.stroke_width(2),
        ))?
        .label("Confirmed eval loss")
        .legend(|(x, y)| {
            EmptyElement::at((x + 12, y))
                + PathElement::new(vec![(-12, 0), (12, 0)], CYAN.stroke_width(2))
                + Circle::new((0, 0), 4, CYAN.filled())
        });
    chart.draw_series(
        DATA.iter()
            .map(|&(x, y)| Circle::new((x, y), 5, CYAN.filled())),
    )?;

    let value_label_style = (FONT, 13)
        .into_font()
        .color(&WHITE.mix(0.9))
        .pos(Pos::new(HPos::Center, VPos::Bottom));
    chart.draw_series(DATA.iter().map(|&(x, y)| {
        EmptyElement::at((x, y))
            + Text::new(format!("{:.4}", y), (0, -10), value_label_style.clone())
    }))?;

    let current_style = solid(&ORANGE, 2);
    chart
        .draw_series(DashedLineSeries::new(
            [(CURRENT_STEP, 5.30f64), (CURRENT_STEP, 7.70f64)],
            8,
            5,
            current_style,
        ))?
        .label(format!("Current step {}, eval queued", CURRENT_STEP))
        .legend(move |(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 6, y), (x + 12, y), (x + 18, y), (x + 24, y)],
                current_style,
            )
        });

    let target_style = solid(&PURPLE, 2);
    chart
        .draw_series(DashedLineSeries::new(
            [(TARGET_STEP, 5.30f64), (TARGET_STEP, 7.70f64)],
            2,
            5,
            target_style,
        ))?
        .label(format!("Target step {}", TARGET_STEP))
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 24, y)], target_style));

    let fit_label_left = (FONT, 13)
        .into_font()
        .color(&WHITE.mix(0.85))
        .pos(Pos::new(HPos::Left, VPos::Top));
    let fit_label_right = (FONT, 13)
        .into_font()
        .color(&WHITE.mix(0.85))
        .pos(Pos::new(HPos::Right, VPos::Top));
    chart.draw_series(std::iter::once(
        EmptyElement::at((CURRENT_STEP, fit(CURRENT_STEP)))
            + Text::new(
                format!("fit ~{:.3}", fit(CURRENT_STEP)),
                (8, 6),
                fit_label_left,
            ),
    ))?;
    chart.draw_series(std::iter::once(
        EmptyElement::at((TARGET_STEP, fit(TARGET_STEP)))
            + Text::new(
                format!("fit ~{:.3}", fit(TARGET_STEP)),
                (-8, 6),
                fit_label_right,
            ),
    ))?;

    draw_text_box(
        &mut chart,
        (90, 5.83),
        (560, 5.40),
        &[
            "Confirmed drop: 1.2374 nats (96 -> 864)",
            "Latest confirmed: 6.0840 at step 864",
            "Step 960 eval not yet logged",
        ],
        14,
    )?;

    let legend_font = (FONT, 13).into_font().color(&WHITE.mix(0.92));
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .margin(10)
        .legend_area_size(28)
        .background_style(PANEL.mix(0.85))
        .border_style(WHITE.mix(0.25))
        .label_font(legend_font)
        .draw()?;

    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}

/// Filled panel (background + border) plus stacked text lines, all in plot
/// coordinates. The y axis points down on screen, so `top_left.1 >
/// bottom_right.1`.
fn draw_text_box<DB: DrawingBackend>(
    chart: &mut ChartContext<
        '_,
        DB,
        Cartesian2d<plotters::coord::types::RangedCoordi32, plotters::coord::types::RangedCoordf64>,
    >,
    top_left: (i32, f64),
    bottom_right: (i32, f64),
    lines: &[&str],
    font_size: u32,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB::ErrorType: 'static,
{
    chart.draw_series(std::iter::once(Rectangle::new(
        [top_left, bottom_right],
        ShapeStyle {
            color: PANEL.mix(0.85).to_rgba(),
            filled: true,
            stroke_width: 1,
        },
    )))?;
    chart.draw_series(std::iter::once(Rectangle::new(
        [top_left, bottom_right],
        ShapeStyle {
            color: WHITE.mix(0.25).to_rgba(),
            filled: false,
            stroke_width: 1,
        },
    )))?;
    let text_style = (FONT, font_size)
        .into_font()
        .color(&WHITE.mix(0.9))
        .pos(Pos::new(HPos::Left, VPos::Top));
    let line_step = (font_size as i32) + 4;
    for (i, line) in lines.iter().enumerate() {
        chart.draw_series(std::iter::once(
            EmptyElement::at(top_left)
                + Text::new(
                    line.to_string(),
                    (10, 8 + (i as i32) * line_step),
                    text_style.clone(),
                ),
        ))?;
    }
    Ok(())
}

#[test]
fn entry_point() {
    main().unwrap()
}
