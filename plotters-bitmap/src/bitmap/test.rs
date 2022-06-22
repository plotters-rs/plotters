use super::*;

#[test]
fn test_bitmap_backend() {
    use plotters::prelude::*;

    let mut buffer = vec![0; 10 * 10 * 3];

    {
        let back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        let area = back.into_drawing_area();
        area.fill(&WHITE).unwrap();
        area.draw(&PathElement::new(vec![(0, 0), (10, 10)], RED.filled()))
            .unwrap();
        area.present().unwrap();
    }

    for i in 0..10 {
        assert_eq!(buffer[i * 33], 255);
        assert_eq!(buffer[i * 33 + 1], 0);
        assert_eq!(buffer[i * 33 + 2], 0);
        buffer[i * 33] = 255;
        buffer[i * 33 + 1] = 255;
        buffer[i * 33 + 2] = 255;
    }

    assert!(buffer.into_iter().all(|x| x == 255));
}

#[cfg(test)]
#[test]
fn test_bitmap_backend_fill_half() {
    use plotters::prelude::*;
    let mut buffer = vec![0; 10 * 10 * 3];

    {
        let back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        let area = back.into_drawing_area();
        area.draw(&Rectangle::new([(0, 0), (5, 10)], RED.filled()))
            .unwrap();
        area.present().unwrap();
    }
    for x in 0..10 {
        for y in 0..10 {
            assert_eq!(
                buffer[(y * 10 + x) as usize * 3 + 0],
                if x <= 5 { 255 } else { 0 }
            );
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 1], 0);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 2], 0);
        }
    }

    let mut buffer = vec![0; 10 * 10 * 3];

    {
        let back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        let area = back.into_drawing_area();
        area.draw(&Rectangle::new([(0, 0), (10, 5)], RED.filled()))
            .unwrap();
        area.present().unwrap();
    }
    for x in 0..10 {
        for y in 0..10 {
            assert_eq!(
                buffer[(y * 10 + x) as usize * 3 + 0],
                if y <= 5 { 255 } else { 0 }
            );
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 1], 0);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 2], 0);
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_backend_blend() {
    use plotters::prelude::*;
    let mut buffer = vec![255; 10 * 10 * 3];

    {
        let back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        let area = back.into_drawing_area();
        area.draw(&Rectangle::new(
            [(0, 0), (5, 10)],
            RGBColor(0, 100, 200).mix(0.2).filled(),
        ))
        .unwrap();
        area.present().unwrap();
    }

    for x in 0..10 {
        for y in 0..10 {
            let (r, g, b) = if x <= 5 {
                (205, 225, 245)
            } else {
                (255, 255, 255)
            };
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 0], r);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 1], g);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 2], b);
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_backend_split_and_fill() {
    use plotters::prelude::*;
    let mut buffer = vec![255; 10 * 10 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        for (sub_backend, color) in back.split(&[5]).into_iter().zip([&RED, &GREEN].iter()) {
            sub_backend.into_drawing_area().fill(*color).unwrap();
        }
    }

    for x in 0..10 {
        for y in 0..10 {
            let (r, g, b) = if y < 5 { (255, 0, 0) } else { (0, 255, 0) };
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 0], r);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 1], g);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 2], b);
        }
    }
}

#[cfg(test)]
#[test]
fn test_draw_rect_out_of_range() {
    use plotters::prelude::*;
    let mut buffer = vec![0; 1099 * 1000 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));

        back.draw_line((1100, 0), (1100, 999), &RED.to_rgba())
            .unwrap();
        back.draw_line((0, 1100), (999, 1100), &RED.to_rgba())
            .unwrap();
        back.draw_rect((1100, 0), (1100, 999), &RED.to_rgba(), true)
            .unwrap();
    }

    for x in 0..1000 {
        for y in 0..1000 {
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 0], 0);
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 1], 0);
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 2], 0);
        }
    }
}

#[cfg(test)]
#[test]
fn test_draw_line_out_of_range() {
    use plotters::prelude::*;
    let mut buffer = vec![0; 1000 * 1000 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));

        back.draw_line((-1000, -1000), (2000, 2000), &WHITE.to_rgba())
            .unwrap();

        back.draw_line((999, -1000), (999, 2000), &WHITE.to_rgba())
            .unwrap();
    }

    for x in 0..1000 {
        for y in 0..1000 {
            let expected_value = if x == y || x == 999 { 255 } else { 0 };
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 0], expected_value);
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 1], expected_value);
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 2], expected_value);
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_blend_large() {
    use plotters::prelude::*;
    let mut buffer = vec![0; 1000 * 1000 * 3];

    for fill_color in [RED, GREEN, BLUE].iter() {
        buffer.iter_mut().for_each(|x| *x = 0);

        {
            let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));

            back.draw_rect((0, 0), (1000, 1000), &WHITE.mix(0.1), true)
                .unwrap(); // should be (24, 24, 24)
            back.draw_rect((0, 0), (100, 100), &fill_color.mix(0.5), true)
                .unwrap(); // should be (139, 24, 24)
        }

        for x in 0..1000 {
            for y in 0..1000 {
                let expected_value = if x <= 100 && y <= 100 {
                    let (r, g, b) = fill_color.to_rgba().rgb();
                    (
                        if r > 0 { 139 } else { 12 },
                        if g > 0 { 139 } else { 12 },
                        if b > 0 { 139 } else { 12 },
                    )
                } else {
                    (24, 24, 24)
                };
                assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 0], expected_value.0);
                assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 1], expected_value.1);
                assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 2], expected_value.2);
            }
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_bgrx_pixel_format() {
    use crate::bitmap_pixel::BGRXPixel;
    use plotters::prelude::*;
    let mut rgb_buffer = vec![0; 1000 * 1000 * 3];
    let mut bgrx_buffer = vec![0; 1000 * 1000 * 4];

    {
        let mut rgb_back = BitMapBackend::with_buffer(&mut rgb_buffer, (1000, 1000));
        let mut bgrx_back =
            BitMapBackend::<BGRXPixel>::with_buffer_and_format(&mut bgrx_buffer, (1000, 1000))
                .unwrap();

        rgb_back
            .draw_rect((0, 0), (1000, 1000), &BLACK, true)
            .unwrap();
        bgrx_back
            .draw_rect((0, 0), (1000, 1000), &BLACK, true)
            .unwrap();

        rgb_back
            .draw_rect(
                (0, 0),
                (1000, 1000),
                &RGBColor(0xaa, 0xbb, 0xcc).mix(0.85),
                true,
            )
            .unwrap();
        bgrx_back
            .draw_rect(
                (0, 0),
                (1000, 1000),
                &RGBColor(0xaa, 0xbb, 0xcc).mix(0.85),
                true,
            )
            .unwrap();

        rgb_back
            .draw_rect((0, 0), (1000, 1000), &RED.mix(0.85), true)
            .unwrap();
        bgrx_back
            .draw_rect((0, 0), (1000, 1000), &RED.mix(0.85), true)
            .unwrap();

        rgb_back.draw_circle((300, 300), 100, &GREEN, true).unwrap();
        bgrx_back
            .draw_circle((300, 300), 100, &GREEN, true)
            .unwrap();

        rgb_back.draw_rect((10, 10), (50, 50), &BLUE, true).unwrap();
        bgrx_back
            .draw_rect((10, 10), (50, 50), &BLUE, true)
            .unwrap();

        rgb_back
            .draw_rect((10, 10), (50, 50), &WHITE, true)
            .unwrap();
        bgrx_back
            .draw_rect((10, 10), (50, 50), &WHITE, true)
            .unwrap();

        rgb_back
            .draw_rect((10, 10), (15, 50), &YELLOW, true)
            .unwrap();
        bgrx_back
            .draw_rect((10, 10), (15, 50), &YELLOW, true)
            .unwrap();
    }

    for x in 0..1000 {
        for y in 0..1000 {
            assert!(
                (rgb_buffer[y * 3000 + x * 3 + 0] as i32
                    - bgrx_buffer[y * 4000 + x * 4 + 2] as i32)
                    .abs()
                    <= 1
            );
            assert!(
                (rgb_buffer[y * 3000 + x * 3 + 1] as i32
                    - bgrx_buffer[y * 4000 + x * 4 + 1] as i32)
                    .abs()
                    <= 1
            );
            assert!(
                (rgb_buffer[y * 3000 + x * 3 + 2] as i32
                    - bgrx_buffer[y * 4000 + x * 4 + 0] as i32)
                    .abs()
                    <= 1
            );
        }
    }
}
#[cfg(test)]
#[test]
fn test_draw_simple_lines() {
    use plotters::prelude::*;
    let mut buffer = vec![0; 1000 * 1000 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));
        back.draw_line((500, 0), (500, 1000), &WHITE.filled().stroke_width(5))
            .unwrap();
    }

    let nz_count = buffer.into_iter().filter(|x| *x != 0).count();

    assert_eq!(nz_count, 6 * 1000 * 3);
}

#[cfg(test)]
#[test]
fn test_bitmap_blit() {
    let src_bitmap: Vec<u8> = (0..100)
        .map(|y| (0..300).map(move |x| ((x * y) % 253) as u8))
        .flatten()
        .collect();

    use plotters::prelude::*;
    let mut buffer = vec![0; 1000 * 1000 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));
        back.blit_bitmap((500, 500), (100, 100), &src_bitmap[..])
            .unwrap();
    }

    for y in 0..1000 {
        for x in 0..1000 {
            if x >= 500 && x < 600 && y >= 500 && y < 600 {
                let lx = x - 500;
                let ly = y - 500;
                assert_eq!(buffer[y * 3000 + x * 3 + 0] as usize, (ly * lx * 3) % 253);
                assert_eq!(
                    buffer[y * 3000 + x * 3 + 1] as usize,
                    (ly * (lx * 3 + 1)) % 253
                );
                assert_eq!(
                    buffer[y * 3000 + x * 3 + 2] as usize,
                    (ly * (lx * 3 + 2)) % 253
                );
            } else {
                assert_eq!(buffer[y * 3000 + x * 3 + 0], 0);
                assert_eq!(buffer[y * 3000 + x * 3 + 1], 0);
                assert_eq!(buffer[y * 3000 + x * 3 + 2], 0);
            }
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
#[cfg(test)]
mod test {
    use crate::BitMapBackend;
    use image::{ImageBuffer, Rgb};
    use plotters::prelude::*;
    use plotters::style::text_anchor::{HPos, Pos, VPos};
    use std::fs;
    use std::path::Path;

    static DST_DIR: &str = "target/test/bitmap";

    fn checked_save_file(name: &str, content: &[u8], w: u32, h: u32) {
        /*
          Please use the PNG file to manually verify the results.
        */
        assert!(content.iter().any(|x| *x != 0));
        fs::create_dir_all(DST_DIR).unwrap();
        let file_name = format!("{}.png", name);
        let file_path = Path::new(DST_DIR).join(file_name);
        println!("{:?} created", file_path);
        let img = ImageBuffer::<Rgb<u8>, &[u8]>::from_raw(w, h, content).unwrap();
        img.save(&file_path).unwrap();
    }

    fn draw_mesh_with_custom_ticks(tick_size: i32, test_name: &str) {
        let (width, height) = (500, 500);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
            root.fill(&WHITE).unwrap();

            let mut chart = ChartBuilder::on(&root)
                .caption("This is a test", ("sans-serif", 20))
                .set_all_label_area_size(40)
                .build_cartesian_2d(0..10, 0..10)
                .unwrap();

            chart
                .configure_mesh()
                .set_all_tick_mark_size(tick_size)
                .draw()
                .unwrap();
        }
        checked_save_file(test_name, &buffer, width, height);
    }

    #[test]
    fn test_draw_mesh_no_ticks() {
        draw_mesh_with_custom_ticks(0, "test_draw_mesh_no_ticks");
    }

    #[test]
    fn test_draw_mesh_negative_ticks() {
        draw_mesh_with_custom_ticks(-10, "test_draw_mesh_negative_ticks");
    }

    #[test]
    fn test_text_draw() {
        let (width, height) = (1500, 800);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
            root.fill(&WHITE).unwrap();
            let root = root
                .titled("Image Title", ("sans-serif", 60).into_font())
                .unwrap();

            let mut chart = ChartBuilder::on(&root)
                .caption("All anchor point positions", ("sans-serif", 20))
                .set_all_label_area_size(40)
                .build_cartesian_2d(0..100, 0..50)
                .unwrap();

            chart
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .x_desc("X Axis")
                .y_desc("Y Axis")
                .draw()
                .unwrap();

            let ((x1, y1), (x2, y2), (x3, y3)) = ((-30, 30), (0, -30), (30, 30));

            for (dy, trans) in [
                FontTransform::None,
                FontTransform::Rotate90,
                FontTransform::Rotate180,
                FontTransform::Rotate270,
            ]
            .iter()
            .enumerate()
            {
                for (dx1, h_pos) in [HPos::Left, HPos::Right, HPos::Center].iter().enumerate() {
                    for (dx2, v_pos) in [VPos::Top, VPos::Center, VPos::Bottom].iter().enumerate() {
                        let x = 150_i32 + (dx1 as i32 * 3 + dx2 as i32) * 150;
                        let y = 120 + dy as i32 * 150;
                        let draw = |x, y, text| {
                            root.draw(&Circle::new((x, y), 3, &BLACK.mix(0.5))).unwrap();
                            let style = TextStyle::from(("sans-serif", 20).into_font())
                                .pos(Pos::new(*h_pos, *v_pos))
                                .transform(trans.clone());
                            root.draw_text(text, &style, (x, y)).unwrap();
                        };
                        draw(x + x1, y + y1, "dood");
                        draw(x + x2, y + y2, "dog");
                        draw(x + x3, y + y3, "goog");
                    }
                }
            }
        }
        checked_save_file("test_text_draw", &buffer, width, height);
    }

    #[test]
    fn test_text_clipping() {
        let (width, height) = (500_i32, 500_i32);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width as u32, height as u32))
                .into_drawing_area();
            root.fill(&WHITE).unwrap();

            let style = TextStyle::from(("sans-serif", 20).into_font())
                .pos(Pos::new(HPos::Center, VPos::Center));
            root.draw_text("TOP LEFT", &style, (0, 0)).unwrap();
            root.draw_text("TOP CENTER", &style, (width / 2, 0))
                .unwrap();
            root.draw_text("TOP RIGHT", &style, (width, 0)).unwrap();

            root.draw_text("MIDDLE LEFT", &style, (0, height / 2))
                .unwrap();
            root.draw_text("MIDDLE RIGHT", &style, (width, height / 2))
                .unwrap();

            root.draw_text("BOTTOM LEFT", &style, (0, height)).unwrap();
            root.draw_text("BOTTOM CENTER", &style, (width / 2, height))
                .unwrap();
            root.draw_text("BOTTOM RIGHT", &style, (width, height))
                .unwrap();
        }
        checked_save_file("test_text_clipping", &buffer, width as u32, height as u32);
    }

    #[test]
    fn test_series_labels() {
        let (width, height) = (500, 500);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
            root.fill(&WHITE).unwrap();

            let mut chart = ChartBuilder::on(&root)
                .caption("All series label positions", ("sans-serif", 20))
                .set_all_label_area_size(40)
                .build_cartesian_2d(0..50, 0..50)
                .unwrap();

            chart
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .draw()
                .unwrap();

            chart
                .draw_series(std::iter::once(Circle::new((5, 15), 5, &RED)))
                .expect("Drawing error")
                .label("Series 1")
                .legend(|(x, y)| Circle::new((x, y), 3, RED.filled()));

            chart
                .draw_series(std::iter::once(Circle::new((5, 15), 10, &BLUE)))
                .expect("Drawing error")
                .label("Series 2")
                .legend(|(x, y)| Circle::new((x, y), 3, BLUE.filled()));

            for pos in vec![
                SeriesLabelPosition::UpperLeft,
                SeriesLabelPosition::MiddleLeft,
                SeriesLabelPosition::LowerLeft,
                SeriesLabelPosition::UpperMiddle,
                SeriesLabelPosition::MiddleMiddle,
                SeriesLabelPosition::LowerMiddle,
                SeriesLabelPosition::UpperRight,
                SeriesLabelPosition::MiddleRight,
                SeriesLabelPosition::LowerRight,
                SeriesLabelPosition::Coordinate(70, 70),
            ]
            .into_iter()
            {
                chart
                    .configure_series_labels()
                    .border_style(&BLACK.mix(0.5))
                    .position(pos)
                    .draw()
                    .expect("Drawing error");
            }
        }
        checked_save_file("test_series_labels", &buffer, width, height);
    }

    #[test]
    fn test_draw_pixel_alphas() {
        let (width, height) = (100_i32, 100_i32);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width as u32, height as u32))
                .into_drawing_area();
            root.fill(&WHITE).unwrap();
            for i in -20..20 {
                let alpha = i as f64 * 0.1;
                root.draw_pixel((50 + i, 50 + i), &BLACK.mix(alpha))
                    .unwrap();
            }
        }
        checked_save_file(
            "test_draw_pixel_alphas",
            &buffer,
            width as u32,
            height as u32,
        );
    }
}
