use prettytable::{row, Table};

use crate::task_manager::Task;

pub fn tabular_output(tasks: &Vec<Task>) -> String {
    let mut table = Table::new();
    table.add_row(row!["ID", "TYPE", "DESCRIPTION"]);
    for task in tasks {
        table.add_row(row![task.task_id, task.clock_type, task.description]);
    }
    table.to_string()
}
