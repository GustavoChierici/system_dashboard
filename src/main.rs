use std::collections::VecDeque;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::widget::canvas::{Cache, Frame, Geometry};
use iced::widget::progress_bar;
use iced::{executor, time, Size};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{button, column, container, row, scrollable, text, pick_list, Container, Column, Text, Row, Space, Scrollable};
use iced::{
    Application, Color, Command, Element, Length, Settings, Subscription,
};
use iced_lazy::responsive;
use iced_native::{event, subscription, Event};
mod proc;

pub fn main() -> iced::Result {
    Grid::run(Settings::default())
}

pub struct Grid {
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Maximize(pane_grid::Pane),
    Restore,
    Close(pane_grid::Pane),
    CloseFocused,
    InfoSelected(SystemInfo),
    Tick,
    OpenTerminal,
}

impl Application for Grid {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (panes, _) = pane_grid::State::new(Pane::new());

        (
            Grid {
                panes,
                panes_created: 1,
                focus: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("System Dashboard")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Split(axis, pane) => {
                let result = self.panes.split(
                    axis,
                    &pane,
                    Pane::new(),
                );

                if let Some((pane, _)) = result {
                    self.focus = Some(pane);
                }

                self.panes_created += 1;
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self.panes.split(
                        axis,
                        &pane,
                        Pane::new(),
                    );

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) =
                        self.panes.adjacent(&pane, direction)
                    {
                        self.focus = Some(adjacent);
                    }
                }
            }
            Message::Clicked(pane) => {
                self.focus = Some(pane);
            }
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            }
            Message::Dragged(pane_grid::DragEvent::Dropped {
                pane,
                target,
            }) => {
                self.panes.swap(&pane, &target);
            }
            Message::Dragged(_) => {}
            Message::TogglePin(pane) => {
                if let Some(Pane { is_pinned, .. }) = self.panes.get_mut(&pane)
                {
                    *is_pinned = !*is_pinned;
                }
            }
            Message::Maximize(pane) => self.panes.maximize(&pane),
            Message::Restore => {
                self.panes.restore();
            }
            Message::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(&pane) {
                    self.focus = Some(sibling);
                }
            }
            Message::CloseFocused => {
                if let Some(pane) = self.focus {
                    if let Some(Pane { is_pinned, .. }) = self.panes.get(&pane)
                    {
                        if !is_pinned {
                            if let Some((_, sibling)) = self.panes.close(&pane)
                            {
                                self.focus = Some(sibling);
                            }
                        }
                    }
                }
            }
            Message::InfoSelected(info) => {
                if let Some(pane) = self.focus {
                    if let Some(Pane {selected_info, ..}) = self.panes.get_mut(&pane) {
                        *selected_info = info;
                    }
                }
            }
            Message::Tick => {
                for pane in self.panes.iter_mut() {
                    pane.1.cpu_chart.update();
                }
            }
            Message::OpenTerminal => {
                proc::open_terminal();
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            subscription::events_with(|event, status| {
                if let event::Status::Captured = status {
                    return None;
                }

                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        modifiers,
                        key_code,
                    }) if modifiers.command() => handle_hotkey(key_code),
                    _ => {
                        None
                    },
                }
            }),
            time::every(Duration::from_millis(1000)).map(|_| Message::Tick),
        ])
    }

    fn view(&self) -> Element<Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            let pin_button = button(
                text(if pane.is_pinned { "Unpin" } else { "Pin" }).size(14),
            )
            .on_press(Message::TogglePin(id))
            .padding(3);

            let title = row![
                pin_button,
                text(pane.selected_info).style(if is_focused {
                    PANE_ID_COLOR_FOCUSED
                } else {
                    PANE_ID_COLOR_UNFOCUSED
                }),
            ]
            .spacing(5);

            let title_bar = pane_grid::TitleBar::new(title)
                .controls(view_controls(
                    id,
                    total_panes,
                    pane.is_pinned,
                    is_maximized,
                    pane.selected_info
                ))
                .padding(10)
                .style(if is_focused {
                    style::title_bar_focused
                } else {
                    style::title_bar_active
                });

            pane_grid::Content::new(responsive(move |_| {
                view_content(&pane.cpu_chart, pane.selected_info)
            }))
            .title_bar(title_bar)
            .style(if is_focused {
                style::pane_focused
            } else {
                style::pane_active
            })
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .on_click(Message::Clicked)
        .on_drag(Message::Dragged)
        .on_resize(10, Message::Resized);

        container(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }
}

const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xC7 as f32 / 255.0,
);
const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(
    181.0,
    32.0,
    186.0,
);

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;
    use pane_grid::{Axis, Direction};

    let direction = match key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match key_code {
        KeyCode::V => Some(Message::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(Message::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(Message::CloseFocused),
        _ => direction.map(Message::FocusAdjacent),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemInfo {
    CPU,
    Mem,
    Processes,
    Uname,
}

impl SystemInfo {
    const ALL: [SystemInfo; 4] = [
        SystemInfo::CPU,
        SystemInfo::Mem,
        SystemInfo::Processes,
        SystemInfo::Uname,
    ];
}

impl Default for SystemInfo {
    fn default() -> SystemInfo {
        SystemInfo::Processes
    }
}

impl std::fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SystemInfo::CPU => "CPU info",
                SystemInfo::Mem => "Mem info",
                SystemInfo::Processes => "Processes monitor",
                SystemInfo::Uname => "System information"
            }
        )
    }
}

struct Pane {
    pub is_pinned: bool,
    pub selected_info: SystemInfo,
    pub cpu_chart: SystemChart,
}

impl Pane {
    fn new() -> Self {
        Self {
            is_pinned: false,
            selected_info: SystemInfo::default(),
            cpu_chart: SystemChart::default()
        }
    }
}

fn view_content<'a>(
    cpu_chart: &'a SystemChart,
    info: SystemInfo,
) -> Element<'a, Message> {
    let d = proc::get_meminfo();
    let mut data: VecDeque<&str> = d.split("\n").collect();
    data.pop_front();
    data.pop_back();
    let mut mem_total = 0.0;
    let mut mem_used = 0.0;
    let mut swap_total = 0.0;
    let mut swap_used = 0.0;
    // println!("{:#?}", data[1].split(" ").collect::<Vec<&str>>());

    mem_total = (data[0].split(" ").collect::<Vec<&str>>())[6].parse::<f32>().unwrap();
    mem_used = (data[0].split(" ").collect::<Vec<&str>>())[11].parse::<f32>().unwrap();
    swap_total = (data[1].split(" ").collect::<Vec<&str>>())[7].parse::<f32>().unwrap();
    swap_used = (data[1].split(" ").collect::<Vec<&str>>())[14].parse::<f32>().unwrap();
    let content_data = column![match info {
        SystemInfo::CPU => {
            column![
                // row![cpu_chart.view()],
                cpu_chart.view(),
            ]
        },
        SystemInfo::Mem => {
            column![
                text(format!("Mem: {:.2}GB; Used: {:.2}GB", mem_total/1048576.0, mem_used/1048576.0)),
                progress_bar(0.0..=mem_total, mem_used),
                text(format!("Swap: {:.2}GB; Used: {:.2}GB", swap_total/1048576.0, swap_used/1048576.0)),
                progress_bar(0.0..=swap_total, swap_used),
            ]
        },
        SystemInfo::Processes => {
            column![
                text(proc::get_monitor_info())
            ]
        },
        SystemInfo::Uname => {
            column![
                text(proc::get_uname())
            ]
        }
    }];

    let content = column![
        content_data,
    ]
    .width(Length::Fill)
    .spacing(10)
    .align_items(Alignment::Center);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .center_y()
        .into()
}

fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    is_maximized: bool,
    info: SystemInfo,
) -> Element<'a, Message> {
    let mut row = row![].spacing(5);

    if total_panes > 1 {
        let toggle = {
            let (content, message) = if is_maximized {
                ("Restore", Message::Restore)
            } else {
                ("Maximize", Message::Maximize(pane))
            };
            button(text(content).size(14))
                .style(theme::Button::Secondary)
                .padding(3)
                .on_press(message)
        };

        row = row.push(toggle);
    }

    let controls = column![
        pick_list(
            &SystemInfo::ALL[..],
            Some(info),
            Message::InfoSelected
        ),
    ]
    .max_width(200);

    row = row.push(controls);

    let hsplit = button(text("H+").size(14))
        .style(theme::Button::Secondary)
        .padding(3)
        .on_press(Message::Split(pane_grid::Axis::Horizontal, pane));

    row = row.push(hsplit);

    let vsplit = button(text("V+").size(14))
        .style(theme::Button::Secondary)
        .padding(3)
        .on_press(Message::Split(pane_grid::Axis::Vertical, pane));

    row = row.push(vsplit);


    let terminal = button(text("Open Terminal").size(14))
        .style(theme::Button::Secondary)
        .padding(3)
        .on_press(Message::OpenTerminal);

    row = row.push(terminal);

    let mut close = button(text("Close").size(14))
        .style(theme::Button::Destructive)
        .padding(3);

    if total_panes > 1 && !is_pinned {
        close = close.on_press(Message::Close(pane));
    }

    row.push(close).into()
}

mod style {
    use iced::widget::container;
    use iced::{Theme, Color};

    pub fn title_bar_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.background.strong.text),
            background: Some(palette.background.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.primary.strong.text),
            background: Some(Color::from_rgb(181.0/255.0, 32.0/255.0, 186.0/255.0).into()),
            ..Default::default()
        }
    }

    pub fn pane_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.background.strong.color,
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: Color::from_rgb(181.0/255.0, 32.0/255.0, 186.0/255.0),
            ..Default::default()
        }
    }
}

use plotters::prelude::ChartBuilder;
use plotters_backend::DrawingBackend;
use plotters_iced::{Chart, ChartWidget, plotters_backend};

struct SystemChart {
    last_sample_time: Instant,
    items_per_row: usize,
    processors: Vec<CPUChart>,
    chart_height: u16,
    last_idle: Vec<f64>,
    last_total: Vec<f64>,
}

impl Default for SystemChart {
    fn default() -> Self {
        Self {
            last_sample_time: Instant::now(),
            items_per_row: 1,
            processors: Default::default(),
            chart_height: 300,
            last_idle: Vec::new(),
            last_total: Vec::new(),
        }
    }
}

impl SystemChart {
    #[inline]
    fn is_initialized(&self) -> bool {
        !self.processors.is_empty()
    }

    #[inline]
    fn should_update(&self) -> bool {
        !self.is_initialized() || self.last_sample_time.elapsed() > Duration::from_millis(1000)
    }

    fn update(&mut self) {
        if !self.should_update() {
            return;
        }
        //eprintln!("refresh...");

        let cpu_usage = proc::get_cpuinfo(&mut self.last_idle, &mut self.last_total);
        self.last_sample_time = Instant::now();
        let now = Utc::now();
        let data = cpu_usage.iter().map(|v| *v as i32);

        //check if initialized
        if !self.is_initialized() {
            // eprintln!("init...");
            let mut processors: Vec<_> = data
                .map(|percent| CPUChart::new(vec![(now, percent)].into_iter()))
                .collect();
            self.processors.append(&mut processors);
        } else {
            //eprintln!("update...");
            for (percent, p) in data.zip(self.processors.iter_mut()) {
                p.push_data(now, percent);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if !self.is_initialized() {
            Text::new("Loading...")
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .into()
        } else {
            let mut col = Column::new().width(Length::Fill).height(Length::Fill);

            let chart_height = self.chart_height;
            let mut idx = 0;
            for chunk in self.processors.chunks(self.items_per_row) {
                let mut row = Row::new()
                    .spacing(15)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Units(chart_height))
                    .align_items(Alignment::Center);
                for item in chunk {
                    row = row.push(item.view(idx));
                    idx += 1;
                }
                while idx % self.items_per_row != 0 {
                    row = row.push(Space::new(Length::Fill, Length::Fill));
                    idx += 1;
                }
                col = col.push(row);
            }

            Scrollable::new(col).height(Length::Fill).into()
        }
    }
}

// #[derive(Debug, Clone, Copy)]
struct CPUChart {
    cache: Cache,
    data_points: VecDeque<(DateTime<Utc>, i32)>,
    limit: Duration,
}

impl CPUChart {
    pub fn new(data: impl Iterator<Item = (DateTime<Utc>, i32)>) -> Self {
        let data_points: VecDeque<_> = data.collect();
        Self {
            cache: Cache::new(),
            data_points,
            limit: Duration::from_secs(60 as u64),
        }
    }

    fn push_data(&mut self, time: DateTime<Utc>, value: i32) {
        let cur_ms = time.timestamp_millis();
        self.data_points.push_front((time, value));
        loop {
            if let Some((time, _)) = self.data_points.back() {
                let diff = Duration::from_millis((cur_ms - time.timestamp_millis()) as u64);
                if diff > self.limit {
                    self.data_points.pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();
    }

    fn view(&self, idx: usize) -> Element<Message> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(5)
                .push(Text::new(format!("CPU{}", idx)))
                .push(
                    ChartWidget::new(self).height(Length::Fill)
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}

impl Chart<Message> for CPUChart {
    type State = ();

    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut _builder: ChartBuilder<DB>) {
        use plotters::{prelude::*, style::Color};

        const PLOT_LINE_COLOR: RGBColor = RGBColor(181, 32, 186);

        // Acquire time range
        let newest_time = self
            .data_points
            .front()
            .unwrap_or(&(
                chrono::DateTime::from_utc(
                    chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    chrono::Utc,
                ),
                0,
            ))
            .0;
        let oldest_time = newest_time - chrono::Duration::seconds(60 as i64);
        let mut chart = _builder
            .x_label_area_size(0)
            .y_label_area_size(28)
            .margin(20)
            .build_cartesian_2d(oldest_time..newest_time, 0..100)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(plotters::style::colors::BLUE.mix(0.1))
            .light_line_style(plotters::style::colors::BLUE.mix(0.05))
            .axis_style(ShapeStyle::from(plotters::style::colors::BLUE.mix(0.45)).stroke_width(1))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&plotters::style::colors::BLUE.mix(0.65))
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y| format!("{}%", y))
            .draw()
            .expect("failed to draw chart mesh");

        chart
            .draw_series(
                AreaSeries::new(
                    self.data_points.iter().map(|x| (x.0, x.1 as i32)),
                    0,
                    PLOT_LINE_COLOR.mix(0.175),
                )
                .border_style(ShapeStyle::from(PLOT_LINE_COLOR).stroke_width(2)),
            )
            .expect("failed to draw chart data");
    }

    // fn draw_chart<DB: DrawingBackend>(&self, _state: &Self::State, root: DrawingArea<DB, Shift>) {
    //     let children = root.split_evenly((2, 2));
    //     for (i, area) in children.iter().enumerate() {
    //         let builder = ChartBuilder::on(area);
    //         draw_chart(builder, i + 1);
    //     }
    // }
}

// fn draw_chart<DB: DrawingBackend>(mut chart: ChartBuilder<DB>, power: usize) {
//     let mut chart = chart
//         .margin(30)
//         .caption(format!("y=x^{}", power), ("sans-serif", 22))
//         .x_label_area_size(30)
//         .y_label_area_size(30)
//         .build_cartesian_2d(-1f32..1f32, -1.2f32..1.2f32)
//         .unwrap();

//     chart
//         .configure_mesh()
//         .x_labels(3)
//         .y_labels(3)
//         .draw()
//         .unwrap();

//     chart
//         .draw_series(LineSeries::new(
//             (-50..=50)
//                 .map(|x| x as f32 / 50.0)
//                 .map(|x| (x, x.powf(power as f32))),
//             &RED,
//         ))
//         .unwrap();
// }
