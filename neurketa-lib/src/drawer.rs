use svg::node::element::{Rectangle, Style, Text};
use svg::Document;

use crate::TimeSeries;

const RATIO: f64 = 1.0 / 20.0;

const COLOURS: &[&str] = &["#FFFF0B", "#E35000", "#BC0051", "#3E0070", "#270027"];

pub fn draw_time_series(
    filepath: impl AsRef<std::path::Path>,
    series: &TimeSeries,
) -> std::io::Result<()> {
    let gapless_width = series.events.iter().map(|e| e.delta).sum::<f64>();
    let height = RATIO * gapless_width;

    let mut time = 0.0;
    let mut x = 0.25 * gapless_width;
    let gap = 0.01 * gapless_width;
    let y = 0.25 * height;
    let mut labels = Vec::with_capacity(series.events.len() + 1);
    let mut boxes_and_labels = Vec::with_capacity(series.events.len());
    for (e, colour) in series.events.iter().zip(COLOURS.iter()) {
        let text = Text::new(e.id.as_str())
            .set("lengthAdjust", e.delta)
            .set("x", "50%")
            .set("y", "90%")
            .set("font-size", format!("{}px", height))
            .set("text-anchor", "middle")
            .set("alignment-baseline", "middle");

        let rect = Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", *colour);

        let label = Text::new(format!("{}", time))
            .set("font-size", format!("{}px", height / 2.0))
            .set("text-anchor", "middle")
            .set("x", x - gap)
            .set("y", 1.75 * height);

        let time_box = Document::new()
            .set("class", "time-box")
            .set("x", x)
            .set("y", y)
            .set("width", e.delta)
            .set("height", height)
            .add(rect)
            .add(text);

        let over_label = Text::new(format!("{}", e.delta))
            .set("lengthAdjust", e.delta)
            .set("x", "50%")
            .set("y", "40%")
            .set("font-size", format!("{}px", 0.75 * height))
            .set("text-anchor", "middle")
            .set("alignment-baseline", "middle");

        let label_group = Document::new()
            .set("class", "over-label")
            .set("x", x)
            .set("y", y - height)
            .set("width", e.delta)
            .set("height", 2.0 * height)
            .add(over_label);

        boxes_and_labels.push((time_box, label_group));
        labels.push(label);

        x += e.delta + gap;
        time += e.delta;
    }
    labels.push(
        Text::new(format!("{}", time))
            .set("font-size", format!("{}px", height / 2.0))
            .set("text-anchor", "middle")
            .set("x", x - gap)
            .set("y", 1.75 * height),
    );

    let total_width = x - gap + 0.25 * gapless_width; // remove last gap
    let mut document = Document::new().set("viewBox", (0, 0, 1.5 * total_width, 1.5 * height));
    document = document.add(Style::new(".over-label {\n\tfill-opacity: 0;\n}\n\n.time-box:hover + .over-label {\n\tfill-opacity: 1;\n}"));
    for (time_box, label) in boxes_and_labels {
        document = document.add(time_box).add(label);
    }
    for label in labels {
        document = document.add(label);
    }
    svg::save(filepath, &document)
}
