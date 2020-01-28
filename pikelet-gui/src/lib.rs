use druid::widget::{Align, Button, Flex, Label, Padding};
use druid::{
    AppDelegate, AppLauncher, Command, ContextMenu, Data, DelegateCtx, Env, Event, EventCtx,
    LocalizedString, MenuDesc, MenuItem, Selector, Widget, WindowDesc, WindowId,
};

mod data;

use log::info;

const MENU_COUNT_ACTION: Selector = Selector::new("menu-count-action");
const MENU_INCREMENT_ACTION: Selector = Selector::new("menu-increment-action");
const MENU_DECREMENT_ACTION: Selector = Selector::new("menu-decrement-action");

pub fn run() {
    let main_window = WindowDesc::new(ui_builder).menu(make_menu(&data::AppState::default()));
    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .launch(data::AppState::default())
        .expect("launch failed");
}

// this is just an experiment for how we might reduce boilerplate.
trait EventCtxExt {
    fn set_menu<T: 'static>(&mut self, menu: MenuDesc<T>);
}

impl EventCtxExt for EventCtx<'_, '_> {
    fn set_menu<T: 'static>(&mut self, menu: MenuDesc<T>) {
        let cmd = Command::new(druid::commands::SET_MENU, menu);
        self.submit_command(cmd, None);
    }
}

impl EventCtxExt for DelegateCtx<'_> {
    fn set_menu<T: 'static>(&mut self, menu: MenuDesc<T>) {
        let cmd = Command::new(druid::commands::SET_MENU, menu);
        self.submit_command(cmd, None);
    }
}

fn ui_builder() -> impl Widget<data::AppState> {
    let text = LocalizedString::new("hello-counter")
        .with_arg("count", |data: &data::AppState, _env| {
            data.menu_count.into()
        });
    let label = Label::new(text);
    let inc_button = Button::<data::AppState>::new("Add menu item", |ctx, data, _env| {
        data.menu_count += 1;
        ctx.set_menu(make_menu::<data::AppState>(data));
    });
    let dec_button = Button::<data::AppState>::new("Remove menu item", |ctx, data, _env| {
        data.menu_count = data.menu_count.saturating_sub(1);
        ctx.set_menu(make_menu::<data::AppState>(data));
    });

    let mut col = Flex::column();
    col.add_child(Align::centered(Padding::new(5.0, label)), 1.0);
    let mut row = Flex::row();
    row.add_child(Padding::new(5.0, inc_button), 1.0);
    row.add_child(Padding::new(5.0, dec_button), 1.0);
    col.add_child(row, 1.0);
    col
}

struct Delegate;

impl AppDelegate<data::AppState> for Delegate {
    fn event(
        &mut self,
        event: Event,
        data: &mut data::AppState,
        _env: &Env,
        ctx: &mut DelegateCtx,
    ) -> Option<Event> {
        match event {
            Event::Command(ref cmd) if cmd.selector == druid::commands::NEW_FILE => {
                let new_win = WindowDesc::new(ui_builder)
                    .menu(make_menu(data))
                    .window_size((data.selected as f64 * 100.0 + 300.0, 500.0));
                let command = Command::new(druid::commands::NEW_WINDOW, new_win);
                ctx.submit_command(command, None);
                None
            }
            Event::Command(ref cmd) if cmd.selector == MENU_COUNT_ACTION => {
                data.selected = *cmd.get_object().unwrap();
                ctx.set_menu(make_menu::<data::AppState>(data));
                None
            }
            // wouldn't it be nice if a menu (like a button) could just mutate state
            // directly if desired?
            Event::Command(ref cmd) if cmd.selector == MENU_INCREMENT_ACTION => {
                data.menu_count += 1;
                ctx.set_menu(make_menu::<data::AppState>(data));
                None
            }
            Event::Command(ref cmd) if cmd.selector == MENU_DECREMENT_ACTION => {
                data.menu_count = data.menu_count.saturating_sub(1);
                ctx.set_menu(make_menu::<data::AppState>(data));
                None
            }
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                let menu = ContextMenu::new(make_context_menu::<data::AppState>(), mouse.pos);
                let cmd = Command::new(druid::commands::SHOW_CONTEXT_MENU, menu);
                ctx.submit_command(cmd, None);
                None
            }
            other => Some(other),
        }
    }

    fn window_added(
        &mut self,
        id: WindowId,
        _data: &mut data::AppState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        info!("Window added, id: {:?}", id);
    }

    fn window_removed(
        &mut self,
        id: WindowId,
        _data: &mut data::AppState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        info!("Window removed, id: {:?}", id);
    }
}

#[allow(unused_assignments)]
fn make_menu<T: Data>(state: &data::AppState) -> MenuDesc<T> {
    let mut base = MenuDesc::empty();
    #[cfg(target_os = "macos")]
    {
        base = druid::platform_menus::mac::menu_bar();
    }
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        base = base.append(druid::platform_menus::win::file::default());
    }
    if state.menu_count != 0 {
        base = base.append(
            MenuDesc::new(LocalizedString::new("Custom")).append_iter(|| {
                (0..state.menu_count).map(|i| {
                    MenuItem::new(
                        LocalizedString::new("hello-counter")
                            .with_arg("count", move |_, _| i.into()),
                        Command::new(MENU_COUNT_ACTION, i),
                    )
                    .disabled_if(|| i % 3 == 0)
                    .selected_if(|| i == state.selected)
                })
            }),
        );
    }
    base
}

fn make_context_menu<T: Data>() -> MenuDesc<T> {
    MenuDesc::empty()
        .append(MenuItem::new(
            LocalizedString::new("Increment"),
            MENU_INCREMENT_ACTION,
        ))
        .append(MenuItem::new(
            LocalizedString::new("Decrement"),
            MENU_DECREMENT_ACTION,
        ))
}

// use iced::{Column, Container, Element, Row, Sandbox, Settings, Text};

// pub fn run() {
//     Workspace::run(Settings::default())
// }

// #[derive(Debug, Clone)]
// pub enum Message {}

// pub struct Workspace {
//     globals: pikelet::core::Globals,
// }

// impl Sandbox for Workspace {
//     type Message = Message;

//     fn new() -> Workspace {
//         Workspace {
//             globals: pikelet::core::Globals::default(),
//         }
//     }

//     fn title(&self) -> String {
//         "Pikelet".to_owned()
//     }

//     fn update(&mut self, message: Message) {
//         match message {}
//     }

//     fn view(&mut self) -> Element<Message> {
//         let Workspace { globals } = self;

//         Container::new(
//             Column::new()
//                 .push(Text::new("Hi this is Pikelet!"))
//                 .push(Text::new("Globals:"))
//                 .push(
//                     globals
//                         .entries()
//                         .fold(Column::new(), |column, (name, (r#type, term))| {
//                             column.push({
//                                 let entry = Row::new()
//                                     .push(Text::new(name))
//                                     .push(Text::new(" : "))
//                                     .push(view_term(r#type));

//                                 match term {
//                                     None => entry.push(Text::new("")).push(Text::new("")),
//                                     Some(term) => {
//                                         entry.push(Text::new(" = ")).push(view_term(term))
//                                     }
//                                 }
//                             })
//                         }),
//                 ),
//         )
//         .into()
//     }
// }

// fn view_term<M: 'static>(term: &pikelet::core::Term) -> Element<M> {
//     use pikelet::core::{Constant, Term, UniverseLevel, UniverseOffset};

//     match term {
//         Term::Universe(UniverseLevel(level)) => Row::new()
//             .push(Text::new(format!("Univ^{}", level))) // TODO: superscript?
//             .into(),
//         Term::Global(name) => Text::new(name).into(),
//         Term::Local(_) => Text::new("todo").into(),
//         Term::Constant(Constant::U8(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::U16(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::U32(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::U64(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::S8(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::S16(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::S32(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::S64(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::F32(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::F64(data)) => Text::new(data.to_string()).into(),
//         Term::Constant(Constant::Char(data)) => Text::new(format!("{:?}", data)).into(),
//         Term::Constant(Constant::String(data)) => Text::new(format!("{:?}", data)).into(),
//         Term::Sequence(_) => Text::new("todo").into(),
//         Term::Ann(term, r#type) => Row::new()
//             .push(view_term(term))
//             .push(Text::new(" : "))
//             .push(view_term(r#type))
//             .into(),
//         Term::RecordTerm(_) => Text::new("todo").into(),
//         Term::RecordType(_) => Text::new("todo").into(),
//         Term::RecordElim(_, _) => Text::new("todo").into(),
//         Term::FunctionType(_, _) => Text::new("todo").into(),
//         Term::FunctionTerm(_, _) => Text::new("todo").into(),
//         Term::FunctionElim(_, _) => Text::new("todo").into(),
//         Term::Lift(term, UniverseOffset(offset)) => Row::new()
//             .push(view_term(term))
//             .push(Text::new(format!("^{}", offset)))
//             .into(),
//         Term::Error => Text::new("ERROR!").into(),
//     }
// }
