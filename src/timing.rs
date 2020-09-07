use std::time::{Instant, Duration};

pub struct Scheduler { 
    pub tasks: Vec<TemporalTask>,
}

impl Scheduler {
    pub fn new() -> Self {
        return Self {
            tasks: vec![],
        }
    }

    pub fn create_task(&mut self, task: TemporalTask) {
        self.tasks.push(task);
    }

    pub fn execute_pending(&mut self) {
        let now = Instant::now();
        for task in self.tasks.iter_mut() {
            if now.duration_since(task.last_execution) > task.interval {
                //println!("{:?}", now.duration_since(task.last_execution));
                &(task.execute_func)(task.ama.clone());
                task.last_execution = now;
            }
        }
    } 
}

pub struct TemporalTask { 
    pub interval : Duration,
    pub execute_func: fn(AMA),
    pub last_execution: Instant,
    pub ama: AMA
}

unsafe impl Send for TemporalTask {}

impl TemporalTask {
    pub fn new(interval: Duration, execute_func: fn(AMA), args: AMA) -> Self {
        Self { interval, execute_func, last_execution: Instant::now(), ama: args } 
    }
}

type AMA = std::sync::Arc<std::sync::Mutex<crate::app::App>>;