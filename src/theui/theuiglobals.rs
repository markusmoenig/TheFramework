use crate::prelude::*;

/// Create widgets for Int2 value and add them to the layout.
pub fn create_int2_widgets(
    layout: &mut dyn TheHLayoutTrait,
    redirect_as: TheId,
    value: Vec2i,
    labels: Vec<&str>,
) {
    layout.set_redirect_as(redirect_as);

    let mut text = TheText::new(TheId::empty());
    text.set_text(labels[0].to_string());
    let mut name_edit = TheTextLineEdit::new(TheId::named("Int2 X"));
    name_edit.set_range(TheValue::RangeI32(core::ops::RangeInclusive::new(
        std::i32::MIN,
        std::i32::MAX,
    )));
    name_edit.limiter_mut().set_max_width(100);
    name_edit.set_text(value.x.to_string());
    name_edit.set_associated_layout(layout.id().clone());

    layout.add_widget(Box::new(text));
    layout.add_widget(Box::new(name_edit));

    let mut text = TheText::new(TheId::empty());
    text.set_text(labels[1].to_string());
    let mut name_edit = TheTextLineEdit::new(TheId::named("Int2 Y"));
    name_edit.set_range(TheValue::RangeI32(core::ops::RangeInclusive::new(
        std::i32::MIN,
        std::i32::MAX,
    )));
    name_edit.limiter_mut().set_max_width(100);
    name_edit.set_text(value.y.to_string());
    name_edit.set_associated_layout(layout.id().clone());

    layout.add_widget(Box::new(text));
    layout.add_widget(Box::new(name_edit));
}

/// Create widgets for Float2 value and add them to the layout.
pub fn create_float2_widgets(
    layout: &mut dyn TheHLayoutTrait,
    redirect_as: TheId,
    value: Vec2f,
    labels: Vec<&str>,
) {
    layout.set_redirect_as(redirect_as);

    let mut text = TheText::new(TheId::empty());
    text.set_text(labels[0].to_string());
    let mut name_edit = TheTextLineEdit::new(TheId::named("Float2 X"));
    name_edit.set_range(TheValue::RangeF32(core::ops::RangeInclusive::new(
        std::f32::MIN,
        std::f32::MAX,
    )));
    name_edit.limiter_mut().set_max_width(100);
    name_edit.set_text(value.x.to_string());
    name_edit.set_associated_layout(layout.id().clone());

    layout.add_widget(Box::new(text));
    layout.add_widget(Box::new(name_edit));

    let mut text = TheText::new(TheId::empty());
    text.set_text(labels[1].to_string());
    let mut name_edit = TheTextLineEdit::new(TheId::named("Float2 Y"));
    name_edit.set_range(TheValue::RangeF32(core::ops::RangeInclusive::new(
        std::f32::MIN,
        std::f32::MAX,
    )));
    name_edit.limiter_mut().set_max_width(100);
    name_edit.set_text(value.y.to_string());
    name_edit.set_associated_layout(layout.id().clone());

    layout.add_widget(Box::new(text));
    layout.add_widget(Box::new(name_edit));
}
