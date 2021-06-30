use crate::prelude::*;

#[test]
fn regression_test_issue_267() {
    let p1 = (338, 122);
    let p2 = (365, 122);

    let mut backend = BitMapBackend::new("blub.png", (800, 600));

    backend
        .draw_line(p1, p2, &RGBColor(0, 0, 0).stroke_width(0))
        .unwrap();
}
