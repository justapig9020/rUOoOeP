use std::cmp;
pub fn into_table(title: &str, rows: Vec<String>) -> String {
    if rows.len() <= 0 {
        return String::new();
    }
    let width = rows.iter().map(|s| s.len()).max().unwrap();
    let width = cmp::max(width, title.len());

    let height = rows.len();
    let size = (2 * height + 3) * (width + 3);
    let mut table = String::with_capacity(size);
    let bar: String = (0..width).map(|_| '-').collect();
    let divider = format!("+{}+\n", bar);

    table.push_str(&divider);
    let title_row = format!("|{}|\n", align_center(title, width));
    table.push_str(&title_row);
    for row in rows.iter() {
        table.push_str(&divider);
        let row = format!("|{}|\n", align_center(row, width));
        table.push_str(&row);
    }
    table.push_str(&divider);

    table
}

fn align_center(text: &str, size: usize) -> String {
    let mut s = String::with_capacity(size);
    let space = size - text.len();
    let front = space / 2;
    let rare = front + space % 2;
    let front_space: String = (0..front).map(|_| ' ').collect();
    let rare_space: String = (0..rare).map(|_| ' ').collect();
    s.push_str(&front_space);
    s.push_str(text);
    s.push_str(&rare_space);
    s
}
