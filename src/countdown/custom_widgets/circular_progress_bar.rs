use std::f32::consts::PI;

use iced::{Renderer as IcedRenderer, Vector};

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Widget, tree};
use iced::mouse;
use iced::widget::canvas;
use iced::{Color, Element, Length, Radians, Rectangle, Size};

pub struct CircularProgressBar {
    pub circular_size: f32,
    pub circular_color: Color,
    pub bar_height: f32,
    pub track_color: Color,
    pub progress: f32, // 0.0 to 1.0
}

#[derive(Debug, Default)]
pub struct State {
    cache: canvas::Cache<IcedRenderer>,
}

impl<Message, Theme> Widget<Message, Theme, IcedRenderer> for CircularProgressBar {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.circular_size),
            height: Length::Fixed(self.circular_size),
        }
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        _renderer: &IcedRenderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();

        state.cache.clear();

        layout::atomic(limits, self.circular_size, self.circular_size)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut IcedRenderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();

        let bounds = layout.bounds();

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width().min(frame.height()) / 2.0 - self.bar_height / 2.0;

            let track_path = canvas::Path::circle(frame.center(), track_radius);

            // background circle
            frame.stroke(
                &track_path,
                canvas::Stroke::default()
                    .with_color(self.track_color)
                    .with_width(self.bar_height),
            );

            // progress arc
            let progress = self.progress.clamp(0.0, 1.0);

            if progress > 0.0 {
                let start_angle = Radians(-PI / 2.0);
                let end_angle = Radians(-PI / 2.0 + 2.0 * PI * progress);

                let mut builder = canvas::path::Builder::new();

                builder.arc(canvas::path::Arc {
                    center: frame.center(),
                    radius: track_radius,
                    start_angle,
                    end_angle,
                });

                let bar_path = builder.build();

                frame.stroke(
                    &bar_path,
                    canvas::Stroke::default()
                        .with_color(self.circular_color)
                        .with_width(self.bar_height)
                        .with_line_cap(canvas::LineCap::Round),
                );
            }
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            use iced::advanced::graphics::geometry::Renderer as _;

            renderer.draw_geometry(geometry);
        });
    }
}

impl<'a, Message, Theme> From<CircularProgressBar> for Element<'a, Message, Theme, IcedRenderer> {
    fn from(text: CircularProgressBar) -> Element<'a, Message, Theme, IcedRenderer> {
        Element::new(text)
    }
}
