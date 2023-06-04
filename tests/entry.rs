mod cli;
mod fmn;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    use task_reminder::comm::get_local_utc_offset;
    task_reminder::setup_logger();
    get_local_utc_offset();
}
