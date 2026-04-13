use clap::Parser;
use rustrrrrrrr::{ModbusSerialClient, list_available_ports};
use anyhow::Result;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "COM1")]
    port: String,
    
    #[clap(short, long, default_value = "9600")]
    baud_rate: u32,
    
    #[clap(short, long, default_value = "1")]
    slave_id: u8,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("Available serial ports:");
    let ports = list_available_ports()?;
    for port in &ports {
        println!("- {}", port.port_name);
    }
    
    println!("\nConnecting to port {} with baud rate {}...", args.port, args.baud_rate);
    let mut client = ModbusSerialClient::new(&args.port, args.baud_rate)?;
    
    println!("\nReading registers 0-99 (100 registers)...");
    let registers = client.read_holding_registers(args.slave_id, 0, 100)?;
    
    println!("\nRegister values:");
    for (i, value) in registers.iter().enumerate() {
        println!("Register {}: {}", i, value);
    }
    
    Ok(())
}
