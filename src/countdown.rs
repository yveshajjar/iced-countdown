use iced::window::{self, Id, maximize, *};
use iced::{Element, Subscription, Task, Vector};

mod custom_widgets;
mod dashboard;

pub struct App {
    main_window_id: Option<Id>,
    view: View,
    window_ratio: f32,
    window_size: Vector,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    // Start the app and open the main window
    AppStart(Id),
    WindowClosed(Id),
    WindowResized(Vector),

    DashboardMsg(dashboard::DashboardMessage),
}

#[derive(Debug)]
pub enum View {
    Dashboard(dashboard::Dashboard),
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        let (id, open) = window::open(window::Settings::default());
        let max = maximize(id, true);

        (
            Self {
                main_window_id: Some(id),
                view: View::Dashboard(dashboard::Dashboard::new()),
                window_ratio: 1.0,
                window_size: Vector::new(800.0, 600.0),
            },
            // Use batch to run both tasks
            Task::batch([open.map(AppMessage::AppStart), max]),
        )
    }

    pub fn title(&self, _window: iced::window::Id) -> String {
        "Focus".into()
    }

    fn update(&mut self, msg: AppMessage) -> Task<AppMessage> {
        app_message(self, msg)
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        use dashboard::Dashboard;

        let dashboard_subscriptions = Dashboard::subscriptions().map(AppMessage::DashboardMsg);

        let close_subscription = iced::window::close_events().map(AppMessage::WindowClosed);

        let window_resize_subscription =
            iced::window::resize_events().map(|(_, size)| AppMessage::WindowResized(size.into()));

        Subscription::batch([
            dashboard_subscriptions,
            close_subscription,
            window_resize_subscription,
        ])
    }

    fn view(&self, _window_id: iced::window::Id) -> Element<'_, AppMessage> {
        use View::*;

        match &self.view {
            Dashboard(dashboard) => dashboard.view().map(AppMessage::DashboardMsg),
        }
    }

    pub fn run() -> iced::Result {
        iced::daemon(Self::new, Self::update, Self::view)
            .title(Self::title)
            .subscription(Self::subscription)
            .antialiasing(true)
            .run()
    }
}

/// Helper function to handle app messages and keep the update function clean, using minimal nesting and early returns to improve readability
#[inline]
fn app_message(app: &mut App, msg: AppMessage) -> Task<AppMessage> {
    use AppMessage::*;

    match msg {
        AppStart(id) => {
            println!("Main window opened with ID: {id}");
            Task::none()
        }
        WindowClosed(id) => {
            // If the main window is closed, exit the application
            if app.main_window_id != Some(id) {
                println!("Window closed with ID: {id}");
                return close(id);
            }

            println!("Main window closed with ID: {id}, exiting application");
            iced::exit()
        }
        WindowResized(size) => {
            app.window_ratio = size.x / size.y;
            app.window_size = size;
            Task::none()
        }
        DashboardMsg(dashboard_msg) => {
            use dashboard::Action;

            let View::Dashboard(dashboard) = &mut app.view;

            match dashboard.update(dashboard_msg) {
                Action::None => Task::none(),
                Action::Run(task) => task.map(AppMessage::DashboardMsg),
            }
        }
    }
}
