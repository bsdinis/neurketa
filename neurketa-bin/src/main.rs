use neurketa::drawer::draw_time_series;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let time_series = [
        (3.4, 0, "first".to_string()),
        (1.2, 0, "second".to_string()),
        (4.2, 0, "last".to_string()),
    ]
    .into_iter()
    .collect();

    draw_time_series("image.svg", &time_series).context("failed to draw")
}
