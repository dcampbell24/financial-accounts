use std::fmt::Display;

use plotters::{
    series::AreaSeries,
    style::{Color, FontTransform, IntoFont, ShapeStyle},
};
use rust_decimal::prelude::ToPrimitive;

use crate::app::message::Message;

use super::{account::transactions::Transactions, solarized, Duration};

pub struct Chart<T: Clone + Display> {
    pub txs: Transactions<T>,
    pub duration: Duration,
}

impl<T: Clone + Display> plotters_iced::Chart<Message> for Chart<T> {
    type State = ();

    fn build_chart<DB: plotters::prelude::DrawingBackend>(
        &self,
        _state: &Self::State,
        mut chart: plotters::prelude::ChartBuilder<DB>,
    ) {
        let txs = match self.duration {
            Duration::Week => self.txs.last_week(),
            Duration::Month => self.txs.last_month(),
            Duration::Year => self.txs.last_year(),
            Duration::All => self.txs.clone(),
        };

        if let (Some(Some(min_balance)), Some(Some(max_balance)), Some(min_date), Some(max_date)) = (
            txs.min_balance().map(|min| min.to_f64()),
            txs.max_balance().map(|max| max.to_f64()),
            txs.min_date(),
            txs.max_date(),
        ) {
            let mut chart = chart
                .x_label_area_size(28)
                .y_label_area_size(28)
                .margin(60)
                .build_cartesian_2d(min_date..max_date, min_balance..max_balance)
                .expect("failed to build chart");

            chart
                .configure_mesh()
                .bold_line_style(solarized::plot::base0())
                .light_line_style(solarized::plot::base1().mix(0.25))
                .axis_style(ShapeStyle::from(solarized::plot::base0()).stroke_width(1))
                .x_labels(10)
                .x_label_style(
                    ("sans-serif", 15)
                        .into_font()
                        .color(&solarized::plot::base0()),
                )
                .x_label_formatter(&|y| y.format("%Y-%m-%d %Z").to_string())
                .y_labels(10)
                .y_label_style(
                    ("sans-serif", 15)
                        .into_font()
                        .color(&solarized::plot::base0())
                        .transform(FontTransform::Rotate90),
                )
                .y_label_formatter(&thousands::Separable::separate_with_underscores)
                .draw()
                .expect("failed to draw chart mesh");

            chart
                .draw_series(
                    AreaSeries::new(
                        txs.txs
                            .iter()
                            .map(|tx| (tx.date, tx.balance.to_f64().unwrap())),
                        0.0,
                        solarized::plot::blue(),
                    )
                    .border_style(ShapeStyle::from(solarized::plot::blue()).stroke_width(2)),
                )
                .expect("failed to draw chart data");
        }
    }
}
