use eframe::egui;
use crate::ModbusSerialClient;

#[derive(Default)]
pub struct ModbusApp {
    // 串口设置
    port: String,
    baud_rate: u32,
    slave_id: u8,
    available_ports: Vec<String>,
    
    // 寄存器设置
    start_address: u16,
    register_count: u16,
    
    // 控制状态
    connected: bool,
    client: Option<ModbusSerialClient>,
    registers: Vec<u16>,
    error_message: String,
    
    // 显示样式
    display_style: DisplayStyle,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
enum DisplayStyle {
    #[default]
    Decimal,
    Hexadecimal,
    Binary,
}

impl ModbusApp {
    pub fn new() -> Self {
        let mut app = Self::default();
        app.port = "COM4".to_string();
        app.baud_rate = 9600;
        app.slave_id = 1;
        app.start_address = 0;
        app.register_count = 100;
        app.available_ports = crate::list_available_ports().unwrap_or_default().into_iter().map(|p| p.port_name).collect();
        app
    }
    
    fn connect(&mut self) {
        match ModbusSerialClient::new(&self.port, self.baud_rate) {
            Ok(client) => {
                self.client = Some(client);
                self.connected = true;
                self.error_message = "".to_string();
            }
            Err(e) => {
                self.error_message = format!("Failed to connect: {:?}", e);
                self.connected = false;
            }
        }
    }
    
    fn read_registers(&mut self) {
        if let Some(client) = &mut self.client {
            match client.read_holding_registers(self.slave_id, self.start_address, self.register_count) {
                Ok(registers) => {
                    self.registers = registers;
                    self.error_message = "".to_string();
                }
                Err(e) => {
                    self.error_message = format!("Failed to read registers: {:?}", e);
                }
            }
        } else {
            self.error_message = "Not connected".to_string();
        }
    }
    
    fn disconnect(&mut self) {
        self.client = None;
        self.connected = false;
    }
}



impl eframe::App for ModbusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 设置字体大小
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(18.0, egui::FontFamily::Proportional),
            );
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            );
            
            ui.heading("Modbus RTU上位机");
            ui.separator();
            
            // 使用垂直布局
            egui::ScrollArea::vertical().show(ui, |ui| {
                // 第一行：串口设置和控制集成在一起
                ui.horizontal(|ui| {
                    // 串口设置
                    ui.group(|ui| {
                        ui.heading("串口设置");
                        ui.add_space(10.0);
                        
                        // 使用网格布局
                        let grid = egui::Grid::new(egui::Id::new("serial_settings"))
                            .num_columns(2)
                            .spacing([20.0, 10.0]);
                        
                        grid.show(ui, |ui| {
                            ui.label("端口:");
                            egui::ComboBox::from_id_source(egui::Id::new("port_combo"))
                                .selected_text(&self.port)
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    // 确保available_ports不为空
                                    if self.available_ports.is_empty() {
                                        self.available_ports = crate::list_available_ports().unwrap_or_default().into_iter().map(|p| p.port_name).collect();
                                    }
                                    
                                    for port in &self.available_ports {
                                        ui.selectable_value(&mut self.port, port.clone(), port);
                                    }
                                });
                            ui.end_row();
                            
                            ui.label("波特率:");
                            egui::ComboBox::from_id_source(egui::Id::new("baud_rate_combo"))
                                .selected_text(&self.baud_rate.to_string())
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    let baud_rates = vec![1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200];
                                    for &rate in baud_rates.iter() {
                                        ui.selectable_value(&mut self.baud_rate, rate, &rate.to_string());
                                    }
                                });
                            ui.end_row();
                            
                            ui.label("从机ID:");
                            ui.add(egui::DragValue::new(&mut self.slave_id)
                                .range(1..=255));
                            ui.end_row();
                        });
                    });
                    
                    ui.add_space(20.0);
                    
                    // 控制按钮
                    ui.group(|ui| {
                        ui.heading("控制");
                        ui.add_space(10.0);
                        
                        ui.vertical(|ui| {
                            if !self.connected {
                                if ui.button(egui::RichText::new("连接").size(14.0)).clicked() {
                                    self.connect();
                                }
                            } else {
                                if ui.button(egui::RichText::new("断开").size(14.0)).clicked() {
                                    self.disconnect();
                                }
                                ui.add_space(10.0);
                                if ui.button(egui::RichText::new("读取寄存器").size(14.0)).clicked() {
                                    self.read_registers();
                                }
                            }
                        });
                    });
                });
                
                ui.add_space(20.0);
                
                // 第二行：Modbus相关的寄存器设置放在一起
                ui.group(|ui| {
                    ui.heading("Modbus设置");
                    ui.add_space(10.0);
                    
                    egui::Grid::new(egui::Id::new("modbus_settings"))
                        .num_columns(2)
                        .spacing([40.0, 10.0])
                        .show(ui, |ui| {
                            // 寄存器设置
                            ui.vertical(|ui| {
                                ui.heading("寄存器设置");
                                ui.add_space(5.0);
                                
                                let grid = egui::Grid::new(egui::Id::new("register_settings"))
                                    .num_columns(2)
                                    .spacing([20.0, 10.0]);
                                
                                grid.show(ui, |ui| {
                                    ui.label("起始地址:");
                                    ui.add(egui::DragValue::new(&mut self.start_address)
                                        .range(0..=65535));
                                    ui.end_row();
                                    
                                    ui.label("寄存器数量:");
                                    ui.add(egui::DragValue::new(&mut self.register_count)
                                        .range(1..=125));
                                    ui.end_row();
                                });
                            });
                            
                            // 显示样式
                            ui.vertical(|ui| {
                                ui.heading("显示样式");
                                ui.add_space(5.0);
                                
                                ui.vertical(|ui| {
                                    ui.selectable_value(&mut self.display_style, DisplayStyle::Decimal, "十进制");
                                    ui.selectable_value(&mut self.display_style, DisplayStyle::Hexadecimal, "十六进制");
                                    ui.selectable_value(&mut self.display_style, DisplayStyle::Binary, "二进制");
                                });
                            });
                        });
                });
                
                ui.add_space(20.0);
                
                // 错误信息
                if !self.error_message.is_empty() {
                    ui.group(|ui| {
                        ui.heading("错误信息");
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(&self.error_message).color(egui::Color32::RED));
                    });
                    ui.add_space(20.0);
                }
                
                // 寄存器数据显示
                ui.group(|ui| {
                    ui.heading("寄存器数据");
                    ui.add_space(10.0);
                    
                    if !self.registers.is_empty() {
                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .show(ui, |ui| {
                                // 根据窗口宽度自动调整列数
                                let window_width = ui.available_width();
                                let item_width = 200.0; // 每个寄存器项的宽度
                                let columns = (window_width / item_width).max(1.0).min(5.0) as usize; // 最多5列
                                
                                // 使用网格布局显示寄存器数据，按阵列形式排列
                                let mut grid = egui::Grid::new(egui::Id::new("register_data"))
                                    .num_columns(columns)
                                    .spacing([20.0, 10.0]);
                                
                                grid.show(ui, |ui| {
                                    for (i, &value) in self.registers.iter().enumerate() {
                                        let address = self.start_address + i as u16;
                                        ui.group(|ui| {
                                            ui.label(format!("寄存器 {}", address));
                                            match self.display_style {
                                                DisplayStyle::Decimal => ui.label(format!("{}", value)),
                                                DisplayStyle::Hexadecimal => ui.label(format!("0x{:04X}", value)),
                                                DisplayStyle::Binary => ui.label(format!("{:016b}", value)),
                                            };
                                        });
                                        
                                        // 每列显示完后换行
                                        if (i + 1) % columns == 0 {
                                            ui.end_row();
                                        }
                                    }
                                    
                                    // 确保最后一行也换行
                                    if self.registers.len() % columns != 0 {
                                        ui.end_row();
                                    }
                                });
                            });
                    } else {
                        ui.label("尚未读取寄存器数据");
                    }
                });
            });
        });
    }
}
