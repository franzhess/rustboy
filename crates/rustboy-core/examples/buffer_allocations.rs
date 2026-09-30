use rustboy_core::mbc::Cartridge;
use rustboy_core::{Machine, CPU_FREQUENCY};
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

const PROFILE_SECONDS: usize = 10;

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static REALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        REQUESTED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        REQUESTED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        REALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        REQUESTED_BYTES.fetch_add(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

#[derive(Clone, Copy)]
struct AllocatorSnapshot {
    allocations: usize,
    reallocations: usize,
    deallocations: usize,
    requested_bytes: usize,
}

impl AllocatorSnapshot {
    fn capture() -> Self {
        Self {
            allocations: ALLOCATIONS.load(Ordering::Relaxed),
            reallocations: REALLOCATIONS.load(Ordering::Relaxed),
            deallocations: DEALLOCATIONS.load(Ordering::Relaxed),
            requested_bytes: REQUESTED_BYTES.load(Ordering::Relaxed),
        }
    }

    fn difference(self, earlier: Self) -> Self {
        Self {
            allocations: self.allocations - earlier.allocations,
            reallocations: self.reallocations - earlier.reallocations,
            deallocations: self.deallocations - earlier.deallocations,
            requested_bytes: self.requested_bytes - earlier.requested_bytes,
        }
    }

    fn subtract(self, other: Self) -> Self {
        Self {
            allocations: self.allocations - other.allocations,
            reallocations: self.reallocations - other.reallocations,
            deallocations: self.deallocations - other.deallocations,
            requested_bytes: self.requested_bytes - other.requested_bytes,
        }
    }
}

struct Measurement {
    allocator: AllocatorSnapshot,
    cycles: usize,
    frames: usize,
    audio_buffers: usize,
}

fn machine_with_lcd_control(lcd_control: u8) -> Machine {
    let mut rom = vec![0; 0x8000];
    // LD A,lcd_control; LDH (LCDC),A; JR -2.
    rom[0x100..0x106].copy_from_slice(&[0x3E, lcd_control, 0xE0, 0x40, 0x18, 0xFE]);
    let mut machine = Machine::new(Cartridge::from_bytes(rom).expect("valid synthetic ROM"));
    machine.step();
    machine.step();
    machine
}

fn measure(machine: &mut Machine) -> Measurement {
    let target_cycles = CPU_FREQUENCY * PROFILE_SECONDS;
    let before = AllocatorSnapshot::capture();
    let mut cycles = 0;
    let mut frames = 0;
    let mut audio_buffers = 0;

    while cycles < target_cycles {
        let result = machine.step();
        cycles += result.cycles;
        if let Some(frame) = result.frame {
            frames += 1;
            black_box(frame.pixels());
        }
        for buffer in result.audio_buffers {
            audio_buffers += 1;
            black_box(buffer.samples());
        }
    }

    Measurement {
        allocator: AllocatorSnapshot::capture().difference(before),
        cycles,
        frames,
        audio_buffers,
    }
}

fn print_measurement(label: &str, measurement: &Measurement) {
    println!("{label}");
    println!("  cycles: {}", measurement.cycles);
    println!("  frames: {}", measurement.frames);
    println!("  audio buffers: {}", measurement.audio_buffers);
    println!("  allocations: {}", measurement.allocator.allocations);
    println!("  reallocations: {}", measurement.allocator.reallocations);
    println!("  deallocations: {}", measurement.allocator.deallocations);
    println!(
        "  requested bytes: {}",
        measurement.allocator.requested_bytes
    );
}

fn main() {
    let mut audio_only_machine = machine_with_lcd_control(0x00);
    let mut video_and_audio_machine = machine_with_lcd_control(0x91);

    let audio_only = measure(&mut audio_only_machine);
    let video_and_audio = measure(&mut video_and_audio_machine);
    assert_eq!(audio_only.cycles, video_and_audio.cycles);
    assert_eq!(audio_only.audio_buffers, video_and_audio.audio_buffers);

    let video = Measurement {
        allocator: video_and_audio.allocator.subtract(audio_only.allocator),
        cycles: video_and_audio.cycles,
        frames: video_and_audio.frames,
        audio_buffers: 0,
    };

    println!("Buffer allocation profile over {PROFILE_SECONDS} emulated seconds");
    print_measurement("audio only", &audio_only);
    print_measurement("video and audio", &video_and_audio);
    print_measurement("derived video cost", &video);
}
