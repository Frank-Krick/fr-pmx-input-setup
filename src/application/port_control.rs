use super::{AppListLine, Message, PortType, Theme};
use iced::color;
use iced::widget::{combo_box, combo_box::State, row, svg, text, Column, Container, Row};

pub struct InputConfigStateIndicatorSvgs {
    pub invalid: svg::Handle,
    pub valid: svg::Handle,
    pub valid_and_saved: svg::Handle,
}

pub fn port_path_combo_box<'a>(
    label_text: String,
    states: &'a State<String>,
    selected_path: &Option<String>,
    input_id: u32,
    message: fn(u32, String) -> Message,
) -> iced::widget::Row<'a, Message, Theme, iced::Renderer> {
    row![
        text(label_text.clone())
            .width(125)
            .height(35)
            .vertical_alignment(iced::alignment::Vertical::Center),
        combo_box(
            states,
            "Select port path",
            selected_path.as_ref(),
            move |path| { message(input_id, path.clone()) }
        )
        .width(500)
        .padding(5)
    ]
    .padding(5)
}

pub fn port_type_combo_box<'a>(
    port_name: String,
    port_types: &'a State<PortType>,
    selected_port_type: &Option<PortType>,
    input_id: u32,
    message: fn(u32, PortType) -> Message,
    indicator_svg: &svg::Handle,
) -> iced::widget::Row<'a, Message, Theme, iced::Renderer> {
    Row::<Message, Theme, iced::Renderer>::from_vec(vec![
        text(port_name.clone()).height(35).width(125).into(),
        combo_box(
            port_types,
            "Select input type",
            selected_port_type.as_ref(),
            move |selected_port_type| message(input_id, selected_port_type),
        )
        .width(250)
        .padding(5)
        .into(),
        svg(indicator_svg.clone())
            .width(35)
            .height(35)
            .style(iced::theme::Svg::custom_fn(|_theme| svg::Appearance {
                color: Some(color!(0xffffff)),
            }))
            .into(),
    ])
    .padding(5)
}

pub fn port_control<'a>(
    line: &AppListLine,
    port_types: &'a State<PortType>,
    port_paths: &'a State<String>,
    svg_indicators: &InputConfigStateIndicatorSvgs,
) -> Container<'a, Message, Theme, iced::Renderer> {
    let mut column = Column::new().push(port_type_combo_box(
        line.name.clone(),
        port_types,
        &line.selected_port_type,
        line.pmx_input_id,
        Message::PortTypeSelected,
        match (line.is_valid(), line.saved) {
            (true, true) => &svg_indicators.valid_and_saved,
            (true, false) => &svg_indicators.valid,
            (false, _) => &svg_indicators.invalid,
        },
    ));

    match &line.selected_port_type {
        Some(port_type) => match port_type {
            PortType::Mono => {
                column = column.push(port_path_combo_box(
                    String::from("Left port"),
                    port_paths,
                    &line.selected_left_out_port_path,
                    line.pmx_input_id,
                    Message::LeftPortSelected,
                ));
            }
            PortType::Stereo => {
                column = column.push(port_path_combo_box(
                    String::from("Left port"),
                    port_paths,
                    &line.selected_left_out_port_path,
                    line.pmx_input_id,
                    Message::LeftPortSelected,
                ));
                column = column.push(port_path_combo_box(
                    String::from("Right port"),
                    port_paths,
                    &line.selected_right_out_port_path,
                    line.pmx_input_id,
                    Message::RightPortSelected,
                ));
            }
            PortType::None => (),
        },
        None => (),
    }

    iced::widget::Container::new(column)
}
