use rgbemu::emulation::address_mapper::AddressMapper;
use rgbemu::emulation::cartridge::{Cartridge, CartridgeHeader, CartridgeMemory, CartridgeType};
use rgbemu::emulation::device::{Device, DeviceType, ExecutionState};
use rgbemu::emulation::mappers::Mapper;

fn create_test_cartridge(rom: &[u8]) -> Box<Cartridge> {
    let mut memory = CartridgeMemory::new(16384, 0);
    memory.rom[0x100..0x100 + rom.len()].copy_from_slice(rom);

    let cartridge_type = CartridgeType::new(0, 0, 0);
    let mapper = Mapper::from_cartridge_type(cartridge_type);
    let header = CartridgeHeader {
        title: None,
        supports_gbc: false,
        supports_sgb: false,
        cartridge_type,
        is_japanese: false
    };

    Box::new(Cartridge {
        memory,
        mapper,
        header
    })
}

pub fn run_program(code: &[u8]) -> Device {
    let mut device = Device::new(DeviceType::GameBoy, None);
    let cartridge = create_test_cartridge(code);

    device.bus.cartridge = Some(cartridge);

    while device.execution_state != ExecutionState::Halted {
        println!("{:?}", device.regs);
        device.run_tick();
    }

    device
}

pub fn read_address(device: &Device, address: u16) -> u8 {
    device.bus.read_8(device.bus.resolve_address(address))
}
