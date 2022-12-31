mod cli;
mod context;
mod fmn;
mod scheduler;
mod task;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    use task_reminder::comm::get_tzdiff;
    task_reminder::setup_logger();
    get_tzdiff();
}
