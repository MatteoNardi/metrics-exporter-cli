use std::fmt::Display;

#[derive(Debug)]
pub struct TableBuilder {
    path: Vec<String>,
    header: Vec<Entry>,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            header: Vec::new(),
            path: Vec::new(),
        }
    }

    fn new_subsection(path: Vec<String>) -> Self {
        Self {
            header: Vec::new(),
            path,
        }
    }

    pub fn group(mut self, name: &str, f: impl Fn(TableBuilder) -> TableBuilder) -> TableBuilder {
        let mut path = self.path.clone();
        path.push(name.to_string());
        self.header.push(Entry::Group(Group {
            name: name.to_string(),
            entries: f(TableBuilder::new_subsection(path)).header,
        }));
        self
    }

    pub fn field(mut self, name: &str) -> TableBuilder {
        let mut full_path = self.path.clone();
        full_path.push(name.to_string());
        self.header.push(Entry::Field(Field {
            name: name.to_string(),
            full_path,
            display: DisplayInfo {
                len: name.len(),
                display_kind: DisplayKind::Number,
            },
            _last_value: Value::Int(0),
        }));
        self
    }

    pub fn build(self) -> Table {
        let mut header_lines = Vec::new();

        let depth = depth(&self.header);
        header_lines.resize_with(depth, Default::default);
        fill_header_lines(&self.header, depth, &mut header_lines);
        header_lines
            .iter_mut()
            .for_each(|x| *x = x.trim_end().to_string());

        Table {
            header_lines,
            fields: collect_fields(self.header),
        }
    }
}

fn collect_fields(entries: Vec<Entry>) -> Vec<Field> {
    let mut result = Vec::new();
    for entry in entries {
        match entry {
            Entry::Group(group) => result.append(&mut collect_fields(group.entries)),
            Entry::Field(field) => result.push(field),
        }
    }
    result
}

fn depth(entries: &Vec<Entry>) -> usize {
    entries
        .iter()
        .map(|entry| match entry {
            Entry::Group(group) => depth(&group.entries) + 1,
            Entry::Field(_) => 1,
        })
        .max()
        .unwrap_or(0)
}

fn fill_header_lines(entries: &Vec<Entry>, depth: usize, mut lines: &mut Vec<String>) -> usize {
    let mut len = 0;
    let mut it = entries.iter().peekable();
    while let Some(entry) = it.next() {
        match entry {
            Entry::Group(group) => {
                let i = lines.len() - depth;
                let child_len = fill_header_lines(&group.entries, depth - 1, &mut lines);
                len += add_centered_str(&mut lines[i], &group.name, child_len);

                // Join groups with " | "
                if depth != 1 && it.peek().is_some() {
                    for j in i..lines.len() {
                        lines[j].push_str(&" | ");
                    }
                    len += 3;
                }
            }
            Entry::Field(field) => {
                if depth == 1 {
                    lines.last_mut().unwrap().push_str(&field.name);
                    len += field.display.len;
                } else {
                    len += fill_header_lines(&vec![entry.clone()], depth - 1, &mut lines);
                }
            }
        }
        // Join terminal fields with " "
        if depth == 1 && it.peek().is_some() {
            lines.last_mut().unwrap().push(' ');
            len += 1;
        }
    }
    len
}

/// Append value to output, making sure it takes at least minimum_len characters.
/// Return the number of added characters. If minimum_len is bigger than value.len(),
/// text will be centered.
fn add_centered_str(output: &mut String, value: &str, minimum_len: usize) -> usize {
    let initial_len = output.len();
    if minimum_len > value.len() {
        let extra_space_to_insert = minimum_len - value.len();
        for _ in 0..((extra_space_to_insert) / 2) {
            output.push(' ');
        }
        output.push_str(value);
        for _ in 0..((extra_space_to_insert + 1) / 2) {
            output.push(' ');
        }
    } else {
        output.push_str(value);
    }
    output.len() - initial_len
}

pub struct Table {
    header_lines: Vec<String>,
    fields: Vec<Field>,
    // last_values: Vec<Value>
}

#[derive(Clone, Debug)]
enum Entry {
    Group(Group),
    Field(Field),
}

#[derive(Clone, Debug)]
struct Group {
    name: String,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
struct Field {
    name: String,
    full_path: Vec<String>,
    display: DisplayInfo,
    // TODO:
    _last_value: Value,
}

#[derive(Clone, Debug)]
struct DisplayInfo {
    /// How much space the field should take
    len: usize,
    display_kind: DisplayKind,
}

#[derive(Clone, Debug)]
enum DisplayKind {
    Number,
    // TODO:
    _Difference,
    _Histogram,
}

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    F64(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(x) => write!(f, "{}", x),
            Value::F64(x) => write!(f, "{}", x),
        }
    }
}

// positional?
impl Table {
    pub fn header(&self) -> String {
        self.header_lines.join("\n")
    }

    // Given a list of path components, with the last one being the field and
    // the first ones the gorups, return the entry position in the table, if found.
    pub fn position_of(&self, path: Vec<String>) -> Option<usize> {
        self.fields.iter().position(|field| field.full_path == path)
    }

    // Each entry gets an associated index at build time, field should be supplied in order
    pub fn display_row(&self, values: Vec<Value>) -> String {
        let mut output = String::new();
        for (value, field) in values.iter().zip(&self.fields) {
            match field.display.display_kind {
                DisplayKind::Number => {
                    let v = value.to_string();
                    if field.display.len > v.len() {
                        for _ in 0..(field.display.len - v.len()) {
                            output.push(' ')
                        }
                        output.push_str(&v);
                    }
                }
                DisplayKind::_Difference => todo!(),
                DisplayKind::_Histogram => todo!(),
            }
            output.push(' ');
        }
        output.pop();
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_a() -> Table {
        TableBuilder::new()
            .group("input", |input| input.field("counter").field("counter2"))
            .build()
    }

    #[test]
    fn header_simple() {
        assert_eq!(
            table_a().header(),
            [
                //
                "     input",
                "counter counter2",
            ]
            .join("\n")
        );
    }

    #[test]
    fn index_simple() {
        let table = table_a();
        assert_eq!(
            table.position_of(vec!["input".to_string(), "counter".to_string()]),
            Some(0)
        );
        assert_eq!(
            table.position_of(vec!["input".to_string(), "counter2".to_string()]),
            Some(1)
        );
        assert_eq!(
            table.position_of(vec!["input".to_string(), "counter3".to_string()]),
            None
        );
    }

    #[test]
    fn value_simple() {
        let table = table_a();
        assert_eq!(&table.display_row(vec![2, 324]), "      2      324",);
    }

    fn table_b() -> Table {
        TableBuilder::new()
            .group("g1", |input| input.field("c1").field("c2"))
            .group("g2", |input| input.field("c3").field("c4"))
            .build()
    }

    #[test]
    fn header_with_multiple_groups() {
        assert_eq!(
            table_b().header(),
            [
                //
                " g1   |  g2",
                "c1 c2 | c3 c4",
            ]
            .join("\n")
        );
    }

    #[test]
    fn index_with_multiple_groups() {
        let table = table_b();
        assert_eq!(
            table.position_of(vec!["g1".to_string(), "c2".to_string()]),
            Some(1)
        );
        assert_eq!(
            table.position_of(vec!["g2".to_string(), "c3".to_string()]),
            Some(2)
        );
    }
}
