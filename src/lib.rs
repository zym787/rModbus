use serialport::{SerialPort, SerialPortInfo};
use std::time::Duration;
use anyhow::{Result, Context};

pub mod gui;

pub struct ModbusSerialClient {
    port: Box<dyn SerialPort>,
}

impl ModbusSerialClient {
    pub fn new(port: &str, baud_rate: u32) -> Result<Self> {
        let port = serialport::new(port, baud_rate)
            .timeout(Duration::from_millis(1000))
            .parity(serialport::Parity::None)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .open()
            .context("Failed to open serial port")?;
        Ok(Self { port })
    }

    pub fn read_holding_registers(&mut self, slave_id: u8, address: u16, count: u16) -> Result<Vec<u16>> {
        // 构建Modbus RTU读取保持寄存器命令
        let mut request = vec![
            slave_id,
            0x03, // 读取保持寄存器功能码
            (address >> 8) as u8, // 寄存器地址高字节
            (address & 0xFF) as u8, // 寄存器地址低字节
            (count >> 8) as u8, // 寄存器数量高字节
            (count & 0xFF) as u8, // 寄存器数量低字节
        ];
        
        // 计算CRC校验
        let crc = Self::calculate_crc(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);
        
        // 发送请求
        self.port.write_all(&request).context("Failed to write to serial port")?;
        
        // 读取响应
        let mut response = vec![0; 5 + (count * 2) as usize];
        self.port.read_exact(&mut response).context("Failed to read from serial port")?;
        
        // 验证响应
        if response[0] != slave_id || response[1] != 0x03 {
            return Err(anyhow::anyhow!("Invalid response"));
        }
        
        // 解析寄存器值
        let mut registers = Vec::new();
        for i in 0..count {
            let high = response[3 + (i * 2) as usize];
            let low = response[4 + (i * 2) as usize];
            registers.push((high as u16) << 8 | low as u16);
        }
        
        Ok(registers)
    }

    pub fn read_input_registers(&mut self, slave_id: u8, address: u16, count: u16) -> Result<Vec<u16>> {
        // 构建Modbus RTU读取输入寄存器命令
        let mut request = vec![
            slave_id,
            0x04, // 读取输入寄存器功能码
            (address >> 8) as u8, // 寄存器地址高字节
            (address & 0xFF) as u8, // 寄存器地址低字节
            (count >> 8) as u8, // 寄存器数量高字节
            (count & 0xFF) as u8, // 寄存器数量低字节
        ];
        
        // 计算CRC校验
        let crc = Self::calculate_crc(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);
        
        // 发送请求
        self.port.write_all(&request).context("Failed to write to serial port")?;
        
        // 读取响应
        let mut response = vec![0; 5 + (count * 2) as usize];
        self.port.read_exact(&mut response).context("Failed to read from serial port")?;
        
        // 验证响应
        if response[0] != slave_id || response[1] != 0x04 {
            return Err(anyhow::anyhow!("Invalid response"));
        }
        
        // 解析寄存器值
        let mut registers = Vec::new();
        for i in 0..count {
            let high = response[3 + (i * 2) as usize];
            let low = response[4 + (i * 2) as usize];
            registers.push((high as u16) << 8 | low as u16);
        }
        
        Ok(registers)
    }

    fn calculate_crc(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &byte in data {
            crc ^= byte as u16;
            for _ in 0..8 {
                if crc & 0x0001 != 0 {
                    crc >>= 1;
                    crc ^= 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc
    }
}

pub fn list_available_ports() -> Result<Vec<SerialPortInfo>> {
    serialport::available_ports().context("Failed to list available ports")
}
