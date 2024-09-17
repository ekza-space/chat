#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "Login App",
        options,
        Box::new(|_cc| Ok(Box::new(LoginApp::default()))),
    )
}

struct LoginApp {
    username: String,
    password: String,
}

impl Default for LoginApp {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
        }
    }
}

impl eframe::App for LoginApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Login");

            ui.horizontal(|ui| {
                let username_label = ui.label("Username: ");
                ui.text_edit_singleline(&mut self.username)
                    .labelled_by(username_label.id);
            });

            ui.horizontal(|ui| {
                let password_label = ui.label("Password: ");
                ui.add(egui::TextEdit::singleline(&mut self.password).password(true))
                    .labelled_by(password_label.id);
            });

            if ui.button("Submit").clicked() {
                // TODO: login logic here
                println!("Username: {}, Password: {}", self.username, self.password);
            }
        });
    }
}
