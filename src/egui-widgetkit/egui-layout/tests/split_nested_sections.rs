use egui_layout::Split;

#[test]
fn test_nested_splits_rendering() {
    let mut called_sidebar = false;
    let mut called_header = false;
    let mut called_content = false;

    egui::__run_test_ctx(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            Split::horizontal()
                .spacing(8.0)
                .section(0.33, |ui| {
                    called_sidebar = true;
                    ui.label("Sidebar");
                })
                .section(0.67, |ui| {
                    Split::vertical()
                        .spacing(4.0)
                        .section(0.20, |ui| {
                            called_header = true;
                            ui.label("Header");
                        })
                        .section(0.80, |ui| {
                            called_content = true;
                            ui.label("Content");
                        })
                        .show(ui);
                })
                .show(ui);
        });
    });

    assert!(called_sidebar, "Sidebar section closure must be executed");
    assert!(called_header, "Header nested section closure must be executed");
    assert!(called_content, "Content nested section closure must be executed");
}

#[test]
fn test_empty_split_handling() {
    egui::__run_test_ctx(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let res = Split::horizontal().show(ui);
            assert_eq!(res.rect.size(), egui::Vec2::ZERO);
        });
    });
}
