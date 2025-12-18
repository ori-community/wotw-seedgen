use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Router,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tokio::{
    runtime::Runtime,
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
    time::{Instant, sleep_until},
};

pub fn init(
    router: Router,
    runtime: &mut Runtime,
    duration: Duration,
    inactive_send: UnboundedSender<()>,
) -> Router {
    let last_activity = Arc::new(Mutex::new(InactivityTimeout::new(duration)));

    spawn_last_activity_check(runtime, last_activity.clone(), inactive_send);

    router.layer(axum::middleware::from_fn_with_state(
        last_activity,
        update_last_activity,
    ))
}

struct InactivityTimeout {
    deadline: Instant,
    duration: Duration,
}

impl InactivityTimeout {
    pub fn new(duration: Duration) -> Self {
        let deadline = Instant::now() + duration;

        Self { deadline, duration }
    }

    fn reset(&mut self) {
        self.deadline = Instant::now() + self.duration;
    }

    fn is_elapsed(&self) -> bool {
        Instant::now() >= self.deadline
    }
}

type LastActivityState = Arc<Mutex<InactivityTimeout>>;

async fn update_last_activity(
    State(inactivity_timeout): State<LastActivityState>,
    request: Request,
    next: Next,
) -> Response {
    inactivity_timeout.lock().unwrap().reset();

    next.run(request).await
}

fn spawn_last_activity_check(
    runtime: &mut Runtime,
    last_activity: LastActivityState,
    inactive_send: UnboundedSender<()>,
) -> JoinHandle<()> {
    runtime.spawn(async move {
        loop {
            let deadline = {
                let last_activity = last_activity.lock().unwrap();

                if last_activity.is_elapsed() {
                    inactive_send.send(()).unwrap();

                    return;
                }

                last_activity.deadline
            };

            sleep_until(deadline).await;
        }
    })
}
