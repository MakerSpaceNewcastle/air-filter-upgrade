use crate::{
    display::Color,
    fan::{FanCommand, FanSpeed},
    run_logic::{State, Trigger},
};
use core::{cell::RefCell, fmt::Write};
use defmt::debug;
use embedded_graphics::{
    prelude::{DrawTarget, Point, Primitive, Size, WebColors},
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
    Drawable,
};
use u8g2_fonts::U8g2TextStyle;

#[derive(Default)]
pub(crate) struct MainScreen {
    state: Option<State>,

    redraw_cmd: RefCell<bool>,
    redraw_time: RefCell<bool>,
}

impl MainScreen {
    pub(crate) fn update_state(&mut self, state: State) {
        match &self.state {
            Some(old_state) => {
                if old_state.fan_command() != state.fan_command() {
                    *self.redraw_cmd.borrow_mut() = true;
                }
                if old_state.time_remaining() != state.time_remaining() {
                    *self.redraw_time.borrow_mut() = true;
                }
            }
            None => {
                *self.redraw_cmd.borrow_mut() = true;
                *self.redraw_time.borrow_mut() = true;
            }
        }

        self.state = Some(state);
    }
}

impl Drawable for MainScreen {
    type Output = ();
    type Color = Color;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let display_box = target.bounding_box();

        let half_height = display_box.size.height / 2;
        let box_size = Size::new(display_box.size.width, half_height);

        let top = Rectangle::new(display_box.top_left, box_size);
        let bottom = Rectangle::new(
            display_box.top_left + Point::new(0, half_height as i32),
            box_size,
        );

        let box_style = PrimitiveStyleBuilder::new()
            .fill_color(Color::CSS_BLACK)
            .build();

        if let Ok(mut redraw) = self.redraw_cmd.try_borrow_mut() {
            if *redraw {
                debug!("Redrawing fan command");

                let fan_cmd = self
                    .state
                    .as_ref()
                    .expect("should have a state if the redraw flag was set")
                    .fan_command();

                top.into_styled(box_style).draw(target)?;

                // Display the running status
                Text::with_alignment(
                    match fan_cmd {
                        FanCommand::Stop => "Off",
                        FanCommand::Run(FanSpeed::Low) => "Low",
                        FanCommand::Run(FanSpeed::Medium) => "Mid",
                        FanCommand::Run(FanSpeed::High) => "High",
                    },
                    top.center() + Point::new(0, 53 / 2),
                    U8g2TextStyle::new(
                        u8g2_fonts::fonts::u8g2_font_inb53_mr,
                        match fan_cmd {
                            FanCommand::Stop => Color::CSS_GRAY,
                            FanCommand::Run(_) => Color::CSS_WHITE,
                        },
                    ),
                    Alignment::Center,
                )
                .draw(target)?;

                *redraw = false;
            }
        }

        if let Ok(mut redraw) = self.redraw_time.try_borrow_mut() {
            if *redraw {
                debug!("Redrawing time");

                let time_remaining = self
                    .state
                    .as_ref()
                    .expect("should have a state if the redraw flag was set")
                    .time_remaining();

                let time_str = match time_remaining {
                    Some(time_remaining) => {
                        let seconds = time_remaining.as_secs();
                        let minutes = seconds / 60;
                        let seconds = seconds % 60;

                        let mut time_str = heapless::String::<16>::new();
                        time_str
                            .write_fmt(format_args!("{:02}:{:02}", minutes, seconds))
                            .unwrap();

                        time_str
                    }
                    None => {
                        let mut time_str = heapless::String::new();
                        time_str.write_str("--:--").unwrap();

                        time_str
                    }
                };

                bottom.into_styled(box_style).draw(target)?;

                // Display the remaining run time
                Text::with_alignment(
                    &time_str,
                    bottom.center() + Point::new(0, 78 / 2),
                    U8g2TextStyle::new(
                        u8g2_fonts::fonts::u8g2_font_logisoso78_tn,
                        if time_remaining.is_some() {
                            Color::CSS_WHITE
                        } else {
                            Color::CSS_GRAY
                        },
                    ),
                    Alignment::Center,
                )
                .draw(target)?;

                *redraw = false;
            }
        }

        Ok(())
    }
}
