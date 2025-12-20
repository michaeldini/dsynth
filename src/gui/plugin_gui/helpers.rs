use super::Message;
use nih_plug::prelude::*;
use nih_plug_iced::{Length, widget, widgets as nih_widgets};

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

pub(super) fn param_checkbox<'a>(label: &str, param: &'a BoolParam) -> widget::Row<'a, Message> {
    use widget::*;
    let is_checked = param.value();
    let param_ptr = param.as_ptr();

    Row::new()
        .push(Text::new(label).width(Length::Units(60)))
        .push(Checkbox::new(is_checked, "", move |new_value| {
            let normalized = if new_value { 1.0 } else { 0.0 };
            Message::ParamUpdate(nih_widgets::ParamMessage::SetParameterNormalized(
                param_ptr, normalized,
            ))
        }))
        .spacing(8)
        .padding(2)
}
