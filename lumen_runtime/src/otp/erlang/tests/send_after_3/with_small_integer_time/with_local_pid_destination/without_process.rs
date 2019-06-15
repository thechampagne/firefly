use super::*;

use std::thread;
use std::time::Duration;

#[test]
fn sends_nothing_when_timer_expires() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &(milliseconds(), strategy::term(arc_process.clone())),
                |(milliseconds, message)| {
                    let destination = process::identifier::local::next();

                    let time = milliseconds.into_process(&arc_process);

                    let result =
                        erlang::send_after_3(time, destination, message, arc_process.clone());

                    prop_assert!(
                        result.is_ok(),
                        "Timer reference not returned.  Got {:?}",
                        result
                    );

                    let timer_reference = result.unwrap();

                    prop_assert_eq!(timer_reference.tag(), Boxed);

                    let unboxed_timer_reference: &Term = timer_reference.unbox_reference();

                    prop_assert_eq!(unboxed_timer_reference.tag(), LocalReference);

                    thread::sleep(Duration::from_millis(milliseconds + 1));
                    timer::timeout();

                    // does not send to original process either
                    prop_assert!(!has_message(&arc_process, message));

                    Ok(())
                },
            )
            .unwrap();
    });
}
