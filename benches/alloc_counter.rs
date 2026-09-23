//! ベンチマーク用スレッドローカル・アロケーションカウンタ。
//!
//! グローバルアロケータをラップし、alloc バイト数と回数を記録する。
//! ベンチマークコードから `AllocSnapshot::now()` の前後の差分を取る。
//!
//! 変換 API の一部だったことはなく、本プロジェクトのベンチマーク
//! （`benches/memory.rs`）と計測用サンプル（`examples/{quick_mem,measure_mem}.rs`）
//! のためだけに存在する `dev` 専用ユーティリティ。以前は `mdka::alloc_counter`
//! として公開 API に含まれていたが、公開 API の一部だったことは一度もなく
//! （RFC 022 第二部、`2.4.0` で削除）、ここに移設した。
//!
//! CLI バイナリはこのアロケータを登録しない。計測器を出荷物に載せないための
//! 撤去は `2.2.2` で完了している。

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// グローバル統計（原子操作・スレッドセーフ）
pub static ALLOCATED_TOTAL: AtomicUsize = AtomicUsize::new(0);
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
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() && new_size > layout.size() {
            // realloc は成長分だけを新規確保として計上
            ALLOCATED_TOTAL.fetch_add(new_size - layout.size(), Relaxed);
        }
        new_ptr
    }
}

/// 計測スナップショット
#[derive(Debug, Clone, Copy, Default)]
pub struct AllocSnapshot {
    pub allocated: usize,
    pub count: usize,
}

impl AllocSnapshot {
    pub fn now() -> Self {
        Self {
            allocated: ALLOCATED_TOTAL.load(Relaxed),
            count: ALLOC_COUNT.load(Relaxed),
        }
    }

    /// スナップショット間の差分
    pub fn delta_since(&self, baseline: &Self) -> AllocDelta {
        AllocDelta {
            allocated_bytes: self.allocated.saturating_sub(baseline.allocated),
            alloc_count: self.count.saturating_sub(baseline.count),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AllocDelta {
    pub allocated_bytes: usize,
    // Read by quick_mem.rs and memory.rs, not by measure_mem.rs -- each
    // binary compiles this shared file separately, so dead_code sees only
    // its own use, not the type's use across all three.
    #[allow(dead_code)]
    pub alloc_count: usize,
}
