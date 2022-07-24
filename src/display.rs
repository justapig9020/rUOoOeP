use std::cmp;
pub fn side_by_side(table1: String, table2: String, gap: usize) -> String {
    if table1.is_empty() {
        return table2;
    } else if table2.is_empty() {
        return table1;
    }
    let mut table1: Vec<&str> = table1.lines().collect();
    let mut table2: Vec<&str> = table2.lines().collect();

    let len1 = table1.len();
    let len2 = table2.len();
    let shorter = if len1 < len2 {
        &mut table1
    } else {
        &mut table2
    };
    let diff = len1.abs_diff(len2);
    for _ in 0..diff {
        shorter.push("");
    }

    let max = table1.iter().map(|s| s.len()).max().unwrap();
    let mut result = String::new();
    for (s1, s2) in table1.iter().zip(table2.iter()) {
        let pending = max - s1.len() + gap;
        result.push_str(s1);
        for _ in 0..pending {
            result.push_str(" ");
        }
        result.push_str(s2);
        result.push_str("\n");
    }
    result
}

pub fn into_table(title: &str, rows: Vec<String>) -> String {
    if rows.is_empty() {
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
