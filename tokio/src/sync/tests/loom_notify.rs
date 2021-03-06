use crate::sync::Notify;

use loom::future::block_on;
use loom::sync::Arc;
use loom::thread;

#[test]
fn notify_one() {
    loom::model(|| {
        let tx = Arc::new(Notify::new());
        let rx = tx.clone();

        let th = thread::spawn(move || {
            block_on(async {
                rx.notified().await;
            });
        });

        tx.notify_one();
        th.join().unwrap();
    });
}

#[test]
fn notify_waiters() {
    loom::model(|| {
        let notify = Arc::new(Notify::new());
        let tx = notify.clone();
        let notified1 = notify.notified();
        let notified2 = notify.notified();

        let th = thread::spawn(move || {
            tx.notify_waiters();
        });

        th.join().unwrap();

        block_on(async {
            notified1.await;
            notified2.await;
        });
    });
}

#[test]
fn notify_multi() {
    loom::model(|| {
        let notify = Arc::new(Notify::new());

        let mut ths = vec![];

        for _ in 0..2 {
            let notify = notify.clone();

            ths.push(thread::spawn(move || {
                block_on(async {
                    notify.notified().await;
                    notify.notify_one();
                })
            }));
        }

        notify.notify_one();

        for th in ths.drain(..) {
            th.join().unwrap();
        }

        block_on(async {
            notify.notified().await;
        });
    });
}

#[test]
fn notify_drop() {
    use crate::future::poll_fn;
    use std::future::Future;
    use std::task::Poll;

    loom::model(|| {
        let notify = Arc::new(Notify::new());
        let rx1 = notify.clone();
        let rx2 = notify.clone();

        let th1 = thread::spawn(move || {
            let mut recv = Box::pin(rx1.notified());

            block_on(poll_fn(|cx| {
                if recv.as_mut().poll(cx).is_ready() {
                    rx1.notify_one();
                }
                Poll::Ready(())
            }));
        });

        let th2 = thread::spawn(move || {
            block_on(async {
                rx2.notified().await;
                // Trigger second notification
                rx2.notify_one();
                rx2.notified().await;
            });
        });

        notify.notify_one();

        th1.join().unwrap();
        th2.join().unwrap();
    });
}
