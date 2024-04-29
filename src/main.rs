#![no_std]
#![no_main]
#![allow(unused)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod serial;
pub mod vga_buffer;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
  println!("Hello World{}", "!");

  #[cfg(test)]
  test_main();

  loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  print!("{info:?}");
  loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failed);
  loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
  serial_println!("Running {} tests", tests.len());
  for test in tests {
    test.run();
  }
  exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
  assert_eq!(1, 1);
}

#[test_case]
fn test_println_simple() {
  println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
  for _ in 0..200 {
    println!("test_println_many output");
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
  Success = 0x10,
  Failed = 0x11,
}
pub fn exit_qemu(exit_code: QemuExitCode) {
  use x86_64::instructions::port::Port;

  unsafe {
    let mut port = Port::new(0xf4);
    port.write(exit_code as u32);
  }
}

pub trait Testable {
  fn run(&self) -> ();
}

impl<T> Testable for T
where
  T: Fn(),
{
  fn run(&self) {
    serial_print!("{}...\t", core::any::type_name::<T>());
    self();
    serial_println!("[ok]");
  }
}
