use iui::prelude::*;
use iui::controls::{Label, Button, Entry, VerticalBox, Group};

pub fn window() {
    let ui = UI::init().expect("Couldn't initialize UI library");
    let mut win = Window::new(&ui, "Cobweb", 800, 800, WindowType::NoMenubar);
    
    let mut vbox = VerticalBox::new(&ui);
    vbox.set_padded(&ui, true);

    let mut group_vbox = VerticalBox::new(&ui);
    let mut group = Group::new(&ui, "Group");

    let mut quit_button = Button::new(&ui, "Quit");
    quit_button.on_clicked(&ui, {
        let ui = ui.clone();
        move |_| {
            ui.quit();
        }
    });

    let mut conn_button = Button::new(&ui, "Connect");

    let mut ip_text = String::new();
    ip_text.push_str("IP Address");
    let ip_label = Label::new(&ui, &ip_text);

    let ip_entry = Entry::new(&ui);

    let mut pass_text = String::new();
    pass_text.push_str("Password");
    let pass_label = Label::new(&ui, &pass_text);

    let pass_entry = Entry::new(&ui);

    group_vbox.append(&ui, ip_label, LayoutStrategy::Compact);
    group_vbox.append(&ui, ip_entry.clone(), LayoutStrategy::Compact);
    group_vbox.append(&ui, pass_label, LayoutStrategy::Compact);
    group_vbox.append(&ui, pass_entry.clone(), LayoutStrategy::Compact);
    group_vbox.append(&ui, conn_button.clone(), LayoutStrategy::Compact);
    group.set_child(&ui, group_vbox);
    vbox.append(&ui, group, LayoutStrategy::Compact);
    vbox.append(&ui, quit_button, LayoutStrategy::Compact);

    conn_button.on_clicked(&ui, {
        let ui = ui.clone();
        move |btn| {
            let pass = pass_entry.value(&ui);
            let ip = ip_entry.value(&ui);
            println!("{}\n{}", ip, pass);
            btn.set_text(&ui, "Connected");
        }
    });

    win.set_child(&ui, vbox);
    win.show(&ui);
    ui.main();
}
