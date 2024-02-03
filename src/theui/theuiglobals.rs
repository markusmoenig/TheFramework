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

/// Opens a new dialog with a text edit widget.
pub fn open_text_dialog(
    window_title: &str,
    title: &str,
    text: &str,
    ui: &mut TheUI,
    ctx: &mut TheContext,
) {
    let width = 300;
    let height = 100;

    let mut canvas = TheCanvas::new();
    canvas.limiter_mut().set_max_size(vec2i(width, height));

    let mut text_layout: TheTextLayout = TheTextLayout::new(TheId::empty());
    text_layout.set_margin(vec4i(20, 20, 20, 20));

    text_layout.limiter_mut().set_max_width(width);
    let mut name_edit = TheTextLineEdit::new(TheId::named("Dialog Value"));
    name_edit.set_text(text.to_string());
    name_edit.limiter_mut().set_max_width(200);
    text_layout.add_pair(title.to_string(), Box::new(name_edit));

    canvas.set_layout(text_layout);
    ui.show_dialog(window_title, canvas, vec![TheDialogButtonRole::Accept], ctx);
}

/// Opens a new dialog with a deletion confirmation text.
pub fn open_delete_confirmation_dialog(
    window_title: &str,
    text: &str,
    uuid: Uuid,
    ui: &mut TheUI,
    ctx: &mut TheContext,
) {
    let width = 300;
    let height = 100;

    let mut canvas = TheCanvas::new();
    canvas.limiter_mut().set_max_size(vec2i(width, height));

    let mut hlayout: TheHLayout = TheHLayout::new(TheId::empty());
    // text_layout.set_margin(vec4i(20, 20, 20, 20));

    hlayout.limiter_mut().set_max_width(width);
    let mut text_widget = TheText::new(TheId::named_with_id("Dialog Value", uuid));
    text_widget.set_text(text.to_string());
    text_widget.limiter_mut().set_max_width(200);
    hlayout.add_widget(Box::new(text_widget));

    canvas.set_layout(hlayout);
    ui.show_dialog(
        window_title,
        canvas,
        vec![TheDialogButtonRole::Delete, TheDialogButtonRole::Reject],
        ctx,
    );
}
