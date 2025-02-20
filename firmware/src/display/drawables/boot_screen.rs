use crate::display::Color;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    prelude::{DrawTarget, Point, Primitive, WebColors},
    primitives::{Line, PrimitiveStyleBuilder, StrokeAlignment},
    text::{Alignment, Text},
    Drawable,
};

pub(crate) struct BootScreen {}

impl Drawable for BootScreen {
    type Output = ();
    type Color = Color;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let text_style = MonoTextStyle::new(&FONT_10X20, Color::CSS_BLACK);

        let line_style = PrimitiveStyleBuilder::new()
            .stroke_color(Color::CSS_YELLOW)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        let display_box = target.bounding_box();

        // Fill the display with pink pixels
        target.clear(Color::CSS_HOT_PINK)?;

        // Draw a one pixel border around the display
        display_box.into_styled(line_style).draw(target)?;

        // Draw a line through the display area
        Line::new(
            display_box
                .bottom_right()
                .expect("the display bounding box should be of size > 1x1"),
            display_box.top_left,
        )
        .into_styled(line_style)
        .draw(target)?;

        // Show some text
        Text::with_alignment(
            "Air Filter\nControl System",
            display_box.center(),
            text_style,
            Alignment::Center,
        )
        .draw(target)?;

        // Show the firmware version
        Text::with_alignment(
            env!("VERSION"),
            display_box.center() + Point::new(0, 75),
            text_style,
            Alignment::Center,
        )
        .draw(target)?;

        Ok(())
    }
}
