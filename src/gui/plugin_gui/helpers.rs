use super::Message;
use nih_plug::prelude::*;
use nih_plug_iced::{widget, widgets as nih_widgets, Length};

pub(super) fn param_row<'a, P: Param>(
    label: &str,
    param: &'a P,
    state: &'a mut nih_widgets::param_slider::State,
) -> widget::Row<'a, Message> {
    use widget::*;
    Row::new()
        .push(Text::new(label).width(Length::Units(60)))
        .push(nih_widgets::ParamSlider::new(state, param).map(Message::ParamUpdate))
        .spacing(8)
        .padding(2)
}
