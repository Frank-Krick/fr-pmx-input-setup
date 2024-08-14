use fr_pipewire_registry::ports::port_client::PortClient;
use fr_pipewire_registry::ports::{ListPort, ListPortsRequest, PortDirection};
use iced::application::Application;

use iced::widget::{column, combo_box, row, text, Column, Row};
use iced::{Command, Element};
use pmx::input::{PmxInput, PmxInputType};
use pmx::pmx_registry_client::PmxRegistryClient;
use pmx::EmptyRequest;
use tonic::Request;

pub mod pmx {
    tonic::include_proto!("pmx");

    pub mod input {
        tonic::include_proto!("pmx.input");
    }
}

pub mod fr_pipewire_registry {
    pub mod ports {
        tonic::include_proto!("fr_pipewire_registry.ports");
    }
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    LoadInputsCompleted((Vec<PmxInput>, Vec<ListPort>)),
    PortTypeSelected(u32, PortType),
    LeftPortSelected(u32, String),
    RightPortSelected(u32, String),
}

#[derive(Clone, Debug)]
pub enum PortType {
    Mono,
    Stereo,
    None,
}

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortType::Mono => f.write_str("Mono"),
            PortType::Stereo => f.write_str("Stereo"),
            PortType::None => f.write_str("None"),
        }
    }
}

#[derive(Clone)]
struct AppListLine {
    pmx_input_id: u32,
    name: String,
    selected_port_type: Option<PortType>,
    selected_left_out_port_path: Option<String>,
    selected_right_out_port_path: Option<String>,
}

pub struct App {
    inputs: Vec<AppListLine>,
    port_types: iced::widget::combo_box::State<PortType>,
    pipewire_out_port_paths: iced::widget::combo_box::State<String>,
    pipewire_in_port_paths: iced::widget::combo_box::State<String>,
}

#[derive(Default, Clone)]
pub struct AppFlags {
    pub port_registry_url: String,
    pub pmx_registry_url: String,
}

type Executor = iced::executor::Default;
type Message = AppMessage;
type Theme = iced::Theme;
type Flags = AppFlags;

impl Application for App {
    type Executor = Executor;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            App {
                inputs: Vec::new(),
                port_types: iced::widget::combo_box::State::new(vec![
                    PortType::Mono,
                    PortType::Stereo,
                    PortType::None,
                ]),
                pipewire_out_port_paths: iced::widget::combo_box::State::new(Vec::new()),
                pipewire_in_port_paths: iced::widget::combo_box::State::new(Vec::new()),
            },
            iced::Command::perform(
                async move {
                    let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001")
                        .await
                        .unwrap();
                    let request = Request::new(EmptyRequest {});
                    let inputs_response = client.list_inputs(request).await.unwrap();

                    let mut client = PortClient::connect("http://127.0.0.1:50000").await.unwrap();
                    let request = Request::new(ListPortsRequest {});
                    let ports_respose = client.list_ports(request).await.unwrap();
                    (
                        inputs_response.get_ref().inputs.clone(),
                        ports_respose.get_ref().ports.clone(),
                    )
                },
                Self::Message::LoadInputsCompleted,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Input Setup")
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::GruvboxDark
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::LoadInputsCompleted(inputs) => {
                self.inputs = inputs
                    .0
                    .iter()
                    .map(|i| AppListLine {
                        pmx_input_id: i.id,
                        name: i.name.clone(),
                        selected_port_type: match i.input_type {
                            x if x == PmxInputType::StereoInput as i32 => Some(PortType::Stereo),
                            x if x == PmxInputType::MonoInput as i32 => Some(PortType::Mono),
                            _ => Some(PortType::None),
                        },
                        selected_left_out_port_path: i.left_port_path.clone(),
                        selected_right_out_port_path: i.right_port_path.clone(),
                    })
                    .collect();
                let in_port_paths = inputs
                    .1
                    .iter()
                    .filter(|p| p.direction == PortDirection::In as i32)
                    .map(|p| p.path.clone())
                    .collect();

                self.pipewire_in_port_paths = iced::widget::combo_box::State::new(in_port_paths);

                let out_port_paths = inputs
                    .1
                    .iter()
                    .filter(|p| p.direction == PortDirection::Out as i32)
                    .map(|p| p.path.clone())
                    .collect();

                self.pipewire_out_port_paths = iced::widget::combo_box::State::new(out_port_paths);
                Command::none()
            }
            AppMessage::PortTypeSelected(id, port_type) => {
                let changed_input_index = self
                    .inputs
                    .iter()
                    .enumerate()
                    .find(|(_index, input)| input.pmx_input_id == id)
                    .unwrap()
                    .0;
                self.inputs[changed_input_index].selected_port_type = Some(port_type);
                Command::none()
            }
            AppMessage::LeftPortSelected(id, path) => {
                let changed_input_index = self
                    .inputs
                    .iter()
                    .enumerate()
                    .find(|(_index, input)| input.pmx_input_id == id)
                    .unwrap()
                    .0;
                self.inputs[changed_input_index].selected_left_out_port_path = Some(path.clone());
                Command::none()
            }
            AppMessage::RightPortSelected(id, path) => {
                let changed_input_index = self
                    .inputs
                    .iter()
                    .enumerate()
                    .find(|(_index, input)| input.pmx_input_id == id)
                    .unwrap()
                    .0;
                self.inputs[changed_input_index].selected_right_out_port_path = Some(path.clone());
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let input_rows = self.inputs.clone();

        let elements = input_rows
            .into_iter()
            .map(|i| {
                iced::widget::Container::new(row![column![
                    Row::<Self::Message, Self::Theme, iced::Renderer>::from_vec(vec![
                        text(i.name.clone()).height(35).width(125).into(),
                        combo_box(
                            &self.port_types,
                            "Select input type",
                            i.selected_port_type.as_ref(),
                            move |selected_port_type| {
                                Self::Message::PortTypeSelected(i.pmx_input_id, selected_port_type)
                            },
                        )
                        .width(250)
                        .padding(5)
                        .into(),
                    ])
                    .padding(5),
                    row![
                        text(String::from("Left Port"))
                            .width(125)
                            .height(35)
                            .vertical_alignment(iced::alignment::Vertical::Center),
                        combo_box(
                            &self.pipewire_out_port_paths,
                            "Select port path",
                            i.selected_left_out_port_path.as_ref(),
                            move |path| {
                                Self::Message::LeftPortSelected(i.pmx_input_id, path.clone())
                            }
                        )
                        .width(500)
                        .padding(5)
                    ]
                    .padding(5),
                    row![
                        text(String::from("Right Port"))
                            .width(125)
                            .height(35)
                            .vertical_alignment(iced::alignment::Vertical::Center),
                        combo_box(
                            &self.pipewire_out_port_paths,
                            "Select port path",
                            i.selected_right_out_port_path.as_ref(),
                            move |path| {
                                Self::Message::RightPortSelected(i.pmx_input_id, path.clone())
                            }
                        )
                        .width(500)
                        .padding(5)
                    ]
                    .padding(5)
                ]])
                .style(iced::widget::container::Appearance {
                    text_color: Some(self.theme().extended_palette().background.strong.text),
                    background: Some(self.theme().extended_palette().background.base.color.into()),
                    border: iced::Border {
                        color: self.theme().extended_palette().background.strong.color,
                        width: 1.0,
                        radius: 1.into(),
                    },
                    shadow: iced::Shadow::default(),
                })
                .into()
            })
            .collect::<Vec<Element<Self::Message, Self::Theme, iced::Renderer>>>();
        Column::from_vec(elements).padding(5).spacing(10).into()
    }
}
