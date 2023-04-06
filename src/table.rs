#[derive(Debug)]
pub struct TableBuilder {
    header: Vec<Entry>,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self { header: Vec::new() }
    }

    pub fn group(mut self, name: &str, f: impl Fn(TableBuilder) -> TableBuilder) -> TableBuilder {
        self.header.push(Entry::Group(Group {
            name: name.to_string(),
            entries: f(TableBuilder::new()).header,
        }));
        self
    }

    pub fn field(mut self, name: &str) -> TableBuilder {
        self.header.push(Entry::Field(Field {
            name: name.to_string(),
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
            Entry::Field(field) => 1,
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
                if child_len > group.name.len() {
                    let extra_space_to_insert = child_len - group.name.len();
                    for _ in 0..((extra_space_to_insert) / 2) {
                        lines[i].push(' ');
                    }
                    lines[i].push_str(&group.name);
                    for _ in 0..((extra_space_to_insert + 1) / 2) {
                        lines[i].push(' ');
                    }
                    len += child_len;
                } else {
                    lines[i].push_str(&group.name);
                    len += group.name.len();
                }

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
                    len += field.name.len();
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
    // len: usize,
}

type Value = i64;

// positional?
impl Table {
    pub fn header(&self) -> String {
        self.header_lines.join("\n")
    }

    pub fn display_row(&self, values: Vec<Value>) -> String {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn header_simple() {
        let table = TableBuilder::new()
            .group("input", |input| input.field("counter").field("counter2"))
            .build();
        assert_eq!(
            table.header(),
            [
                //
                "     input",
                "counter counter2",
            ]
            .join("\n")
        );
    }

    #[test]
    fn header_with_multiple_groups() {
        let table = TableBuilder::new()
            .group("g1", |input| input.field("c1").field("c2"))
            .group("g2", |input| input.field("c3").field("c4"))
            .build();
        assert_eq!(
            table.header(),
            [
                //
                " g1   |  g2",
                "c1 c2 | c3 c4",
            ]
            .join("\n")
        );
    }
}
