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
        let mut header = Vec::new();

        let depth = depth(&self.header);
        header.resize_with(depth, Default::default);
        process(&self.header, depth, &mut header);

        Table {
            header,
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

fn process(entries: &Vec<Entry>, depth: usize, mut lines: &mut Vec<String>) -> usize {
    dbg!(entries);
    let mut len = 0;
    for entry in entries {
        match entry {
            Entry::Group(group) => {
                let i = lines.len() - depth;
                lines[i].push_str(" ");
                lines[i].push_str(&group.name);
                lines[i].push_str(" ");
                len += process(&group.entries, depth - 1, &mut lines);
            }
            Entry::Field(field) => {
                if depth == 1 {
                    lines.last_mut().unwrap().push_str(" ");
                    lines.last_mut().unwrap().push_str(&field.name);
                    lines.last_mut().unwrap().push_str(" ");
                    len += field.name.len() + 2;
                } else {
                    len += process(&vec![entry.clone()], depth - 1, &mut lines);
                }
            }
        }
    }
    len
}

pub struct Table {
    header: Vec<String>,
    fields: Vec<Field>,
    // groups: Vec<Vec<String>>,
    // fields: Vec<Field>,
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
        self.header.join("\n")
    }

    pub fn display_row(&self, values: Vec<Value>) -> String {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn composite_header() {
        let table = TableBuilder::new()
            .group("input", |input| input.field("counter").field("counter2"))
            .build();
        assert_eq!(
            table.header(),
            [
                //
                " input ",
                " counter  counter2 ",
                //"----------------"
            ]
            .join("\n")
        );
    }

    // #[test]
    // fn multiple_header() {
    //     unsafe {
    //         metrics::clear_recorder();
    //     }
    //     let register = CliRegister::install_on_thread();
    //     counter!("input.counter", 10);
    //     counter!("input.rate", 42);
    //     assert_eq!(register.header(), ["input", "counter rate"].join("\n"));
    // }
}
