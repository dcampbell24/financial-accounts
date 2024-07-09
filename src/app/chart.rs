use std::rc::Rc;

use plotters::{
    series::AreaSeries,
    style::{Color, FontTransform, IntoFont, ShapeStyle},
};
use plotters_iced::Chart;
use rust_decimal::prelude::ToPrimitive;
use thousands::Separable;

use crate::app::message::Message;

use super::{account::transactions::Transactions, solarized};

pub struct MyChart {
    pub txs: Rc<dyn Transactions>,
}

impl Chart<Message> for MyChart {
    type State = ();

    fn build_chart<DB: plotters::prelude::DrawingBackend>(
        &self,
        _state: &Self::State,
        mut chart: plotters::prelude::ChartBuilder<DB>,
    ) {
        if let (Some(Some(min_balance)), Some(Some(max_balance)), Some(min_date), Some(max_date)) = (
            self.txs.min_balance().map(|min| min.to_f64()),
            self.txs.max_balance().map(|max| max.to_f64()),
            self.txs.min_date(),
            self.txs.max_date(),
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
                .y_label_formatter(&|y| y.separate_with_underscores())
                .draw()
                .expect("failed to draw chart mesh");

            chart
                .draw_series(
                    AreaSeries::new(
                        self.txs
                            .transactions()
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
