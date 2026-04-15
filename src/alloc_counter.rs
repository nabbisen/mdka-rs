//! ベンチマーク用スレッドローカル・アロケーションカウンタ。
//!
//! グローバルアロケータをラップし、alloc/dealloc バイト数を
//! スレッドローカルに記録する。ベンチマークコードから
//! `reset()` / `snapshot()` で前後の差分を取る。

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// グローバル統計（原子操作・スレッドセーフ）
pub static ALLOCATED_TOTAL: AtomicUsize = AtomicUsize::new(0);
pub static FREED_TOTAL: AtomicUsize = AtomicUsize::new(0);
pub static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            ALLOCATED_TOTAL.fetch_add(layout.size(), Relaxed);
            ALLOC_COUNT.fetch_add(1, Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        FREED_TOTAL.fetch_add(layout.size(), Relaxed);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            // realloc はサイズ差分で計上
            if new_size > layout.size() {
                ALLOCATED_TOTAL.fetch_add(new_size - layout.size(), Relaxed);
            } else {
                FREED_TOTAL.fetch_add(layout.size() - new_size, Relaxed);
            }
        }
        new_ptr
    }
}

/// 計測スナップショット
#[derive(Debug, Clone, Copy, Default)]
pub struct AllocSnapshot {
    pub allocated: usize,
    pub freed: usize,
    pub count: usize,
}

impl AllocSnapshot {
    pub fn now() -> Self {
        Self {
            allocated: ALLOCATED_TOTAL.load(Relaxed),
            freed: FREED_TOTAL.load(Relaxed),
            count: ALLOC_COUNT.load(Relaxed),
        }
    }

    /// スナップショット間の差分
    pub fn delta_since(&self, baseline: &Self) -> AllocDelta {
        AllocDelta {
            allocated_bytes: self.allocated.saturating_sub(baseline.allocated),
            freed_bytes: self.freed.saturating_sub(baseline.freed),
            alloc_count: self.count.saturating_sub(baseline.count),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AllocDelta {
    pub allocated_bytes: usize,
    pub freed_bytes: usize,
    pub alloc_count: usize,
    // ピーク = 測定区間中の live bytes 最大値（近似: allocated - freed の最大）
}

impl AllocDelta {
    pub fn live_bytes(&self) -> usize {
        self.allocated_bytes.saturating_sub(self.freed_bytes)
    }
}
