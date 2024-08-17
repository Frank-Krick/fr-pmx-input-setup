use fr_pipewire_registry::ports::port_client::PortClient;
use fr_pipewire_registry::ports::{ListPort, ListPortsRequest, PortDirection};
use iced::application::Application;
use iced::widget::{svg, Scrollable};

use iced::widget::Column;
use iced::{Command, Element};
use pmx::input::{PmxInput, PmxInputType};
use pmx::pmx_registry_client::PmxRegistryClient;
use pmx::EmptyRequest;
use tonic::Request;

use crate::application::port_control::InputConfigStateIndicatorSvgs;

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

mod port_control;

#[derive(Clone, Debug)]
pub enum AppMessage {
    LoadInputsCompleted((Vec<PmxInput>, Vec<ListPort>)),
    PortTypeSelected(u32, PortType),
    LeftPortSelected(u32, String),
    RightPortSelected(u32, String),
    PortDataSaved(u32),
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
    saved: bool,
}

impl AppListLine {
    fn is_valid(&self) -> bool {
        match self.selected_port_type.clone() {
            Some(port_type) => match port_type {
                PortType::Mono => self.selected_left_out_port_path.is_some(),
                PortType::Stereo => {
                    self.selected_left_out_port_path.is_some()
                        && self.selected_right_out_port_path.is_some()
                }
                PortType::None => true,
            },
            None => false,
        }
    }
}

pub struct App {
    inputs: Vec<AppListLine>,
    port_types: iced::widget::combo_box::State<PortType>,
    pipewire_out_port_paths: iced::widget::combo_box::State<String>,
    pipewire_in_port_paths: iced::widget::combo_box::State<String>,
    svg_indicators: InputConfigStateIndicatorSvgs,
    pmx_registry_url: String,
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

impl App {
    fn read_svg(resource_name: &str) -> svg::Handle {
        svg::Handle::from_path(format!(
            "{}/resources/{}",
            env!("CARGO_MANIFEST_DIR"),
            resource_name
        ))
    }

    async fn save_none_port(id: u32, registry_url: String) {
        let mut client = PmxRegistryClient::connect(registry_url).await.unwrap();
        let request = Request::new(pmx::UpdateInputPortAssignmentsRequest {
            id,
            input_type: PmxInputType::None as i32,
            left_port_path: None,
            right_port_path: None,
        });
        client.update_input_port_assignments(request).await.unwrap();
    }

    async fn save_mono_port(id: u32, left: &str, registry_url: String) {
        let mut client = PmxRegistryClient::connect(registry_url).await.unwrap();
        let request = Request::new(pmx::UpdateInputPortAssignmentsRequest {
            id,
            input_type: PmxInputType::MonoInput as i32,
            left_port_path: Some(String::from(left)),
            right_port_path: None,
        });
        client.update_input_port_assignments(request).await.unwrap();
    }

    async fn save_stereo_input_ports(id: u32, left: &str, right: &str, registry_url: String) {
        let mut client = PmxRegistryClient::connect(registry_url).await.unwrap();
        let request = Request::new(pmx::UpdateInputPortAssignmentsRequest {
            id,
            input_type: PmxInputType::StereoInput as i32,
            left_port_path: Some(String::from(left)),
            right_port_path: Some(String::from(right)),
        });
        client.update_input_port_assignments(request).await.unwrap();
    }

    fn update_port_assignments_if_valid(&self, input_index: usize) -> Command<AppMessage> {
        if self.inputs[input_index].is_valid() {
            let input = self.inputs[input_index].clone();
            let url = self.pmx_registry_url.clone();
            match input.selected_port_type {
                Some(port_type) => match port_type {
                    PortType::Mono => {
                        let input_id = input.pmx_input_id;
                        let path = input.selected_left_out_port_path.unwrap().clone();
                        Command::perform(
                            async move {
                                App::save_mono_port(input_id, path.as_ref(), url).await;
                            },
                            move |_| Message::PortDataSaved(input_index as u32),
                        )
                    }
                    PortType::Stereo => {
                        let input_id = input.pmx_input_id;
                        let left_path = input.selected_left_out_port_path.unwrap().clone();
                        let right_path = input.selected_right_out_port_path.unwrap().clone();
                        Command::perform(
                            async move {
                                App::save_stereo_input_ports(
                                    input_id,
                                    left_path.as_ref(),
                                    right_path.as_ref(),
                                    url,
                                )
                                .await;
                            },
                            move |_| Message::PortDataSaved(input_index as u32),
                        )
                    }
                    PortType::None => {
                        let input_id = input.pmx_input_id;
                        Command::perform(
                            async move {
                                App::save_none_port(input_id, url).await;
                            },
                            move |_| Message::PortDataSaved(input_index as u32),
                        )
                    }
                },
                None => Command::none(),
            }
        } else {
            Command::none()
        }
    }
}

impl Application for App {
    type Executor = Executor;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
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
                svg_indicators: InputConfigStateIndicatorSvgs {
                    invalid: App::read_svg("x-square.svg"),
                    valid: App::read_svg("check2-square.svg"),
                    valid_and_saved: App::read_svg("save2.svg"),
                },
                pmx_registry_url: flags.pmx_registry_url.clone(),
            },
            iced::Command::perform(
                async move {
                    let mut client = PmxRegistryClient::connect(flags.pmx_registry_url)
                        .await
                        .unwrap();
                    let request = Request::new(EmptyRequest {});
                    let inputs_response = client.list_inputs(request).await.unwrap();

                    let mut client = PortClient::connect(flags.port_registry_url).await.unwrap();
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
                        saved: true,
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
                self.inputs[changed_input_index].saved = false;
                self.update_port_assignments_if_valid(changed_input_index)
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
                self.inputs[changed_input_index].saved = false;
                self.update_port_assignments_if_valid(changed_input_index)
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
                self.inputs[changed_input_index].saved = false;
                self.update_port_assignments_if_valid(changed_input_index)
            }
            AppMessage::PortDataSaved(id) => {
                let changed_input_index = self
                    .inputs
                    .iter()
                    .enumerate()
                    .find(|(_index, input)| input.pmx_input_id == id)
                    .unwrap()
                    .0;
                self.inputs[changed_input_index].saved = true;
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let input_rows = self.inputs.clone();

        let elements = input_rows
            .into_iter()
            .map(|i| {
                port_control::port_control(
                    &i,
                    &self.port_types,
                    &self.pipewire_out_port_paths,
                    &self.svg_indicators,
                )
                .into()
            })
            .collect::<Vec<Element<Self::Message, Self::Theme, iced::Renderer>>>();
        Scrollable::new(Column::from_vec(elements).padding(5).spacing(10)).into()
    }
}
