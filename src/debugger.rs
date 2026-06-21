use crate::cpu::{FlagRegister, CPU};

pub fn show(ctx: &egui::Context, cpu: &CPU) -> bool {
    let mut close_requested = false;
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("debugger"),
        egui::ViewportBuilder::default()
            .with_title("CPU Registers")
            .with_inner_size([260.0, 300.0])
            .with_position([0.0, 0.0]),
        |child_ctx, _class| {
            egui::CentralPanel::default().show_inside(child_ctx, |ui| {
                draw_registers(ui, cpu);
            });
            if child_ctx.input(|i| i.viewport().close_requested()) {
                close_requested = true;
            }
        },
    );
    close_requested
}

fn draw_registers(ui: &mut egui::Ui, cpu: &CPU) {
    egui::Grid::new("cpu-registers").striped(true).show(ui, |ui| {
        ui.label("AF");
        ui.label(egui::RichText::new(format!("{:04X}", cpu.get_af())).monospace());
        ui.label("BC");
        ui.label(egui::RichText::new(format!("{:04X}", cpu.get_bc())).monospace());
        ui.end_row();

        ui.label("DE");
        ui.label(egui::RichText::new(format!("{:04X}", cpu.get_de())).monospace());
        ui.label("HL");
        ui.label(egui::RichText::new(format!("{:04X}", cpu.get_hl())).monospace());
        ui.end_row();

        ui.label("PC");
        ui.label(egui::RichText::new(format!("{:04X}", cpu.pc)).monospace());
        ui.label("SP");
        ui.label(egui::RichText::new(format!("{:04X}", cpu.sp)).monospace());
        ui.end_row();
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Flags:");
        ui.label(if cpu.get_flag(FlagRegister::Zero) != 0 { "Z" } else { "-" });
        ui.label(if cpu.get_flag(FlagRegister::Sub) != 0 { "N" } else { "-" });
        ui.label(if cpu.get_flag(FlagRegister::HalfCarry) != 0 { "H" } else { "-" });
        ui.label(if cpu.get_flag(FlagRegister::Carry) != 0 { "C" } else { "-" });
    });

    ui.separator();

    ui.label(format!("IME: {}", cpu.ime));
    ui.label(format!("Halted: {}", cpu.halted));
    ui.label(format!("Stopped: {}", cpu.stopped));
}
