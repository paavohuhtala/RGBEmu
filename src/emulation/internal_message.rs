
use emulation::interrupt::Interrupt;

#[derive(Debug, Clone)]
pub enum InternalMessage {
  None,
  TriggerInterrupt(Interrupt),
  DMATransfer { from: u16 },
  RendererMessage(RendererMessage)
}

#[derive(Debug, Clone)]
pub enum RendererMessage {
  RenderScanline(u8),
  PrepareNextFrame,
  PresentFrame
}
