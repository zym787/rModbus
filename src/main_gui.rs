use rustrrrrrrr::gui::ModbusApp;
use std::sync::Arc;

fn main() {
    let options = eframe::NativeOptions::default();
    
    eframe::run_native(
        "Modbus RTU上位机",
        options,
        Box::new(|cc| {
            // 配置字体，添加对中文字体的支持
            let mut fonts = eframe::egui::FontDefinitions::default();
            
            // 添加中文字体
            fonts.font_data.insert(
                "simhei".to_string(),
                Arc::new(eframe::egui::FontData::from_static(include_bytes!("C:\\Windows\\Fonts\\simhei.ttf"))),
            );
            fonts.font_data.insert(
                "simsun".to_string(),
                Arc::new(eframe::egui::FontData::from_static(include_bytes!("C:\\Windows\\Fonts\\simsun.ttc"))),
            );
            
            // 将中文字体添加到默认字体家族中
            fonts.families.get_mut(&eframe::egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "simhei".to_string());
            fonts.families.get_mut(&eframe::egui::FontFamily::Proportional)
                .unwrap()
                .insert(1, "simsun".to_string());
            
            cc.egui_ctx.set_fonts(fonts);
            
            Ok(Box::<ModbusApp>::default())
        }),
    ).unwrap();
}
