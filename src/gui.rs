use eframe::egui;
use crate::ModbusSerialClient;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct ModbusApp {
    // 串口设置
    port: String,
    baud_rate: u32,
    slave_id: u8,
    available_ports: Vec<String>,
    
    // 寄存器设置
    start_address: u16,
    register_count: u16,
    register_aliases: Vec<String>,
    
    // 控制状态
    connected: bool,
    #[serde(skip)]
    client: Option<ModbusSerialClient>,
    registers: Vec<u16>,
    error_message: String,
    
    // 显示样式
    display_style: DisplayStyle,
    
    // 布局设置
    rows: usize,
    columns: usize,
    
    // 状态信息
    connection_status: String,
    packet_loss: u32,
    error_count: u32,
    last_error: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Serialize, Deserialize)]
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
        app.rows = 5;
        app.columns = 5;
        app.connection_status = "未连接".to_string();
        app.available_ports = crate::list_available_ports().unwrap_or_default().into_iter().map(|p| p.port_name).collect();
        app.load_config();
        app
    }
    
    // 加载配置
    fn load_config(&mut self) {
        let config_path = Self::get_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    *self = config;
                }
            }
        }
    }
    
    // 保存配置
    fn save_config(&self) {
        let config_path = Self::get_config_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&config_path, content);
        }
    }
    
    // 获取配置文件路径
    fn get_config_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("config.json");
        path
    }
    
    // 导入寄存器配置
    fn import_register_config(&mut self, path: &str) {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok((aliases, start_addr, count)) = serde_json::from_str(&content) {
                self.register_aliases = aliases;
                self.start_address = start_addr;
                self.register_count = count;
            }
        }
    }
    
    // 导出寄存器配置
    fn export_register_config(&self, path: &str) {
        let config = (&self.register_aliases, self.start_address, self.register_count);
        if let Ok(content) = serde_json::to_string_pretty(&config) {
            let _ = fs::write(path, content);
        }
    }
    
    fn connect(&mut self) {
        match ModbusSerialClient::new(&self.port, self.baud_rate) {
            Ok(client) => {
                self.client = Some(client);
                self.connected = true;
                self.error_message = "".to_string();
                self.connection_status = "已连接".to_string();
                self.save_config();
            }
            Err(e) => {
                self.error_message = format!("Failed to connect: {:?}", e);
                self.connected = false;
                self.connection_status = "连接失败".to_string();
                self.last_error = self.error_message.clone();
                self.error_count += 1;
            }
        }
    }
    
    fn read_registers(&mut self) {
        if let Some(client) = &mut self.client {
            match client.read_holding_registers(self.slave_id, self.start_address, self.register_count) {
                Ok(registers) => {
                    let registers_len = registers.len();
                    self.registers = registers;
                    self.error_message = "".to_string();
                    // 确保寄存器别名数量与寄存器数量一致
                    while self.register_aliases.len() < registers_len {
                        self.register_aliases.push(format!("寄存器 {}", self.start_address + self.register_aliases.len() as u16));
                    }
                    self.save_config();
                }
                Err(e) => {
                    self.error_message = format!("Failed to read registers: {:?}", e);
                    self.last_error = self.error_message.clone();
                    self.error_count += 1;
                    self.packet_loss += 1;
                }
            }
        } else {
            self.error_message = "Not connected".to_string();
            self.last_error = self.error_message.clone();
            self.error_count += 1;
        }
    }
    
    fn disconnect(&mut self) {
        self.client = None;
        self.connected = false;
        self.connection_status = "未连接".to_string();
        self.save_config();
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
            
            // 只保留一个标题
            ui.heading("AGS阀门控制");
            ui.separator();
            
            // 使用垂直布局
            egui::ScrollArea::vertical().show(ui, |ui| {
                // 第一行：串口设置及状态检测，控制栏和Modbus状态
                ui.horizontal(|ui| {
                    // 串口设置
                    ui.group(|ui| {
                        ui.heading("串口设置");
                        ui.add_space(10.0);
                        
                        // 使用网格布局
                        let grid = egui::Grid::new(egui::Id::new(1))
                            .num_columns(2)
                            .spacing([20.0, 10.0]);
                        
                        grid.show(ui, |ui| {
                            ui.label("端口:");
                            egui::ComboBox::from_id_source(egui::Id::new(2))
                                .selected_text(&self.port)
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    // 确保available_ports不为空
                                    if self.available_ports.is_empty() {
                                        self.available_ports = crate::list_available_ports().unwrap_or_default().into_iter().map(|p| p.port_name).collect();
                                    }
                                    
                                    for (i, port) in self.available_ports.iter().enumerate() {
                                        ui.selectable_value(&mut self.port, port.clone(), port);
                                    }
                                });
                            ui.end_row();
                            
                            ui.label("波特率:");
                            egui::ComboBox::from_id_source(egui::Id::new(3))
                                .selected_text(&self.baud_rate.to_string())
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    let baud_rates = vec![1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200];
                                    for (i, &rate) in baud_rates.iter().enumerate() {
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
                            
                            ui.add_space(10.0);
                            
                            // 导入导出按钮
                            if ui.button(egui::RichText::new("导入配置").size(14.0)).clicked() {
                                // 这里简化处理，实际应该使用文件对话框
                                self.import_register_config("register_config.json");
                            }
                            if ui.button(egui::RichText::new("导出配置").size(14.0)).clicked() {
                                // 这里简化处理，实际应该使用文件对话框
                                self.export_register_config("register_config.json");
                            }
                        });
                    });
                    
                    ui.add_space(20.0);
                    
                    // Modbus实时状态
                    ui.group(|ui| {
                        ui.heading("Modbus状态");
                        ui.add_space(10.0);
                        
                        egui::Grid::new(egui::Id::new(10))
                            .num_columns(2)
                            .spacing([40.0, 10.0])
                            .show(ui, |ui| {
                                ui.label("连接状态:");
                                ui.label(egui::RichText::new(&self.connection_status)
                                    .color(if self.connected { egui::Color32::GREEN } else { egui::Color32::RED }));
                                ui.end_row();
                                
                                ui.label("丢包次数:");
                                ui.label(format!("{}", self.packet_loss));
                                ui.end_row();
                                
                                ui.label("错误次数:");
                                ui.label(format!("{}", self.error_count));
                                ui.end_row();
                                
                                ui.label("最后错误:");
                                ui.label(egui::RichText::new(&self.last_error).color(egui::Color32::RED));
                                ui.end_row();
                            });
                    });
                });
                
                ui.add_space(20.0);
                
                // 第二栏：Modbus设置
                ui.group(|ui| {
                    ui.heading("Modbus设置");
                    ui.add_space(10.0);
                    
                    egui::Grid::new(egui::Id::new(11))
                        .num_columns(3)
                        .spacing([40.0, 10.0])
                        .show(ui, |ui| {
                            // 寄存器设置
                            ui.vertical(|ui| {
                                ui.heading("寄存器设置");
                                ui.add_space(5.0);
                                
                                let grid = egui::Grid::new(egui::Id::new(12))
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
                            
                            // 布局设置
                            ui.vertical(|ui| {
                                ui.heading("布局设置");
                                ui.add_space(5.0);
                                
                                let grid = egui::Grid::new(egui::Id::new(18))
                                    .num_columns(2)
                                    .spacing([20.0, 10.0]);
                                
                                grid.show(ui, |ui| {
                                    ui.label("行数:");
                                    ui.add(egui::DragValue::new(&mut self.rows)
                                        .range(1..=20));
                                    ui.end_row();
                                    
                                    ui.label("列数:");
                                    ui.add(egui::DragValue::new(&mut self.columns)
                                        .range(1..=10));
                                    ui.end_row();
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
                
                // 第四栏：寄存器数据
                ui.group(|ui| {
                    ui.heading("寄存器数据");
                    ui.add_space(10.0);
                    
                    if !self.registers.is_empty() {
                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .show(ui, |ui| {
                                // 使用用户设置的列数
                                let columns = self.columns.max(1).min(10);
                                
                                // 使用网格布局显示寄存器数据，按阵列形式排列
                                let grid = egui::Grid::new(egui::Id::new(21))
                                    .num_columns(columns)
                                    .spacing([20.0, 10.0]);
                                
                                grid.show(ui, |ui| {
                                    for (i, &value) in self.registers.iter().enumerate() {
                                        let address = self.start_address + i as u16;
                                        ui.group(|ui| {
                                            // 显示寄存器别名或默认名称
                                            let alias = if i < self.register_aliases.len() && !self.register_aliases[i].is_empty() {
                                                &self.register_aliases[i]
                                            } else {
                                                &format!("寄存器 {}", address)
                                            };
                                            ui.label(alias);
                                            match self.display_style {
                                                DisplayStyle::Decimal => ui.label(format!("{}", value)),
                                                DisplayStyle::Hexadecimal => ui.label(format!("0x{:04X}", value)),
                                                DisplayStyle::Binary => ui.label(format!("{:016b}", value)),
                                            };
                                            
                                            // 编辑寄存器别名
                                            if i < self.register_aliases.len() {
                                                ui.text_edit_singleline(&mut self.register_aliases[i]);
                                            } else {
                                                let mut new_alias = format!("寄存器 {}", address);
                                                ui.text_edit_singleline(&mut new_alias);
                                                self.register_aliases.push(new_alias);
                                            }
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
