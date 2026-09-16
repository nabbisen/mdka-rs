use super::*;

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "container prefix stack unbalanced at document end")]
fn unbalanced_container_stack_fires_at_finish() {
    // A deliberately unbalanced push: a quote entered and never left.
    let mut sink = Sink::new(16);
    sink.enter_blockquote();
    sink.text("x");
    let _ = sink.finish();
}

#[test]
fn balanced_container_stack_finishes() {
    let mut sink = Sink::new(16);
    sink.enter_blockquote();
    sink.text("x");
    sink.leave_blockquote();
    assert_eq!(sink.finish(), "> x\n");
}
